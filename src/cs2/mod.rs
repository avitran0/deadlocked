use std::fs::File;

use bones::Bones;
use glam::{Vec2, Vec3};
use strum::IntoEnumIterator;

use crate::{
    aimbot::Aimbot,
    config::Config,
    constants::Constants,
    cs2::{offsets::Offsets, target::Target},
    key_codes::KeyCode,
    math::{angles_from_vector, angles_to_fov, vec2_clamp},
    mouse::move_mouse,
    proc::{
        get_module_base_address, get_pid, open_process, read_string_vec, read_vec, validate_pid,
    },
    process_handle::ProcessHandle,
    weapon_class::WeaponClass,
};

mod bones;
pub mod offsets;
mod target;

#[derive(Debug)]
pub struct CS2 {
    is_valid: bool,
    process: Option<ProcessHandle>,
    offsets: Offsets,
    target: Target,
    pid: u64,

    previous_aim_punch: Vec2,
    unaccounted_aim_punch: Vec2,
}

impl Aimbot for CS2 {
    fn is_valid(&self) -> bool {
        self.is_valid && validate_pid(self.pid)
    }

    fn setup(&mut self) {
        self.pid = match get_pid(Constants::PROCESS_NAME) {
            Some(pid) => pid,
            None => {
                self.is_valid = false;
                return;
            }
        };

        let process = match open_process(self.pid) {
            Some(process) => process,
            None => {
                self.is_valid = false;
                return;
            }
        };

        self.offsets = match self.find_offsets(&process) {
            Some(offsets) => offsets,
            None => {
                self.is_valid = false;
                return;
            }
        };

        self.process = Some(process);
        self.is_valid = true;
    }

    fn run(&mut self, config: &Config, mouse: &mut File) {
        let aimbot_coords = self.aimbot(config);
        if let Some(coords) = aimbot_coords {
            move_mouse(mouse, coords);
        }
        if aimbot_coords.is_none() {
            let rcs_coords = self.rcs(config);
            if let Some(coords) = rcs_coords {
                move_mouse(mouse, coords);
            }
        }
    }
}
impl CS2 {
    pub fn new() -> Self {
        Self {
            is_valid: false,
            process: None,
            offsets: Offsets::default(),
            target: Target::default(),
            pid: 0,

            previous_aim_punch: Vec2::ZERO,
            unaccounted_aim_punch: Vec2::ZERO,
        }
    }

    fn rcs(&mut self, config: &Config) -> Option<Vec2> {
        let process = match &self.process {
            Some(process) => process,
            None => {
                self.is_valid = false;
                return None;
            }
        };
        let config = config.games.get(&config.current_game).unwrap().clone();
        if !config.rcs {
            return None;
        }

        let local_controller = self.get_local_controller(process);
        let local_pawn = match self.get_pawn(process, local_controller) {
            Some(pawn) => pawn,
            None => {
                self.target = Target::default();
                return None;
            }
        };

        let weapon_class = self.get_weapon_class(process, local_pawn);
        if [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
        ]
        .contains(&weapon_class)
        {
            return None;
        }

        let shots_fired = self.get_shots_fired(process, local_pawn);
        let aim_punch = match (weapon_class, self.get_aim_punch(process, local_pawn)) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length() == 0.0 && shots_fired > 1 => self.previous_aim_punch,
            (_, punch) => punch,
        };

        if shots_fired <= 1 {
            self.previous_aim_punch = aim_punch;
            self.unaccounted_aim_punch = Vec2::ZERO;
            return None;
        }
        let sensitivity =
            self.get_sensitivity(process) * self.get_fov_multiplier(process, local_pawn);
        let xy = (aim_punch - self.previous_aim_punch) * -1.0;

        let mouse_angle = Vec2::new(
            ((xy.y * 2.0) / sensitivity) / -0.022,
            ((xy.x * 2.0) / sensitivity) / 0.022,
        ) + self.unaccounted_aim_punch;
        self.unaccounted_aim_punch = Vec2::ZERO;

        // only if the aimbot is not active
        self.previous_aim_punch = aim_punch;
        if (0.0..1.0).contains(&mouse_angle.x) {
            self.unaccounted_aim_punch.x = mouse_angle.x;
        }
        if (0.0..1.0).contains(&mouse_angle.y) {
            self.unaccounted_aim_punch.y = mouse_angle.y;
        }
        Some(mouse_angle)
    }

    fn aimbot(&mut self, config: &Config) -> Option<Vec2> {
        let process = match &self.process {
            Some(process) => process,
            None => {
                self.is_valid = false;
                return None;
            }
        };
        let config = config.games.get(&config.current_game).unwrap().clone();

        let local_controller = self.get_local_controller(process);
        let local_pawn = match self.get_pawn(process, local_controller) {
            Some(pawn) => pawn,
            None => {
                self.target = Target::default();
                return None;
            }
        };

        let team = self.get_team(process, local_pawn);
        if team != Constants::TEAM_CT && team != Constants::TEAM_T {
            self.target = Target::default();
            return None;
        }

        let weapon_class = self.get_weapon_class(process, local_pawn);
        if [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
        ]
        .contains(&weapon_class)
        {
            self.target = Target::default();
            return None;
        }

        let aimbot_active = self.is_button_down(process, &config.hotkey);
        let view_angles = self.get_view_angles(process, local_pawn);
        let ffa = self.is_ffa(process);
        let shots_fired = self.get_shots_fired(process, local_pawn);
        let aim_punch = match (weapon_class, self.get_aim_punch(process, local_pawn)) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length() == 0.0 && shots_fired > 1 => self.previous_aim_punch,
            (_, punch) => punch,
        };

        let mut pawns = Vec::with_capacity(64);
        let mut local_pawn_index = 0;
        for i in 1..=64 {
            let controller = match self.get_client_entity(process, i) {
                Some(controller) => controller,
                None => continue,
            };

            let pawn = match self.get_pawn(process, controller) {
                Some(pawn) => pawn,
                None => continue,
            };

            if pawn == local_pawn {
                local_pawn_index = i - 1;
            }
            pawns.push(pawn);
        }

        let mut highest_priority = 0.0;
        let eye_position = self.get_eye_position(process, local_pawn);
        if !self.is_pawn_valid(process, self.target.pawn) {
            self.target = Target::default();
        }
        if !aimbot_active || self.target.pawn == 0 {
            for pawn in pawns {
                if !self.is_pawn_valid(process, pawn) {
                    continue;
                }

                if !ffa && team == self.get_team(process, pawn) {
                    continue;
                }

                let head_position = self.get_bone_position(process, pawn, Bones::Head.u64());
                let distance = eye_position.distance(head_position);
                let angle = self.get_target_angle(process, local_pawn, head_position, aim_punch);
                let fov = angles_to_fov(view_angles, angle);

                if fov > config.fov {
                    continue;
                }

                let priority = 1.0 / (fov + distance);
                if priority > highest_priority {
                    highest_priority = priority;

                    self.target.pawn = pawn;
                    self.target.angle = angle;
                    self.target.bone_index = Bones::Head.u64();
                }
            }
        }

        if highest_priority > config.fov && self.target.pawn == 0 {
            return None;
        }

        if config.visibility_check {
            let spotted_mask = self.get_spotted_mask(process, self.target.pawn);
            if (spotted_mask & (1 << local_pawn_index)) == 0 {
                return None;
            }
        }

        // update target angle
        if self.target.pawn != 0 && config.multibone && !config.aim_lock {
            let mut smallest_fov = 360.0;
            for bone in Bones::iter() {
                let bone_position = self.get_bone_position(process, self.target.pawn, bone.u64());
                let angle = self.get_target_angle(process, local_pawn, bone_position, aim_punch);
                let fov = angles_to_fov(view_angles, angle);

                if fov < smallest_fov {
                    smallest_fov = fov;

                    self.target.angle = angle;
                    self.target.bone_index = bone.u64();
                }
            }
        } else if self.target.pawn != 0 {
            let head_position =
                self.get_bone_position(process, self.target.pawn, Bones::Head.u64());
            let angle = self.get_target_angle(process, local_pawn, head_position, aim_punch);

            self.target.angle = angle;
            self.target.bone_index = Bones::Head.u64();
        }

        if !aimbot_active {
            return None;
        }

        if angles_to_fov(view_angles, self.target.angle) > config.fov {
            return None;
        }

        if !self.is_pawn_valid(process, self.target.pawn) {
            return None;
        }

        if shots_fired < config.start_bullet {
            return None;
        }

        let mut aim_angles = view_angles - self.target.angle;
        vec2_clamp(&mut aim_angles);

        let sensitivity =
            self.get_sensitivity(process) * self.get_fov_multiplier(process, local_pawn);

        let xy = Vec2::new(
            aim_angles.y / sensitivity * 50.0,
            -aim_angles.x / sensitivity * 50.0,
        );
        let mut smooth_angles = Vec2::ZERO;

        if !config.aim_lock && config.smooth >= 1.0 {
            smooth_angles.x = if xy.x.abs() > 1.0 {
                if smooth_angles.x < xy.x {
                    smooth_angles.x + 1.0 + (xy.x / config.smooth)
                } else {
                    smooth_angles.x - 1.0 + (xy.x / config.smooth)
                }
            } else {
                xy.x
            };

            smooth_angles.y = if xy.y.abs() > 1.0 {
                if smooth_angles.y < xy.y {
                    smooth_angles.y + 1.0 + (xy.y / config.smooth)
                } else {
                    smooth_angles.y - 1.0 + (xy.y / config.smooth)
                }
            } else {
                xy.y
            };
        } else {
            smooth_angles = xy;
        }

        Some(smooth_angles)
    }

    fn get_target_angle(
        &self,
        process: &ProcessHandle,
        local_pawn: u64,
        position: Vec3,
        aim_punch: Vec2,
    ) -> Vec2 {
        let eye_position = self.get_eye_position(process, local_pawn);
        let forward = (position - eye_position).normalize();

        let mut angles = angles_from_vector(forward) - aim_punch;
        vec2_clamp(&mut angles);

        angles
    }

    fn find_offsets(&self, process: &ProcessHandle) -> Option<Offsets> {
        let mut offsets = Offsets::default();

        let client_address = get_module_base_address(process, Constants::CLIENT_LIB);
        offsets.library.client = client_address?;

        let engine_address = get_module_base_address(process, Constants::ENGINE_LIB);
        offsets.library.engine = engine_address?;

        let tier0_address = get_module_base_address(process, Constants::TIER0_LIB);
        offsets.library.tier0 = tier0_address?;

        let input_address = get_module_base_address(process, Constants::INPUT_LIB);
        offsets.library.input = input_address?;

        let sdl_address = get_module_base_address(process, Constants::SDL_LIB);
        offsets.library.sdl = sdl_address?;

        let resource_offset =
            process.get_interface_offset(offsets.library.engine, "GameResourceServiceClientV0");
        offsets.interface.resource = resource_offset?;

        offsets.interface.entity = process.read(offsets.interface.resource + 0x50);
        offsets.interface.player = offsets.interface.entity + 0x10;

        offsets.interface.cvar =
            process.get_interface_offset(offsets.library.tier0, "VEngineCvar0")?;
        offsets.interface.input =
            process.get_interface_offset(offsets.library.input, "InputSystemVersion0")?;

        // seems to be in .text section (executable instructions)
        let local_player = process.scan_pattern(
            &[
                0x48, 0x83, 0x3D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x95, 0xC0, 0xC3,
            ],
            "xxx????xxxxx".as_bytes(),
            offsets.library.client,
        );
        offsets.direct.local_player = process.get_relative_address(local_player?, 0x03, 0x08);
        offsets.direct.button_state = process
            .read::<u32>(process.get_interface_function(offsets.interface.input, 19) + 0x14)
            as u64;

        offsets.convar.ffa = process.get_convar(&offsets.interface, "mp_teammates_are_enemies")?;
        offsets.convar.sensitivity = process.get_convar(&offsets.interface, "sensitivity")?;

        let client_module_size = process.module_size(offsets.library.client);
        let client_dump = process.dump_module(offsets.library.client);

        let base = offsets.library.client;
        for i in (0..=(client_module_size - 8)).rev().step_by(8) {
            let mut network_enable = false;

            let mut name_pointer = read_vec::<u64>(&client_dump, i);
            if name_pointer >= base && name_pointer <= base + client_module_size {
                name_pointer = read_vec(&client_dump, name_pointer - base);
                if name_pointer >= base && name_pointer <= base + client_module_size {
                    let name = read_string_vec(&client_dump, name_pointer - base);
                    if name.to_lowercase() == "MNetworkEnable".to_lowercase() {
                        network_enable = true;
                    }
                }
            }

            let name_ptr = match network_enable {
                true => read_vec::<u64>(&client_dump, i + 0x08),
                false => read_vec::<u64>(&client_dump, i),
            };

            if name_ptr < base || name_ptr > base + client_module_size {
                continue;
            }

            let netvar_name = read_string_vec(&client_dump, name_ptr - base);

            match netvar_name.as_str() {
                "m_hPawn" => {
                    if !network_enable || offsets.controller.pawn != 0 {
                        continue;
                    }
                    offsets.controller.pawn = read_vec::<u32>(&client_dump, i + 0x08 + 0x10) as u64;
                }
                "m_iHealth" => {
                    if !network_enable || offsets.pawn.health != 0 {
                        continue;
                    }
                    offsets.pawn.health = read_vec::<u32>(&client_dump, i + 0x08 + 0x10) as u64;
                }
                "m_iTeamNum" => {
                    if !network_enable || offsets.pawn.team != 0 {
                        continue;
                    }
                    offsets.pawn.team = read_vec::<u32>(&client_dump, i + 0x08 + 0x10) as u64;
                }
                "m_lifeState" => {
                    if !network_enable || offsets.pawn.life_state != 0 {
                        continue;
                    }
                    offsets.pawn.life_state = read_vec::<u32>(&client_dump, i + 0x08 + 0x10) as u64;
                }
                "m_pClippingWeapon" => {
                    if offsets.pawn.weapon != 0 {
                        continue;
                    }
                    offsets.pawn.weapon = read_vec::<u32>(&client_dump, i + 0x10) as u64;
                }
                "m_flFOVSensitivityAdjust" => {
                    if offsets.pawn.fov_multiplier != 0 {
                        continue;
                    }
                    offsets.pawn.fov_multiplier = read_vec::<u32>(&client_dump, i + 0x08) as u64;
                }
                "m_pGameSceneNode" => {
                    if offsets.pawn.game_scene_node != 0 {
                        continue;
                    }
                    offsets.pawn.game_scene_node = read_vec::<u32>(&client_dump, i + 0x10) as u64;
                }
                "m_vecViewOffset" => {
                    if !network_enable || offsets.pawn.eye_offset != 0 {
                        continue;
                    }
                    offsets.pawn.eye_offset = read_vec::<u32>(&client_dump, i + 0x08 + 0x10) as u64;
                }
                "m_vecVelocity" => {
                    if offsets.pawn.velocity != 0 {
                        continue;
                    }
                    let value = read_vec::<u32>(&client_dump, i + 0x08) as u64;
                    if !(800..=1600).contains(&value) {
                        continue;
                    }
                    offsets.pawn.velocity = value;
                }
                "m_aimPunchCache" => {
                    if !network_enable || offsets.pawn.aim_punch_cache != 0 {
                        continue;
                    }
                    offsets.pawn.aim_punch_cache =
                        read_vec::<u32>(&client_dump, i + 0x08 + 0x10) as u64;
                }
                "m_iShotsFired" => {
                    if !network_enable || offsets.pawn.shots_fired != 0 {
                        continue;
                    }
                    offsets.pawn.shots_fired =
                        read_vec::<u32>(&client_dump, i + 0x08 + 0x10) as u64;
                }
                "v_angle" => {
                    if offsets.pawn.view_angles != 0 {
                        continue;
                    }
                    offsets.pawn.view_angles = read_vec::<u32>(&client_dump, i + 0x08) as u64;
                }
                "m_entitySpottedState" => {
                    if !network_enable || offsets.pawn.spotted_state != 0 {
                        continue;
                    }
                    let offset = read_vec::<u32>(&client_dump, i + 0x08 + 0x10) as u64;
                    if !(10000..=14000).contains(&offset) {
                        continue;
                    }
                    offsets.pawn.spotted_state = offset;
                }
                "m_pObserverServices" => {
                    if offsets.pawn.observer_services != 0 {
                        continue;
                    }
                    offsets.pawn.observer_services = read_vec::<u32>(&client_dump, i + 0x08) as u64;
                }
                "m_bDormant" => {
                    if offsets.game_scene_node.dormant != 0 {
                        continue;
                    }
                    offsets.game_scene_node.dormant =
                        read_vec::<u32>(&client_dump, i + 0x08) as u64;
                }
                "m_vecAbsOrigin" => {
                    if !network_enable || offsets.game_scene_node.origin != 0 {
                        continue;
                    }
                    offsets.game_scene_node.origin =
                        read_vec::<u32>(&client_dump, i + 0x08 + 0x10) as u64;
                }
                "m_modelState" => {
                    if offsets.game_scene_node.model_state != 0 {
                        continue;
                    }
                    offsets.game_scene_node.model_state =
                        read_vec::<u32>(&client_dump, i + 0x08) as u64;
                }
                "m_bSpotted" => {
                    if offsets.spotted_state.spotted != 0 {
                        continue;
                    }
                    offsets.spotted_state.spotted = read_vec::<u32>(&client_dump, i + 0x10) as u64;
                }
                "m_bSpottedByMask" => {
                    if !network_enable || offsets.spotted_state.mask != 0 {
                        continue;
                    }
                    offsets.spotted_state.mask =
                        read_vec::<u32>(&client_dump, i + 0x08 + 0x10) as u64;
                }
                "m_hObserverTarget" => {
                    if offsets.observer_service.target != 0 {
                        continue;
                    }
                    offsets.observer_service.target =
                        read_vec::<u32>(&client_dump, i + 0x08) as u64;
                }
                _ => {}
            }

            if offsets.all_found() {
                return Some(offsets);
            }
        }
        None
    }

    fn get_local_controller(&self, process: &ProcessHandle) -> u64 {
        process.read(self.offsets.direct.local_player)
    }

    fn get_client_entity(&self, process: &ProcessHandle, index: u64) -> Option<u64> {
        // wtf is this doing, and how?
        let v1 = process.read::<u64>(self.offsets.interface.entity + 0x08 * (index >> 9) + 0x10);
        if v1 == 0 {
            return None;
        }
        // what?
        let entity = process.read(v1 + 120 * (index & 0x1ff));
        if entity == 0 {
            return None;
        }
        Some(entity)
    }

    fn get_pawn(&self, process: &ProcessHandle, controller: u64) -> Option<u64> {
        let v1 = process.read::<i32>(controller + self.offsets.controller.pawn);
        if v1 == -1 {
            return None;
        }

        // what the fuck is this doing?
        let v2 =
            process.read::<u64>(self.offsets.interface.player + 8 * ((v1 as u64 & 0x7fff) >> 9));
        if v2 == 0 {
            return None;
        }

        // bit-fuckery, why is this needed exactly?
        let entity = process.read(v2 + 120 * (v1 as u64 & 0x1ff));
        if entity == 0 {
            return None;
        }
        Some(entity)
    }

    fn get_health(&self, process: &ProcessHandle, pawn: u64) -> i32 {
        let health = process.read(pawn + self.offsets.pawn.health);
        if !(0..=100).contains(&health) {
            return 0;
        }
        health
    }

    fn get_team(&self, process: &ProcessHandle, pawn: u64) -> u8 {
        process.read(pawn + self.offsets.pawn.team)
    }

    fn get_life_state(&self, process: &ProcessHandle, pawn: u64) -> u8 {
        process.read(pawn + self.offsets.pawn.life_state)
    }

    fn get_weapon(&self, process: &ProcessHandle, pawn: u64) -> String {
        // CEntityInstance
        let weapon_entity_instance = process.read(pawn + self.offsets.pawn.weapon);
        if weapon_entity_instance == 0 {
            return String::from(Constants::WEAPON_UNKNOWN);
        }
        self.get_weapon_name(process, weapon_entity_instance)
    }

    fn get_weapon_name(&self, process: &ProcessHandle, weapon_instance: u64) -> String {
        // CEntityIdentity, 0x10 = m_pEntity
        let weapon_entity_identity = process.read::<u64>(weapon_instance + 0x10);
        if weapon_entity_identity == 0 {
            return String::from(Constants::WEAPON_UNKNOWN);
        }
        // 0x20 = m_designerName (pointer -> string)
        let weapon_name_pointer = process.read(weapon_entity_identity + 0x20);
        if weapon_name_pointer == 0 {
            return String::from(Constants::WEAPON_UNKNOWN);
        }
        process.read_string(weapon_name_pointer)
    }

    fn get_weapon_class(&self, process: &ProcessHandle, pawn: u64) -> WeaponClass {
        WeaponClass::from_string(&self.get_weapon(process, pawn))
    }

    fn get_gs_node(&self, process: &ProcessHandle, pawn: u64) -> u64 {
        process.read(pawn + self.offsets.pawn.game_scene_node)
    }

    fn is_dormant(&self, process: &ProcessHandle, pawn: u64) -> bool {
        let gs_node = self.get_gs_node(process, pawn);
        process.read::<u8>(gs_node + self.offsets.game_scene_node.dormant) != 0
    }

    fn get_position(&self, process: &ProcessHandle, pawn: u64) -> Vec3 {
        let gs_node = self.get_gs_node(process, pawn);
        process.read(gs_node + self.offsets.game_scene_node.origin)
    }

    fn get_eye_position(&self, process: &ProcessHandle, pawn: u64) -> Vec3 {
        let position = self.get_position(process, pawn);
        let eye_offset = process.read::<Vec3>(pawn + self.offsets.pawn.eye_offset);

        position + eye_offset
    }

    fn get_bone_position(&self, process: &ProcessHandle, pawn: u64, bone_index: u64) -> Vec3 {
        let gs_node = self.get_gs_node(process, pawn);
        let bone_data =
            process.read::<u64>(gs_node + self.offsets.game_scene_node.model_state + 0x80);

        if bone_data == 0 {
            return Vec3::ZERO;
        }

        process.read(bone_data + (bone_index * 32))
    }

    fn get_shots_fired(&self, process: &ProcessHandle, pawn: u64) -> i32 {
        process.read(pawn + self.offsets.pawn.shots_fired)
    }

    fn get_fov_multiplier(&self, process: &ProcessHandle, pawn: u64) -> f32 {
        process.read(pawn + self.offsets.pawn.fov_multiplier)
    }

    fn get_spotted_mask(&self, process: &ProcessHandle, pawn: u64) -> i32 {
        process.read(pawn + self.offsets.pawn.spotted_state + self.offsets.spotted_state.mask)
    }

    #[allow(unused)]
    fn get_spectator_target(&self, process: &ProcessHandle, pawn: u64) -> Option<u64> {
        let observer_services = process.read::<u64>(pawn + self.offsets.pawn.observer_services);
        if observer_services == 0 {
            return None;
        }

        let target =
            process.read::<u32>(observer_services + self.offsets.observer_service.target) & 0x7fff;
        if target == 0 {
            return None;
        }

        let v2 = process.read::<u64>(self.offsets.interface.player + 8 * (target as u64 >> 9));
        if v2 == 0 {
            return None;
        }

        let entity = process.read(v2 + 120 * (target as u64 & 0x1ff));
        if entity == 0 {
            return None;
        }
        Some(entity)
    }

    fn is_pawn_valid(&self, process: &ProcessHandle, pawn: u64) -> bool {
        if self.is_dormant(process, pawn) {
            return false;
        }

        if self.get_health(process, pawn) <= 0 {
            return false;
        }

        if self.get_life_state(process, pawn) != 0 {
            return false;
        }

        true
    }

    fn get_view_angles(&self, process: &ProcessHandle, pawn: u64) -> Vec2 {
        process.read(pawn + self.offsets.pawn.view_angles)
    }

    fn get_aim_punch(&self, process: &ProcessHandle, pawn: u64) -> Vec2 {
        let length = process.read::<u64>(pawn + self.offsets.pawn.aim_punch_cache);
        if length < 1 {
            return Vec2::ZERO;
        }

        let data_address = process.read::<u64>(pawn + self.offsets.pawn.aim_punch_cache + 0x08);

        process.read(data_address + (length - 1) * 12)
    }

    // convars
    fn get_sensitivity(&self, process: &ProcessHandle) -> f32 {
        process.read(self.offsets.convar.sensitivity + 0x40)
    }

    fn is_ffa(&self, process: &ProcessHandle) -> bool {
        process.read::<u32>(self.offsets.convar.ffa + 0x40) == 1
    }

    // misc
    fn is_button_down(&self, process: &ProcessHandle, button: &KeyCode) -> bool {
        // what the actual fuck is happening here?
        let value = process.read::<u32>(
            self.offsets.interface.input
                + (((button.u64() >> 5) * 4) + self.offsets.direct.button_state),
        );
        ((value >> (button.u64() & 31)) & 1) != 0
    }
}