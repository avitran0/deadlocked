use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use glam::{IVec2, Mat4, Vec2, Vec3};
use utils::{log, sync::Mutex};

use crate::{
    config::{AimbotConfig, Config, DrawMode, KeyMode, RcsConfig, TriggerbotConfig},
    constants::cs2::{self, TEAM_CT, TEAM_T},
    cs2::{
        bones::Bones,
        entity::{
            Entity, EntityInfo, GrenadeInfo, planted_c4::PlantedC4, player::Player, weapon::Weapon,
        },
        features::{
            aimbot::Aimbot, counter_strafe::CounterStrafe, esp_toggle::EspToggle, rcs::Recoil,
            triggerbot::Triggerbot,
        },
        input::Input,
        offsets::Offsets,
        target::Target,
    },
    data::{Data, PlayerData},
    game::Game,
    math::{angles_from_vector, vec2_clamp},
    os::{mouse::Mouse, process::Process},
    parser::{bvh::Bvh, load_map},
    ui::grenades::{Grenade, GrenadeList},
};

pub mod bones;
pub mod entity;
mod features;
mod find_offsets;
mod input;
pub mod key_codes;
mod offsets;
mod schema;
mod target;

const WORLD_SCAN_INTERVAL: Duration = Duration::from_millis(50);
const BVH_CHECK_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct CS2 {
    is_valid: bool,
    process: Process,
    offsets: Offsets,
    input: Input,
    bvh: Option<Bvh>,
    current_bvh: String,
    target: Target,
    players: Vec<Player>,
    entities: Vec<Entity>,
    recoil: Recoil,
    aim: Aimbot,
    trigger: Triggerbot,
    counter_strafe: CounterStrafe,
    esp: EspToggle,
    weapon: Weapon,
    planted_c4: Option<PlantedC4>,
    next_world_scan: Instant,
    next_bvh_check: Instant,
    grenades: Arc<Mutex<GrenadeList>>,
    target_grenade: Option<Grenade>,
}

impl Game for CS2 {
    fn is_valid(&self) -> bool {
        self.is_valid && self.process.is_valid()
    }

    fn setup(&mut self) {
        let Some(process) = Process::open(cs2::PROCESS_NAME) else {
            self.is_valid = false;
            return;
        };
        log::info!("process found, pid: {}", process.pid);
        self.process = process;

        self.offsets = match self.find_offsets() {
            Some(offsets) => offsets,
            None => {
                self.process = Process::new(-1);
                self.is_valid = false;
                return;
            }
        };
        log::info!("offsets found");

        self.is_valid = true;
    }

    fn run(&mut self, config: &Config, mouse: &mut Mouse) {
        if !self.process.is_valid() {
            self.is_valid = false;
            log::debug!("process is no longer valid");
            return;
        }

        self.input.update(&self.process, &self.offsets);

        let now = Instant::now();
        let requires_strict_occlusion = self.aimbot_config(config).smoke_wall_check
            || self.triggerbot_config(config).smoke_wall_check;
        let world_scan_enabled = config.hud.bomb_timer
            || config.hud.dropped_weapons
            || config.hud.grenade_trails
            || config.misc.no_smoke
            || config.misc.change_smoke_color
            || requires_strict_occlusion;

        if world_scan_enabled && now >= self.next_world_scan {
            self.cache_entities();
            self.next_world_scan = now + WORLD_SCAN_INTERVAL;
        } else {
            self.cache_players();
            if !world_scan_enabled {
                self.entities.clear();
                self.planted_c4 = None;
            }
        }

        if now >= self.next_bvh_check {
            self.check_bvh();
            self.next_bvh_check = now + BVH_CHECK_INTERVAL;
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

        self.counter_strafe_tick(config, mouse);
        self.rcs(config, mouse);
        self.find_target(config);
        self.aimbot(config, mouse);
        self.triggerbot(config, mouse);
        self.triggerbot_shoot(mouse);
    }

    fn data(&self, config: &Config, data: &mut Data) {
        data.players.clear();
        data.friendlies.clear();
        data.entities.clear();

        let sdl_window = self.process.read::<u64>(self.offsets.direct.sdl_window);
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

        let player_esp_enabled = config.player.enabled;
        let collect_player_names = player_esp_enabled && config.player.player_name;
        let collect_player_bones = player_esp_enabled
            && (config.player.draw_skeleton != DrawMode::None || config.player.head_circle);
        let collect_player_tags = player_esp_enabled && config.player.tags;
        let collect_player_visibility = player_esp_enabled
            && (config.player.visible_only
                || (config.player.sound.enabled && config.player.sound.show_visible)
                || matches!(config.player.draw_box, DrawMode::Color));
        let collect_player_sound = player_esp_enabled && config.player.sound.enabled;

        for player in &self.players {
            let player_data = PlayerData {
                steam_id: player.steam_id(self),
                health: player.health(self),
                armor: player.armor(self),
                position: player.position(self),
                head: player.bone_position(self, Bones::Head.u64()),
                name: if collect_player_names {
                    player.name(self)
                } else {
                    String::new()
                },
                weapon: player.weapon(self),
                bones: if collect_player_bones {
                    player.all_bones(self)
                } else {
                    Default::default()
                },
                has_defuser: collect_player_tags && player.has_defuser(self),
                has_helmet: collect_player_tags && player.has_helmet(self),
                has_bomb: collect_player_tags && player.has_bomb(self),
                visible: if collect_player_visibility {
                    player.visible(self, &local_player)
                } else {
                    true
                },
                color: player.color(self),
                rotation: player.rotation(self),
                sound: if collect_player_sound {
                    player.is_making_sound(self)
                } else {
                    None
                },
            };

            if !self.is_ffa() && player.team(self) == local_team {
                data.friendlies.push(player_data);
            } else {
                data.players.push(player_data);
            }
        }

        data.local_player = PlayerData {
            steam_id: local_player.steam_id(self),
            health: local_player.health(self),
            armor: local_player.armor(self),
            position: local_player.position(self),
            head: local_player.bone_position(self, Bones::Head.u64()),
            name: String::new(),
            weapon: local_player.weapon(self),
            bones: Default::default(),
            has_defuser: false,
            has_helmet: false,
            has_bomb: false,
            visible: true,
            color: local_player.color(self),
            rotation: local_player.rotation(self),
            sound: None,
        };

        data.entities = self
            .entities
            .iter()
            .map(|e| match e {
                Entity::Weapon { weapon, entity } => EntityInfo::Weapon {
                    weapon: weapon.clone(),
                    position: Player::entity(*entity).position(self),
                },
                Entity::Inferno(inferno) => EntityInfo::Inferno(inferno.info(self)),
                Entity::Smoke(smoke) => EntityInfo::Smoke(smoke.info(self)),
                Entity::Molotov(molotov) => EntityInfo::Molotov(molotov.info(self)),
                Entity::Flashbang(entity) => {
                    EntityInfo::Flashbang(GrenadeInfo::new(*entity, "Flashbang", self))
                }
                Entity::HeGrenade(entity) => {
                    EntityInfo::HeGrenade(GrenadeInfo::new(*entity, "HE Grenade", self))
                }
                Entity::Decoy(entity) => {
                    EntityInfo::Decoy(GrenadeInfo::new(*entity, "Decoy", self))
                }
            })
            .collect();

        data.weapon = local_player.weapon(self);
        data.in_game = true;
        data.is_ffa = self.is_ffa();
        data.is_custom_mode = self.is_custom_game_mode();
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
}

impl CS2 {
    pub fn new(grenades: Arc<Mutex<GrenadeList>>) -> Self {
        Self {
            is_valid: false,
            process: Process::new(-1),
            offsets: Offsets::default(),
            input: Input::new(),
            bvh: None,
            current_bvh: String::new(),
            target: Target::default(),
            players: Vec::with_capacity(64),
            entities: Vec::with_capacity(128),
            recoil: Recoil::default(),
            aim: Aimbot::default(),
            trigger: Triggerbot::default(),
            counter_strafe: CounterStrafe::default(),
            esp: EspToggle::default(),
            weapon: Weapon::default(),
            planted_c4: None,
            next_world_scan: Instant::now(),
            next_bvh_check: Instant::now(),
            grenades,
            target_grenade: None,
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

    pub(crate) fn is_path_clear(&self, start: Vec3, end: Vec3) -> bool {
        if let Some(bvh) = &self.bvh {
            if !bvh.has_line_of_sight(start, end) {
                return false;
            }
        } else {
            // If we cannot validate geometry LOS, fail closed to avoid aiming through walls.
            return false;
        }

        !self.segment_hits_smoke(start, end)
    }

    fn segment_hits_smoke(&self, start: Vec3, end: Vec3) -> bool {
        const SMOKE_RADIUS: f32 = 145.0;
        const SMOKE_RADIUS_SQ: f32 = SMOKE_RADIUS * SMOKE_RADIUS;

        let segment = end - start;
        let segment_len_sq = segment.length_squared();

        for entity in &self.entities {
            let Entity::Smoke(smoke) = entity else {
                continue;
            };

            let center = smoke.info(self).position;
            let t = if segment_len_sq > f32::EPSILON {
                ((center - start).dot(segment) / segment_len_sq).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let closest = start + segment * t;

            if center.distance_squared(closest) <= SMOKE_RADIUS_SQ {
                return true;
            }
        }

        false
    }

    fn entity_has_owner(&self, entity: u64) -> bool {
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

    fn is_custom_game_mode(&self) -> bool {
        let map = self.current_map();
        map.starts_with("workshop/")
            || map.starts_with("custom/")
            || !map.starts_with("de_") && !map.starts_with("cs_")
    }

    fn current_time(&self) -> f32 {
        let global_vars: u64 = self.process.read(self.offsets.direct.global_vars);
        self.process.read(global_vars + 0x30)
    }

    fn current_map(&self) -> String {
        let global_vars: u64 = self.process.read(self.offsets.direct.global_vars);
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
            self.bvh = load_map(&current_map);
            if self.bvh.is_some() {
                log::info!("loaded bvh for {current_map}");
                self.current_bvh = current_map;
            }
        }
    }
}
