use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use glam::{IVec2, Mat4, Vec2, Vec3};

use crate::{
    config::{
        Config,
        aim::{AimbotConfig, KeyMode, RcsConfig, TriggerbotConfig},
        player::VisibilityMode,
    },
    constants::cs2::{self, TEAM_CT, TEAM_T},
    cs2::{
        bones::Bones,
        entity::{
            Entity, EntityInfo, grenade_info, planted_c4::PlantedC4, player::Player, weapon::Weapon,
        },
        features::{aimbot::Aimbot, esp_toggle::EspToggle, rcs::Recoil, triggerbot::Triggerbot},
        input::Input,
        key_codes::KeyCode,
        offsets::Offsets,
        target::Target,
    },
    data::{Data, PlayerData},
    math::{angles_from_vector, vec2_clamp},
    os::{mouse::Mouse, process::Process},
    parser::{
        bvh::{Aabb, Bvh},
        read_map,
    },
};

pub mod bones;
pub mod bvh;
pub mod entity;
mod features;
mod find_offsets;
mod input;
pub mod key_codes;
mod offsets;
mod schema;
mod target;

pub struct CS2 {
    is_valid: bool,
    process: Process,
    offsets: Offsets,
    input: Input,
    bvh: Option<Bvh>,
    current_bvh: String,
    target: Target,
    players: Vec<Player>,
    dead_players: Vec<Player>,
    entities: Vec<Entity>,
    recoil: Recoil,
    aim: Aimbot,
    trigger: Triggerbot,
    esp: EspToggle,
    weapon: Weapon,
    planted_c4: Option<PlantedC4>,
    last_cache: Instant,
}

fn player_hitboxes(bones: &HashMap<Bones, Vec3>) -> Vec<Aabb> {
    const HITBOX_PADDING: f32 = 4.0;
    let padding = Vec3::splat(HITBOX_PADDING);

    Bones::CONNECTIONS
        .iter()
        .filter_map(|(a, b)| {
            let a = *bones.get(a)?;
            let b = *bones.get(b)?;
            Some(Aabb::from_points(&[
                a - padding,
                a + padding,
                b - padding,
                b + padding,
            ]))
        })
        .collect()
}

fn hitboxes_block_line(hitboxes: &[Aabb], start: Vec3, end: Vec3) -> bool {
    let direction = end - start;
    let distance = direction.length();
    if distance <= f32::EPSILON {
        return false;
    }

    let inv_direction = 1.0 / (direction / distance);
    hitboxes
        .iter()
        .any(|hitbox| hitbox.ray_intersect(start, inv_direction, distance))
}

impl CS2 {
    pub fn is_valid(&self) -> bool {
        self.is_valid && self.process.is_valid()
    }

    pub fn setup(&mut self) {
        let Some(process) = Process::open(cs2::PROCESS_NAME) else {
            self.is_valid = false;
            return;
        };
        utils::info!("process found, pid: {}", process.pid);
        self.process = process;

        self.offsets = match self.find_offsets() {
            Some(offsets) => offsets,
            None => {
                self.process = Process::new(-1);
                self.is_valid = false;
                return;
            }
        };
        utils::info!("offsets found");

        self.is_valid = true;
    }

    pub fn run(&mut self, config: &Config, mouse: &mut Mouse) {
        if !self.process.is_valid() {
            self.is_valid = false;
            utils::debug!("process is no longer valid");
            return;
        }

        self.input.update(&self.process, &self.offsets);

        if self.last_cache.elapsed() > Duration::from_millis(200) {
            self.cache_entities();
            self.check_bvh();
            self.last_cache = Instant::now();
        }

        for entity in &self.entities {
            if let Entity::Smoke(smoke) = entity {
                if config.misc.no_smoke {
                    smoke.disable(self);
                }

                if config.misc.change_smoke_color {
                    smoke.color(self, &config.misc.smoke_color);
                }
            }
        }

        self.no_flash(config);
        self.fov_changer(config);

        self.esp_toggle(config);

        self.triggerbot(config);

        self.triggerbot_shoot(mouse);

        self.find_target(config);

        if !self.aimbot(config, mouse) {
            self.rcs(config, mouse);
        }
    }

    pub fn data(&self, config: &Config, data: &mut Data) {
        data.players.clear();
        data.friendlies.clear();
        data.spectators.clear();
        data.entities.clear();

        let sdl_window: usize = self.process.read(self.offsets.direct.sdl_window);
        if sdl_window == 0 {
            data.window_position = Vec2::ZERO;
            data.window_size = Vec2::ONE;
        } else {
            data.window_position = self.process.read::<IVec2>(sdl_window + 0x18).as_vec2();
            data.window_size = self
                .process
                .read::<IVec2>(sdl_window + 0x18 + 0x08)
                .as_vec2();
        }

        let Some(local_player) = Player::local_player(self) else {
            data.weapon = Weapon::default();
            data.in_game = false;
            return;
        };
        let local_team = local_player.team(self);
        if local_team != TEAM_T && local_team != TEAM_CT {
            data.weapon = Weapon::default();
            data.in_game = false;
            return;
        }
        let is_ffa = self.is_ffa();
        let spectator_target = local_player.spectator_target(self);
        let active_player = spectator_target.unwrap_or(local_player);
        let active_pawn = active_player.pawn;

        let mut bones_by_pawn: HashMap<usize, _> = self
            .players
            .iter()
            .map(|player| (player.pawn, player.all_bones(self)))
            .collect();
        let hitboxes_by_pawn: HashMap<usize, _> = bones_by_pawn
            .iter()
            .map(|(&pawn, bones)| (pawn, player_hitboxes(bones)))
            .collect();
        let filter_visibility = config.player.visibility_mode != VisibilityMode::All;
        let ray_origin = active_player.eye_position(self);

        for player in &self.players {
            if spectator_target.is_some() && player.pawn == active_pawn {
                continue;
            }

            let bones = bones_by_pawn.remove(&player.pawn).unwrap_or_default();
            let visible = player.visible(self, &active_player);
            let visible_bones: HashSet<Bones> = if filter_visibility {
                bones
                    .iter()
                    .filter_map(|(&bone, &position)| {
                        let world_visible = player.visible_at(self, ray_origin, position);
                        let player_visible = !hitboxes_by_pawn.iter().any(|(&pawn, hitboxes)| {
                            pawn != player.pawn
                                && hitboxes_block_line(hitboxes, ray_origin, position)
                        });
                        (world_visible && player_visible).then_some(bone)
                    })
                    .collect()
            } else {
                Default::default()
            };
            let visible = if filter_visibility {
                !visible_bones.is_empty()
            } else {
                visible
            };

            let player_data = PlayerData {
                steam_id: player.steam_id(self),
                health: player.health(self),
                armor: player.armor(self),
                position: player.position(self),
                head: bones.get(&Bones::Head).copied().unwrap_or_default(),
                name: player.name(self),
                weapon: player.weapon(self),
                ammo: (player.clip_ammo(self), player.reserve_ammo(self)),
                bones,
                visible_bones,
                has_defuser: player.has_defuser(self),
                has_helmet: player.has_helmet(self),
                has_bomb: player.has_bomb(self),
                visible,
                color: player.color(self),
                rotation: player.rotation(self),
                sound: player.is_making_sound(self),
            };

            if !is_ffa && player.team(self) == local_team {
                data.friendlies.push(player_data);
            } else {
                data.players.push(player_data);
            }
        }

        for player in &self.dead_players {
            if let Some(target) = player.spectator_target(self)
                && target.pawn == active_pawn
            {
                data.spectators.push(player.name(self));
            }
        }

        data.local_player = PlayerData {
            steam_id: local_player.steam_id(self),
            health: local_player.health(self),
            armor: local_player.armor(self),
            position: local_player.position(self),
            head: local_player.bone_position(self, Bones::Head.u64()),
            name: local_player.name(self),
            weapon: local_player.weapon(self),
            ammo: (
                local_player.clip_ammo(self),
                local_player.reserve_ammo(self),
            ),
            bones: local_player.all_bones(self),
            visible_bones: Default::default(),
            has_defuser: local_player.has_defuser(self),
            has_helmet: local_player.has_helmet(self),
            has_bomb: local_player.has_bomb(self),
            visible: true,
            color: local_player.color(self),
            rotation: local_player.rotation(self),
            sound: None,
        };

        data.entities.clear();
        for entity in &self.entities {
            data.entities.push(match entity {
                Entity::Weapon { weapon, entity } => EntityInfo::Weapon {
                    weapon: weapon.clone(),
                    position: Player::entity(*entity).position(self),
                    ammo: (
                        Weapon::clip_ammo(*entity, self),
                        Weapon::reserve_ammo(*entity, self),
                    ),
                },
                Entity::Inferno(inferno) => EntityInfo::Inferno(inferno.info(self)),
                Entity::Smoke(smoke) => EntityInfo::Smoke(smoke.info(self)),
                Entity::Molotov(molotov) => EntityInfo::Molotov(molotov.info(self)),
                Entity::Flashbang(entity) => {
                    EntityInfo::Flashbang(grenade_info(*entity, "Flashbang", self))
                }
                Entity::HeGrenade(entity) => {
                    EntityInfo::HeGrenade(grenade_info(*entity, "HE Grenade", self))
                }
                Entity::Decoy(entity) => EntityInfo::Decoy(grenade_info(*entity, "Decoy", self)),
            });
        }

        data.weapon = local_player.weapon(self);
        data.in_game = true;
        data.is_ffa = is_ffa;
        data.map_name = self.current_map();
        data.aimbot_active = if self.aimbot_config(config).mode == KeyMode::Toggle {
            self.aim.active
        } else {
            false
        };
        data.triggerbot_active = if self.triggerbot_config(config).mode == KeyMode::Toggle {
            self.trigger.active
        } else {
            false
        };
        data.esp_active = self.esp_enabled(config);

        data.view_matrix = self.process.read::<Mat4>(self.offsets.direct.view_matrix);
        data.view_angles = local_player.view_angles(self);

        if let Some(bomb) = &self.planted_c4 {
            data.bomb.planted = bomb.is_planted(self);
            data.bomb.timer = bomb.time_to_explosion(self);
            data.bomb.position = bomb.position(self);
            data.bomb.being_defused = bomb.is_being_defused(self);
            data.bomb.defuse_remain_time = bomb.time_to_defuse(self);
        } else {
            data.bomb.planted = false;
        }
    }

    pub fn new() -> Self {
        Self {
            is_valid: false,
            process: Process::new(-1),
            offsets: Offsets::default(),
            input: Input::new(),
            bvh: None,
            current_bvh: String::new(),
            target: Target::default(),
            players: Vec::with_capacity(64),
            dead_players: Vec::with_capacity(12),
            entities: Vec::with_capacity(128),
            recoil: Recoil::default(),
            aim: Aimbot::default(),
            trigger: Triggerbot::default(),
            esp: EspToggle::default(),
            weapon: Weapon::default(),
            planted_c4: None,
            last_cache: Instant::now(),
        }
    }

    fn aimbot_config<'a>(&self, config: &'a Config) -> &'a AimbotConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.aimbot.enable_override
        {
            return &weapon_config.aimbot;
        }
        &config.aim.global.aimbot
    }

    fn rcs_config<'a>(&self, config: &'a Config) -> &'a RcsConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.rcs.enable_override
        {
            return &weapon_config.rcs;
        }
        &config.aim.global.rcs
    }

    fn triggerbot_config<'a>(&self, config: &'a Config) -> &'a TriggerbotConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.triggerbot.enable_override
        {
            return &weapon_config.triggerbot;
        }
        &config.aim.global.triggerbot
    }

    fn angle_to_target(&self, local_player: &Player, position: &Vec3, aim_punch: &Vec2) -> Vec2 {
        let eye_position = local_player.eye_position(self);
        let forward = (position - eye_position).normalize();

        let mut angles = angles_from_vector(&forward) - aim_punch;
        vec2_clamp(&mut angles);

        angles
    }

    fn entity_has_owner(&self, entity: usize) -> bool {
        self.process
            .read::<i32>(entity + self.offsets.controller.owner_entity)
            != -1
    }

    // convars
    fn get_sensitivity(&self) -> f32 {
        self.process.read(self.offsets.convar.sensitivity + 0x58)
    }

    fn is_ffa(&self) -> bool {
        self.process.read::<u8>(self.offsets.convar.ffa + 0x58) == 1
    }

    fn current_time(&self) -> f32 {
        let global_vars: usize = self.process.read(self.offsets.direct.global_vars);
        self.process.read(global_vars + 0x30)
    }

    fn current_map(&self) -> String {
        let global_vars: usize = self.process.read(self.offsets.direct.global_vars);
        self.process
            .read_string(self.process.read(global_vars + 0x198))
    }

    fn distance_scale(&self, distance: f32) -> f32 {
        if distance > 500.0 {
            1.0
        } else {
            5.0 - (distance / 125.0)
        }
    }

    fn check_bvh(&mut self) {
        let current_map = self.current_map();
        if current_map != self.current_bvh {
            self.bvh = read_map(self);
            if self.bvh.is_some() {
                utils::info!("loaded bvh for {current_map}");
                self.current_bvh = current_map;
            }
        }
    }

    fn check_hotkey(input: &Input, mode: KeyMode, key: KeyCode, active: &mut bool) -> bool {
        match mode {
            KeyMode::Hold => input.is_key_pressed(key),
            KeyMode::Toggle => {
                if input.key_just_pressed(key) {
                    *active = !*active;
                }
                *active
            }
        }
    }
}
