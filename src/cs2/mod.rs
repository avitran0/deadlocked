use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use glam::{IVec2, Mat4, Vec2, Vec3};

use crate::{
    bvh::Bvh,
    config::{AimbotConfig, Config, RcsConfig, TriggerbotConfig, TriggerbotMode},
    constants::cs2::{self, TEAM_CT, TEAM_T, class},
    cs2::{
        bones::Bones,
        entity::{
            Entity, EntityInfo, GrenadeInfo, inferno::Inferno, molotov::Molotov,
            planted_c4::PlantedC4, player::Player, smoke::Smoke, weapon::Weapon,
        },
        esp_toggle::EspToggle,
        offsets::Offsets,
        rcs::Recoil,
        schema::Schema,
        target::Target,
        triggerbot::Triggerbot,
    },
    data::{Data, PlayerData},
    game::Game,
    key_codes::KeyCode,
    math::{angles_from_vector, vec2_clamp},
    os::{mouse::Mouse, process::Process},
};

mod aimbot;
pub mod bones;
pub mod entity;
mod esp_toggle;
#[cfg(feature = "unsafe")]
mod fov_changer;
#[cfg(feature = "unsafe")]
mod no_flash;
mod offsets;
mod rcs;
mod schema;
mod target;
mod triggerbot;

#[derive(Debug)]
pub struct CS2 {
    is_valid: bool,
    process: Process,
    offsets: Offsets,
    bvh: Arc<Mutex<HashMap<String, Bvh>>>,
    target: Target,
    players: Vec<Player>,
    previous_spectators: RefCell<HashMap<u64, u64>>,
    dead_spectators: RefCell<Vec<(String, u64, u64)>>,
    entities: Vec<Entity>,
    recoil: Recoil,
    trigger: Triggerbot,
    wallhack: EspToggle,
    weapon: Weapon,
    planted_c4: Option<PlantedC4>,
    // stealth helpers
    pub aimbot_cooldown: u8,
    pub last_aimbot_move: glam::Vec2,
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

        // self.cache_players();
        self.cache_entities();

        #[cfg(feature = "unsafe")]
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

        #[cfg(feature = "unsafe")]
        {
            self.no_flash(config);
            self.fov_changer(config);
        }
        self.esp_toggle(config);

        self.rcs(config, mouse);
        self.triggerbot(config);

        self.triggerbot_shoot(mouse);

        self.find_target(config);

        if self.is_button_down(&config.aim.hotkey) {
            self.aimbot(config, mouse);
        }
    }

    fn data(&self, config: &Config, data: &mut Data) {
        data.players.clear();
        data.friendlies.clear();
        data.entities.clear();
        data.spectators.clear();
        data.spectator_names.clear();

        let mut current_spectators = HashMap::new();

        let dead_spectators = {
            let mut dead_specs = self.dead_spectators.borrow_mut();
            let specs = dead_specs.clone();
            dead_specs.clear();
            specs
        };

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
        for player in &self.players {
            let player_data = PlayerData {
                steam_id: player.steam_id(self),
                health: player.health(self),
                armor: player.armor(self),
                position: player.position(self),
                head: player.bone_position(self, Bones::Head.u64()),
                name: player.name(self),
                weapon: player.weapon(self),
                bones: player.all_bones(self),
                has_defuser: player.has_defuser(self),
                has_helmet: player.has_helmet(self),
                has_bomb: player.has_bomb(self),
                visible: player.visible(self, &local_player),
                color: player.color(self),
                rotation: player.rotation(self),
            };
            if !self.is_ffa() && player.team(self) == local_team {
                data.friendlies.push(player_data);
            } else {
                data.players.push(player_data);
            }
        }

        let local_player_data = PlayerData {
            steam_id: local_player.steam_id(self),
            health: local_player.health(self),
            armor: local_player.armor(self),
            position: local_player.position(self),
            head: local_player.bone_position(self, Bones::Head.u64()),
            name: local_player.name(self),
            weapon: local_player.weapon(self),
            bones: local_player.all_bones(self),
            has_defuser: local_player.has_defuser(self),
            has_helmet: local_player.has_helmet(self),
            has_bomb: local_player.has_bomb(self),
            visible: true,
            color: local_player.color(self),
            rotation: local_player.rotation(self),
        };
        data.local_player = local_player_data.clone();

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
        data.triggerbot_active = if self.triggerbot_config(config).mode == TriggerbotMode::Toggle {
            self.trigger.active
        } else {
            false
        };
        data.wallhack_active = self.esp_enabled(config);

        for (spectator_name, spectator_id, target_id) in dead_spectators {
            data.spectators.push((spectator_id, target_id));
            data.spectator_names.push((spectator_name, target_id));
            current_spectators.insert(spectator_id, target_id);
        }

        *self.previous_spectators.borrow_mut() = current_spectators;

        data.view_matrix = self.process.read::<Mat4>(self.offsets.direct.view_matrix);

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
    pub fn new(bvh: Arc<Mutex<HashMap<String, Bvh>>>) -> Self {
        Self {
            is_valid: false,
            process: Process::new(-1),
            offsets: Offsets::default(),
            bvh,
            target: Target::default(),
            players: Vec::with_capacity(64),
            previous_spectators: RefCell::new(HashMap::new()),
            dead_spectators: RefCell::new(Vec::new()),
            entities: Vec::with_capacity(128),
            recoil: Recoil::default(),
            trigger: Triggerbot::default(),
            wallhack: EspToggle::default(),
            weapon: Weapon::default(),
            planted_c4: None,
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

    fn find_offsets(&self) -> Option<Offsets> {
        let start = Instant::now();
        let mut offsets = Offsets::default();

        offsets.library.client = self.process.module_base_address(cs2::CLIENT_LIB)?;
        offsets.library.engine = self.process.module_base_address(cs2::ENGINE_LIB)?;
        offsets.library.tier0 = self.process.module_base_address(cs2::TIER0_LIB)?;
        offsets.library.input = self.process.module_base_address(cs2::INPUT_LIB)?;
        offsets.library.sdl = self.process.module_base_address(cs2::SDL_LIB)?;
        offsets.library.schema = self.process.module_base_address(cs2::SCHEMA_LIB)?;

        let Some(resource_offset) = self
            .process
            .get_interface_offset(offsets.library.engine, "GameResourceServiceClientV0")
        else {
            log::warn!("could not get offset for GameResourceServiceClient");
            return None;
        };
        offsets.interface.resource = resource_offset;

        offsets.interface.entity =
            self.process.read::<u64>(offsets.interface.resource + 0x50) + 0x10;

        let Some(cvar_address) = self
            .process
            .get_interface_offset(offsets.library.tier0, "VEngineCvar0")
        else {
            log::warn!("could not get convar interface offset");
            return None;
        };
        offsets.interface.cvar = cvar_address;
        let Some(input_address) = self
            .process
            .get_interface_offset(offsets.library.input, "InputSystemVersion0")
        else {
            log::warn!("could not get input interface offset");
            return None;
        };
        offsets.interface.input = input_address;

        let Some(local_player) = self
            .process
            .scan("48 83 3D ? ? ? ? 00 0F 95 C0 C3", offsets.library.client)
        else {
            log::warn!("could not find local player offset");
            return None;
        };
        offsets.direct.local_player = self.process.get_relative_address(local_player, 0x03, 0x08);
        offsets.direct.button_state = self.process.read::<u32>(
            self.process
                .get_interface_function(offsets.interface.input, 19)
                + 0x14,
        ) as u64;

        let Some(view_matrix) = self
            .process
            .scan("4C 8D 0D ? ? ? ? 4C 89 E6 4C 8D 05", offsets.library.client)
        else {
            log::warn!("could not find view matrix offset");
            return None;
        };
        offsets.direct.view_matrix =
            self.process
                .get_relative_address(view_matrix + 0x0A, 0x03, 0x07);

        let Some(sdl_window) = self
            .process
            .get_module_export(offsets.library.sdl, "SDL_GetKeyboardFocus")
        else {
            log::warn!("could not find sdl window offset");
            return None;
        };
        let sdl_window = self.process.get_relative_address(sdl_window, 0x02, 0x06);
        let sdl_window = self.process.read(sdl_window);
        offsets.direct.sdl_window = self.process.get_relative_address(sdl_window, 0x03, 0x07);

        let Some(planted_c4) = self.process.scan(
            "48 8D 35 ? ? ? ? 66 0F EF C0 C6 05 ? ? ? ? 01 48 8D 3D",
            offsets.library.client,
        ) else {
            log::warn!("could not find planted c4 offset");
            return None;
        };
        offsets.direct.planted_c4 = self.process.get_relative_address(planted_c4, 0x03, 0x0E);

        // xref "lobby_mapveto"
        let Some(global_vars) = self.process.scan(
            "48 8D 05 ? ? ? ? 48 8B 00 8B 50 ? E9",
            offsets.library.client,
        ) else {
            log::warn!("could not find global vars offset");
            return None;
        };
        offsets.direct.global_vars = self.process.get_relative_address(global_vars, 0x03, 0x07);

        let Some(ffa_address) = self
            .process
            .get_convar(offsets.interface.cvar, "mp_teammates_are_enemies")
        else {
            log::warn!("could not get mp_tammates_are_enemies convar offset");
            return None;
        };
        offsets.convar.ffa = ffa_address;
        let Some(sensitivity_address) = self
            .process
            .get_convar(offsets.interface.cvar, "sensitivity")
        else {
            log::warn!("could not get sensitivity convar offset");
            return None;
        };
        offsets.convar.sensitivity = sensitivity_address;

        let schema = Schema::new(&self.process, offsets.library.schema)?;
        let client = schema.get_library(cs2::CLIENT_LIB)?;

        offsets.controller.steam_id = client.get("CBasePlayerController", "m_steamID")?;
        offsets.controller.name = client.get("CBasePlayerController", "m_iszPlayerName")?;
        offsets.controller.pawn = client.get("CBasePlayerController", "m_hPawn")?;
        offsets.controller.desired_fov = client.get("CBasePlayerController", "m_iDesiredFOV")?;
        offsets.controller.owner_entity = client.get("C_BaseEntity", "m_hOwnerEntity")?;
        offsets.controller.color = client.get("CCSPlayerController", "m_iCompTeammateColor")?;

        offsets.pawn.health = client.get("C_BaseEntity", "m_iHealth")?;
        offsets.pawn.armor = client.get("C_CSPlayerPawn", "m_ArmorValue")?;
        offsets.pawn.team = client.get("C_BaseEntity", "m_iTeamNum")?;
        offsets.pawn.life_state = client.get("C_BaseEntity", "m_lifeState")?;
        offsets.pawn.weapon = client.get("C_CSPlayerPawn", "m_pClippingWeapon")?;
        offsets.pawn.fov_multiplier = client.get("C_BasePlayerPawn", "m_flFOVSensitivityAdjust")?;
        offsets.pawn.game_scene_node = client.get("C_BaseEntity", "m_pGameSceneNode")?;
        offsets.pawn.eye_offset = client.get("C_BaseModelEntity", "m_vecViewOffset")?;
        offsets.pawn.eye_angles = client.get("C_CSPlayerPawn", "m_angEyeAngles")?;
        offsets.pawn.velocity = client.get("C_BaseEntity", "m_vecAbsVelocity")?;
        offsets.pawn.aim_punch_cache = client.get("C_CSPlayerPawn", "m_aimPunchCache")?;
        offsets.pawn.shots_fired = client.get("C_CSPlayerPawn", "m_iShotsFired")?;
        offsets.pawn.view_angles = client.get("C_BasePlayerPawn", "v_angle")?;
        offsets.pawn.spotted_state = client.get("C_CSPlayerPawn", "m_entitySpottedState")?;
        offsets.pawn.crosshair_entity = client.get("C_CSPlayerPawn", "m_iIDEntIndex")?;
        offsets.pawn.is_scoped = client.get("C_CSPlayerPawn", "m_bIsScoped")?;
        offsets.pawn.flash_alpha = client.get("C_CSPlayerPawnBase", "m_flFlashMaxAlpha")?;
        offsets.pawn.flash_duration = client.get("C_CSPlayerPawnBase", "m_flFlashDuration")?;
        offsets.pawn.deathmatch_immunity = client.get("C_CSPlayerPawn", "m_bGunGameImmunity")?;

        offsets.pawn.camera_services = client.get("C_BasePlayerPawn", "m_pCameraServices")?;
        offsets.pawn.item_services = client.get("C_BasePlayerPawn", "m_pItemServices")?;
        offsets.pawn.weapon_services = client.get("C_BasePlayerPawn", "m_pWeaponServices")?;
        offsets.pawn.observer_services = client.get("C_BasePlayerPawn", "m_pObserverServices")?;

        offsets.game_scene_node.dormant = client.get("CGameSceneNode", "m_bDormant")?;
        offsets.game_scene_node.origin = client.get("CGameSceneNode", "m_vecAbsOrigin")?;
        offsets.game_scene_node.model_state = client.get("CSkeletonInstance", "m_modelState")?;

        offsets.skeleton.skeleton_instance =
            client.get("CBodyComponentSkeletonInstance", "m_skeletonInstance")?;

        offsets.smoke.did_smoke_effect =
            client.get("C_SmokeGrenadeProjectile", "m_bDidSmokeEffect")?;
        offsets.smoke.smoke_color = client.get("C_SmokeGrenadeProjectile", "m_vSmokeColor")?;

        offsets.molotov.is_incendiary = client.get("C_MolotovProjectile", "m_bIsIncGrenade")?;

        offsets.inferno.is_burning = client.get("C_Inferno", "m_bFireIsBurning")?;
        offsets.inferno.fire_count = client.get("C_Inferno", "m_fireCount")?;
        offsets.inferno.fire_positions = client.get("C_Inferno", "m_firePositions")?;

        offsets.spotted_state.spotted = client.get("EntitySpottedState_t", "m_bSpotted")?;
        offsets.spotted_state.mask = client.get("EntitySpottedState_t", "m_bSpottedByMask")?;

        offsets.camera_services.fov = client.get("CCSPlayerBase_CameraServices", "m_iFOV")?;

        offsets.item_services.has_defuser =
            client.get("CCSPlayer_ItemServices", "m_bHasDefuser")?;
        offsets.item_services.has_helmet = client.get("CCSPlayer_ItemServices", "m_bHasHelmet")?;

        offsets.weapon_services.weapons = client.get("CPlayer_WeaponServices", "m_hMyWeapons")?;

        offsets.observer_services.target =
            client.get("CPlayer_ObserverServices", "m_hObserverTarget")?;

        offsets.weapon.attribute_manager = client.get("C_EconEntity", "m_AttributeManager")?;
        offsets.weapon.item = client.get("C_AttributeContainer", "m_Item")?;
        offsets.weapon.item_definition_index =
            client.get("C_EconItemView", "m_iItemDefinitionIndex")?;

        offsets.planted_c4.is_ticking = client.get("C_PlantedC4", "m_bBombTicking")?;
        offsets.planted_c4.blow_time = client.get("C_PlantedC4", "m_flC4Blow")?;
        offsets.planted_c4.being_defused = client.get("C_PlantedC4", "m_bBeingDefused")?;
        offsets.planted_c4.is_defused = client.get("C_PlantedC4", "m_bBombDefused")?;
        offsets.planted_c4.has_exploded = client.get("C_PlantedC4", "m_bHasExploded")?;
        offsets.planted_c4.defuse_time_left = client.get("C_PlantedC4", "m_flDefuseCountDown")?;

        offsets.entity_identity.size = client.get_class("CEntityIdentity")?.size();

        log::debug!("offsets: {:?} ({:?})", offsets, Instant::now() - start);
        Some(offsets)
    }

    fn entity_has_owner(&self, entity: u64) -> bool {
        self.process
            .read::<i32>(entity + self.offsets.controller.owner_entity)
            != -1
    }

    // convars
    fn get_sensitivity(&self) -> f32 {
        self.process.read(self.offsets.convar.sensitivity + 0x50)
    }

    fn is_ffa(&self) -> bool {
        self.process.read::<u8>(self.offsets.convar.ffa + 0x50) == 1
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
            .read_string(self.process.read(global_vars + 0x190))
    }

    fn distance_scale(&self, distance: f32) -> f32 {
        if distance > 500.0 {
            1.0
        } else {
            5.0 - (distance / 125.0)
        }
    }

    fn cache_entities(&mut self) {
        self.players.clear();
        self.entities.clear();
        let Some(local_player) = Player::local_player(self) else {
            return;
        };
        const NUM_BUCKETS: usize = 64;
        let bucket_pointers = self
            .process
            .read_vec(self.offsets.interface.entity, 0x8 * NUM_BUCKETS);
        for bucket_index in 0..64 {
            let bucket_pointer =
                *bytemuck::from_bytes(&bucket_pointers[bucket_index * 8..(bucket_index + 1) * 8]);
            self.get_entities_in_bucket(bucket_index as u64, bucket_pointer, &local_player);
        }
    }

    fn get_entities_in_bucket(
        &mut self,
        bucket_index: u64,
        bucket_ptr: u64,
        local_player: &Player,
    ) {
        if bucket_ptr == 0 || bucket_ptr >> 48 != 0 {
            return;
        }
        const IDENTITIES_PER_BUCKET: usize = 512;
        let bucket = self.process.read_vec(
            bucket_ptr,
            IDENTITIES_PER_BUCKET * self.offsets.entity_identity.size as usize,
        );
        for index_in_bucket in 0..IDENTITIES_PER_BUCKET {
            let identity_offset = index_in_bucket * self.offsets.entity_identity.size as usize;

            let entity: u64 = *bytemuck::from_bytes(&bucket[identity_offset..identity_offset + 8]);
            if entity == 0 {
                continue;
            }

            let handle_start = identity_offset + 0x10;
            let handle: u32 = *bytemuck::from_bytes(&bucket[handle_start..handle_start + 4]);
            let handle_index = handle & 0x7FFF;
            let entity_index =
                (bucket_index as usize * IDENTITIES_PER_BUCKET + index_in_bucket) as u32;
            if entity_index != handle_index {
                continue;
            }

            let class_info_ptr: u64 =
                *bytemuck::from_bytes(&bucket[identity_offset + 8..identity_offset + 16]);
            let class_info: u64 = self.process.read(class_info_ptr + 0x30);
            let class_name_ptr: u64 = self.process.read(class_info + 0x08);
            let class = self.process.read_string(class_name_ptr);

            match class.as_str() {
                class::PLAYER_CONTROLLER => {
                    let Some(player) = Player::from_controller(entity, self) else {
                        continue;
                    };

                    if !player.is_valid(self) {
                        continue;
                    }

                    if let Some(target) = player.spectator_target(self) {
                        let spectator_id = player.steam_id(self);
                        let target_pawn = target.pawn;
                        let local_pawn = local_player.pawn;

                        if target_pawn == local_pawn {
                            let spectator_name = player.name(self);
                            let local_steam_id = local_player.steam_id(self);
                            self.dead_spectators.borrow_mut().push((
                                spectator_name,
                                spectator_id,
                                local_steam_id,
                            ));
                        }
                    }

                    if player == *local_player {
                        self.target.local_pawn_index = (handle as u64 & 0x7FFF) - 1;
                    } else {
                        self.players.push(player);
                    }
                }
                class::PLANTED_C4 => {
                    let planted_c4 = PlantedC4::new(entity);
                    if planted_c4.is_relevant(self) {
                        self.planted_c4 = Some(planted_c4)
                    }
                }
                class::INFERNO => {
                    self.entities.push(Entity::Inferno(Inferno::new(entity)));
                }
                class::SMOKE => {
                    self.entities.push(Entity::Smoke(Smoke::new(entity)));
                }
                class::MOLOTOV => self.entities.push(Entity::Molotov(Molotov::new(entity))),
                class::FLASHBANG => self.entities.push(Entity::Flashbang(entity)),
                class::HE_GRENADE => self.entities.push(Entity::HeGrenade(entity)),
                class::DECOY => self.entities.push(Entity::Decoy(entity)),
                _ => {
                    // check if weapon
                    let entity_identity: u64 = self.process.read(entity + 0x10);
                    if entity_identity == 0 {
                        continue;
                    }

                    let name_pointer = self.process.read(entity_identity + 0x20);
                    if name_pointer == 0 {
                        continue;
                    }

                    let name = self.process.read_string(name_pointer);

                    if name.starts_with("weapon_") {
                        if self.entity_has_owner(entity) {
                            continue;
                        }

                        let weapon = Weapon::from_handle(entity, self);

                        self.entities.push(Entity::Weapon { weapon, entity })
                    }
                }
            }

            // m_designerName
            /*let name_pointer: u64 =
                *bytemuck::from_bytes(&bucket[identity_offset + 0x20..identity_offset + 0x28]);
            let Some(entity) = self.entity_type(entity, name_pointer) else {
                continue;
            };
            self.entities.push(entity);*/
        }
    }
}
