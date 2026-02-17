use std::{collections::HashMap, sync::Arc};

use glam::{IVec2, Mat4, Vec2, Vec3};
use parking_lot::Mutex;

use crate::{
    config::{AimbotConfig, Config, KeyMode, RcsConfig, TriggerbotConfig},
    constants::cs2::{self, TEAM_CT, TEAM_T},
    cs2::{
        bones::Bones,
        entity::{
            Entity, EntityInfo, GrenadeInfo, planted_c4::PlantedC4, player::Player, weapon::Weapon,
        },
        features::{aimbot::Aimbot, esp_toggle::EspToggle, rcs::Recoil, triggerbot::Triggerbot},
        key_codes::KeyCode,
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
pub mod key_codes;
mod offsets;
mod schema;
mod target;

#[derive(Debug)]
pub struct CS2 {
    is_valid: bool,
    process: Process,
    offsets: Offsets,
    bvh: Option<Bvh>,
    current_bvh: String,
    target: Target,
    players: Vec<Player>,
    entities: Vec<Entity>,
    recoil: Recoil,
    aim: Aimbot,
    trigger: Triggerbot,
    esp: EspToggle,
    weapon: Weapon,
    planted_c4: Option<PlantedC4>,
    grenades: Arc<Mutex<GrenadeList>>,
    target_grenade: Option<Grenade>,
    cache_timer: u32,
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

        if self.cache_timer == 0 {
            self.cache_entities();
            self.cache_timer = 5;
        } else {
            self.cache_timer -= 1;
        }

        self.check_bvh();

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

        self.rcs(config, mouse);
        self.triggerbot(config);

        self.triggerbot_shoot(mouse);

        self.find_target(config);

        self.aimbot(config, mouse);
    }

    fn data(&self, config: &Config, data: &mut Data) {
        data.players.clear();
        data.friendlies.clear();
        data.entities.clear();
        data.spectators.clear();
        data.spectator_names.clear();

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

        let bones_needed = config.player.enabled || config.hud.fov_circle || self.aimbot_config(config).enabled;

        for player in &self.players {
            let bones = if bones_needed {
                player.all_bones(self)
            } else {
                HashMap::new()
            };
            let weapon = player.weapon(self);
            let head = if bones_needed {
                bones.get(&Bones::Head).copied().unwrap_or_default()
            } else {
                player.bone_position(self, Bones::Head.u64())
            };

            let player_data = PlayerData {
                steam_id: player.steam_id(self),
                health: player.health(self),
                armor: player.armor(self),
                position: player.position(self),
                head,
                name: player.name(self),
                weapon: weapon.clone(),
                has_defuser: player.has_defuser(self),
                has_helmet: player.has_helmet(self),
                has_bomb: player.has_bomb(self),
                visible: player.visible(self, &local_player, if bones_needed { Some(&bones) } else { None }),
                color: player.color(self),
                rotation: player.rotation(self),
                sound: player.is_making_sound_optimized(self, &weapon),
                bones,
            };

            if !self.is_ffa() && player.team(self) == local_team {
                data.friendlies.push(player_data);
            } else {
                data.players.push(player_data);
            }
        }

        let local_bones = if bones_needed {
            local_player.all_bones(self)
        } else {
            HashMap::new()
        };
        let local_head = if bones_needed {
            local_bones.get(&Bones::Head).copied().unwrap_or_default()
        } else {
            local_player.bone_position(self, Bones::Head.u64())
        };

        data.local_player = PlayerData {
            steam_id: local_player.steam_id(self),
            health: local_player.health(self),
            armor: local_player.armor(self),
            position: local_player.position(self),
            head: local_head,
            name: local_player.name(self),
            weapon: local_player.weapon(self),
            has_defuser: local_player.has_defuser(self),
            has_helmet: local_player.has_helmet(self),
            has_bomb: local_player.has_bomb(self),
            visible: true,
            color: local_player.color(self),
            rotation: local_player.rotation(self),
            sound: None,
            bones: local_bones,
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
        data.wallhack_active = self.esp_enabled(config);

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
            bvh: None,
            current_bvh: String::new(),
            target: Target::default(),
            players: Vec::with_capacity(64),
            entities: Vec::with_capacity(128),
            recoil: Recoil::default(),
            aim: Aimbot::default(),
            trigger: Triggerbot::default(),
            esp: EspToggle::default(),
            weapon: Weapon::default(),
            planted_c4: None,
            grenades,
            target_grenade: None,
            cache_timer: 0,
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

    fn rcs_config<'a>(&self, config: &'a Config) -> RcsConfig {
        let mut rcs_config = if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.rcs.enable_override
        {
            weapon_config.rcs.clone()
        } else {
            config.aim.global.rcs.clone()
        };
        rcs_config.aimbot_hotkey = config.aim.aimbot_hotkey;
        rcs_config
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

    // misc
    pub fn is_button_down(&self, button: &KeyCode) -> bool {
        // what the actual fuck is happening here?
        let value = self.process.read::<u32>(
            self.offsets.interface.input
                + (((button.u64() >> 5) * 4) + self.offsets.direct.button_state),
        );
        ((value >> (button.u64() & 31)) & 1) != 0
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
