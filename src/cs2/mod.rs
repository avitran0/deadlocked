use std::{collections::HashMap, fs::File};

use bones::Bones;
use constants::Constants;
use glam::{IVec4, Mat4, Vec2, Vec3, Vec4};
use log::{debug, info, warn};
use rand::{rng, Rng};
use strum::IntoEnumIterator;

use crate::{
    aimbot::Aimbot,
    config::AimbotConfig,
    cs2::{offsets::Offsets, player::Target, weapon_class::WeaponClass},
    key_codes::KeyCode,
    math::{angles_from_vector, angles_to_direction, angles_to_fov, jitter, vec2_clamp},
    message::PlayerInfo,
    mouse::{mouse_left_press, mouse_left_release, mouse_move},
    proc::{get_pid, open_process, read_string_vec, read_vec, validate_pid},
    process::Process,
};

mod bones;
mod constants;
pub mod offsets;
mod player;
mod weapon_class;

#[derive(Debug)]
pub struct CS2 {
    is_valid: bool,
    process: Option<Process>,
    offsets: Offsets,
    target: Target,

    previous_aim_punch: Vec2,
    unaccounted_aim_punch: Vec2,
    last_shot_ms: u32,
    player_info: Vec<PlayerInfo>,
}

impl Aimbot for CS2 {
    fn is_valid(&self) -> bool {
        if let Some(process) = &self.process {
            return self.is_valid && validate_pid(process.pid);
        }
        false
    }

    fn setup(&mut self) {
        let pid = match get_pid(Constants::PROCESS_NAME) {
            Some(pid) => pid,
            None => {
                self.is_valid = false;
                return;
            }
        };

        let process = match open_process(pid) {
            Some(process) => process,
            None => {
                self.is_valid = false;
                return;
            }
        };
        info!("game started, pid: {}", pid);

        self.offsets = match self.find_offsets(&process) {
            Some(offsets) => offsets,
            None => {
                self.is_valid = false;
                return;
            }
        };
        info!("offsets found");

        self.process = Some(process);
        self.is_valid = true;
    }

    fn run(&mut self, config: &AimbotConfig, mouse: &mut File) -> Vec<PlayerInfo> {
        self.rcs(config, mouse);
        self.aimbot(config, mouse);
        self.player_info.clone()
    }

    fn game_info(&self) -> (Mat4, Vec4) {
        let process = match &self.process {
            Some(process) => process,
            None => return (Mat4::ZERO, Vec4::ZERO),
        };

        let view_matrix = process.read::<Mat4>(self.offsets.direct.view_matrix);
        //std::mem::swap(&mut view_matrix.x_axis, &mut view_matrix.z_axis);
        let window = process.read::<u64>(self.offsets.direct.sdl_window);
        if window == 0 {
            return (Mat4::ZERO, Vec4::ZERO);
        }
        (view_matrix, process.read::<IVec4>(window + 0x18).as_vec4())
    }
}

impl CS2 {
    pub fn new() -> Self {
        Self {
            is_valid: false,
            process: None,
            offsets: Offsets::default(),
            target: Target::default(),

            previous_aim_punch: Vec2::ZERO,
            unaccounted_aim_punch: Vec2::ZERO,
            last_shot_ms: 0,
            player_info: Vec::with_capacity(64),
        }
    }

    fn rcs(&mut self, config: &AimbotConfig, mouse: &mut File) {
        let process = match &self.process {
            Some(process) => process,
            None => {
                self.is_valid = false;
                return;
            }
        };
        // todo: does not yet work
        /*if self.is_bomb_planted(process) {
            dbg!(self.get_bomb_site(process));
            dbg!(self.get_bomb_blow_time(process));
        }*/
        if !config.rcs {
            return;
        }

        let local_controller = self.get_local_controller(process);
        let local_pawn = match self.get_pawn(process, local_controller) {
            Some(pawn) => pawn,
            None => {
                self.target.reset();
                return;
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
            return;
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
            return;
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
        mouse_move(mouse, mouse_angle)
    }

    fn aimbot(&mut self, config: &AimbotConfig, mouse: &mut File) {
        let process = match &self.process {
            Some(process) => process,
            None => {
                self.is_valid = false;
                if !self.player_info.is_empty() {
                    self.player_info.clear();
                }
                return;
            }
        };

        let engine_ms = self.engine_ms(process);
        if engine_ms > self.last_shot_ms + 100 {
            mouse_left_release(mouse);
            self.last_shot_ms = 0;
        }

        let local_controller = self.get_local_controller(process);
        let local_pawn = match self.get_pawn(process, local_controller) {
            Some(pawn) => pawn,
            None => {
                self.target.reset();
                if !self.player_info.is_empty() {
                    self.player_info.clear();
                }
                return;
            }
        };

        let team = self.get_team(process, local_pawn);

        let weapon_class = self.get_weapon_class(process, local_pawn);

        let aimbot_active = self.is_button_down(process, &config.hotkey);
        let view_angles = self.get_view_angles(process, local_pawn);
        let ffa = self.is_ffa(process);
        let shots_fired = self.get_shots_fired(process, local_pawn);
        let aim_punch = match (&weapon_class, self.get_aim_punch(process, local_pawn) * 2.0) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length() == 0.0 && shots_fired > 1 => self.previous_aim_punch,
            (_, punch) => punch,
        };

        let mut pawns = Vec::with_capacity(64);
        let mut local_pawn_index = 0;
        self.player_info.clear();
        for i in 0..=64 {
            let controller = match self.get_client_entity(process, i) {
                Some(controller) => controller,
                None => continue,
            };

            let pawn = match self.get_pawn(process, controller) {
                Some(pawn) => pawn,
                None => continue,
            };

            if !self.is_pawn_valid(process, pawn) {
                continue;
            }

            if pawn == local_pawn {
                local_pawn_index = i - 1;
            } else {
                pawns.push(pawn);
            }

            if self.get_team(process, pawn) != team {
                let info = PlayerInfo {
                    health: self.get_health(process, pawn),
                    armor: self.get_armor(process, pawn),
                    position: self.get_position(process, pawn),
                    head: self.get_bone_position(process, pawn, Bones::Head.u64()),
                    bones: self.get_bones(process, pawn),
                };
                self.player_info.push(info);
            }
        }

        if !config.enabled && !config.triggerbot {
            return;
        }

        if team != Constants::TEAM_CT && team != Constants::TEAM_T {
            self.target.reset();
            return;
        }

        if [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
        ]
        .contains(&weapon_class)
        {
            self.target.reset();
            return;
        }

        let mut smallest_fov = 360.0;
        let eye_position = self.get_eye_position(process, local_pawn);
        if !self.is_pawn_valid(process, self.target.pawn) {
            self.target.reset();
        }
        if !aimbot_active || self.target.pawn == 0 {
            for pawn in pawns {
                if !ffa && team == self.get_team(process, pawn) {
                    continue;
                }

                let head_position = self.get_bone_position(process, pawn, Bones::Head.u64());
                let distance = eye_position.distance(head_position);
                let angle = self.get_target_angle(process, local_pawn, head_position, aim_punch);
                let fov = angles_to_fov(&view_angles, &angle);

                // scale should be 5.0 at 0 units, and 1.0 at 500+ units
                if fov > (config.fov * self.distance_scale(distance)) {
                    continue;
                }

                if fov < smallest_fov {
                    smallest_fov = fov;

                    self.target.pawn = pawn;
                    self.target.angle = angle;
                    self.target.distance = distance;
                    self.target.bone_index = Bones::Head.u64();
                }
            }
        }

        if self.target.pawn == 0 {
            return;
        }

        if config.visibility_check {
            let spotted_mask = self.get_spotted_mask(process, self.target.pawn);
            if (spotted_mask & (1 << local_pawn_index)) == 0 {
                return;
            }
        }

        // update target angle
        if self.target.pawn != 0 && config.multibone {
            let mut smallest_fov = 360.0;
            for bone in Bones::iter() {
                let bone_position = self.get_bone_position(process, self.target.pawn, bone.u64());
                let distance = eye_position.distance(bone_position);
                let angle = self.get_target_angle(process, local_pawn, bone_position, aim_punch);
                let fov = angles_to_fov(&view_angles, &angle);

                if fov < smallest_fov {
                    smallest_fov = fov;

                    self.target.angle = angle;
                    self.target.distance = distance;
                    self.target.bone_index = bone.u64();
                }
            }
        } else if self.target.pawn != 0 {
            let head_position =
                self.get_bone_position(process, self.target.pawn, Bones::Head.u64());
            let distance = eye_position.distance(head_position);
            let angle = self.get_target_angle(process, local_pawn, head_position, aim_punch);

            self.target.angle = angle;
            self.target.distance = distance;
            self.target.bone_index = Bones::Head.u64();
        }

        if config.triggerbot && self.is_button_down(process, &config.triggerbot_hotkey) {
            const RADIUS: f32 = 8.0;
            let bone_pos =
                self.get_bone_position(process, self.target.pawn, self.target.bone_index);
            let player_pos = self.get_eye_position(process, local_pawn);
            let view_direction = angles_to_direction(&view_angles);
            let l = bone_pos - player_pos;
            let tca = l.dot(view_direction);
            let d2 = l.length_squared() - tca * tca;
            if d2 <= RADIUS * RADIUS {
                let thc = (RADIUS * RADIUS - d2).sqrt();
                let t0 = tca - thc;
                let t1 = tca + thc;
                if (t0 > 0.0 || t1 > 0.0) && self.last_shot_ms == 0 {
                    if config.triggerbot_range.is_empty() {
                        self.last_shot_ms = engine_ms + config.triggerbot_range.start;
                    } else {
                        self.last_shot_ms =
                            engine_ms + rng().random_range(config.triggerbot_range.clone());
                    }
                }
            }
        }
        if engine_ms > self.last_shot_ms && engine_ms < self.last_shot_ms + 10 {
            mouse_left_press(mouse);
        }

        if !aimbot_active || !config.enabled {
            return;
        }

        if angles_to_fov(&view_angles, &self.target.angle)
            > (config.fov * self.distance_scale(self.target.distance))
        {
            return;
        }

        if !self.is_pawn_valid(process, self.target.pawn) {
            return;
        }

        if shots_fired < config.start_bullet {
            return;
        }

        let mut aim_angles = view_angles - self.target.angle;
        if aim_angles.y < -180.0 {
            aim_angles.y += 360.0
        }
        vec2_clamp(&mut aim_angles);

        let sensitivity =
            self.get_sensitivity(process) * self.get_fov_multiplier(process, local_pawn);

        let xy = Vec2::new(
            aim_angles.y / sensitivity * 50.0,
            -aim_angles.x / sensitivity * 50.0,
        );
        let smooth_angles = if !config.aim_lock && config.smooth > 1.0 {
            jitter(&xy, config.smooth)
        } else {
            xy
        };

        mouse_move(mouse, smooth_angles)
    }

    fn get_target_angle(
        &self,
        process: &Process,
        local_pawn: u64,
        position: Vec3,
        aim_punch: Vec2,
    ) -> Vec2 {
        let eye_position = self.get_eye_position(process, local_pawn);
        let forward = (position - eye_position).normalize();

        let mut angles = angles_from_vector(&forward) - aim_punch;
        vec2_clamp(&mut angles);

        angles
    }

    fn find_offsets(&self, process: &Process) -> Option<Offsets> {
        let mut offsets = Offsets::default();

        offsets.library.client = process.module_base_address(Constants::CLIENT_LIB)?;
        offsets.library.engine = process.module_base_address(Constants::ENGINE_LIB)?;
        offsets.library.tier0 = process.module_base_address(Constants::TIER0_LIB)?;
        offsets.library.input = process.module_base_address(Constants::INPUT_LIB)?;
        offsets.library.sdl = process.module_base_address(Constants::SDL_LIB)?;

        let resource_offset =
            process.get_interface_offset(offsets.library.engine, "GameResourceServiceClientV0");
        if resource_offset.is_none() {
            warn!("could not get offset for GameResourceServiceClient");
        }
        offsets.interface.resource = resource_offset?;

        offsets.interface.entity = process.read(offsets.interface.resource + 0x50);
        offsets.interface.player = offsets.interface.entity + 0x10;

        let cvar_address = process.get_interface_offset(offsets.library.tier0, "VEngineCvar0");
        if cvar_address.is_none() {
            warn!("could not get convar interface offset");
        }
        offsets.interface.cvar = cvar_address?;
        let input_address =
            process.get_interface_offset(offsets.library.input, "InputSystemVersion0");
        if input_address.is_none() {
            warn!("could not get input interface offset");
        }
        offsets.interface.input = input_address?;

        let view_matrix = process.scan_pattern(
            &[
                0x48, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00, 0x4C, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00,
                0x48, 0x8D, 0x0D,
            ],
            "xxx????xxx????xxx".as_bytes(),
            offsets.library.client,
        );
        if view_matrix.is_none() {
            warn!("could not find view matrix offset");
        }
        offsets.direct.view_matrix = process.get_relative_address(view_matrix? + 0x07, 0x03, 0x07);

        let local_player = process.scan_pattern(
            &[
                0x48, 0x83, 0x3D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x95, 0xC0, 0xC3,
            ],
            "xxx????xxxxx".as_bytes(),
            offsets.library.client,
        );
        if local_player.is_none() {
            warn!("could not find local player offset");
        }
        offsets.direct.local_player = process.get_relative_address(local_player?, 0x03, 0x08);
        offsets.direct.button_state = process
            .read::<u32>(process.get_interface_function(offsets.interface.input, 19) + 0x14)
            as u64;

        let planted_c4 = process.scan_pattern(
            &[0x00, 0x00, 0x00, 0x00, 0x8B, 0x10, 0x85, 0xD2, 0x0F, 0x8F],
            "????xxxxxx".as_bytes(),
            offsets.library.client,
        );
        if planted_c4.is_none() {
            warn!("could not find planted c4 offset");
        }
        offsets.direct.planted_c4 = process.get_relative_address(planted_c4?, 0x00, 0x07);

        let sdl_window = process.get_module_export(offsets.library.sdl, "SDL_GetKeyboardFocus");
        if sdl_window.is_none() {
            warn!("could not find sdl window offset");
        }
        let sdl_window = process.get_relative_address(sdl_window?, 0x02, 0x06);
        let sdl_window = process.read::<u64>(sdl_window);
        offsets.direct.sdl_window = process.get_relative_address(sdl_window, 0x03, 0x07);

        let ffa_address = process.get_convar(&offsets.interface, "mp_teammates_are_enemies");
        if ffa_address.is_none() {
            warn!("could not get mp_tammates_are_enemies convar offset");
        }
        offsets.convar.ffa = ffa_address?;
        let sensitivity_address = process.get_convar(&offsets.interface, "sensitivity");
        if sensitivity_address.is_none() {
            warn!("could not get sensitivity convar offset");
        }
        offsets.convar.sensitivity = sensitivity_address?;

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
                "m_sSanitizedPlayerName" => {
                    if !network_enable || offsets.controller.name != 0 {
                        continue;
                    }
                    offsets.controller.name = read_vec::<u32>(&client_dump, i + 0x08 + 0x10) as u64;
                }
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
                "m_ArmorValue" => {
                    if !network_enable || offsets.pawn.armor != 0 {
                        continue;
                    }
                    offsets.pawn.armor = read_vec::<u32>(&client_dump, i + 0x08 + 0x10) as u64;
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
                "m_bBombTicking" => {
                    if offsets.bomb.is_ticking != 0 {
                        continue;
                    }
                    offsets.bomb.is_ticking = read_vec::<u32>(&client_dump, i + 0x10) as u64;
                }
                "m_nBombSite" => {
                    if !network_enable || offsets.bomb.bomb_site != 0 {
                        continue;
                    }
                    offsets.bomb.bomb_site = read_vec::<u32>(&client_dump, i + 0x08 + 0x10) as u64;
                }
                "m_flC4Blow" => {
                    if offsets.bomb.blow_time != 0 {
                        continue;
                    }
                    offsets.bomb.blow_time = read_vec::<u32>(&client_dump, i + 0x10) as u64;
                }
                _ => {}
            }

            if offsets.all_found() {
                debug!("offsets: {:?}", offsets);
                return Some(offsets);
            }
        }

        warn!("not all offsets found: {:?}", offsets);
        None
    }

    fn get_local_controller(&self, process: &Process) -> u64 {
        process.read(self.offsets.direct.local_player)
    }

    fn get_client_entity(&self, process: &Process, index: u64) -> Option<u64> {
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

    fn get_pawn(&self, process: &Process, controller: u64) -> Option<u64> {
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

    fn get_health(&self, process: &Process, pawn: u64) -> i32 {
        let health = process.read(pawn + self.offsets.pawn.health);
        if !(0..=100).contains(&health) {
            return 0;
        }
        health
    }

    #[allow(unused)]
    fn get_armor(&self, process: &Process, pawn: u64) -> i32 {
        process.read(pawn + self.offsets.pawn.armor)
    }

    fn get_team(&self, process: &Process, pawn: u64) -> u8 {
        process.read(pawn + self.offsets.pawn.team)
    }

    fn get_life_state(&self, process: &Process, pawn: u64) -> u8 {
        process.read(pawn + self.offsets.pawn.life_state)
    }

    fn get_weapon_name(&self, process: &Process, pawn: u64) -> String {
        // CEntityInstance
        let weapon_entity_instance = process.read::<u64>(pawn + self.offsets.pawn.weapon);
        if weapon_entity_instance == 0 {
            return String::from(Constants::WEAPON_UNKNOWN);
        }
        // CEntityIdentity, 0x10 = m_pEntity
        let weapon_entity_identity = process.read::<u64>(weapon_entity_instance + 0x10);
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

    fn get_weapon_class(&self, process: &Process, pawn: u64) -> WeaponClass {
        WeaponClass::from_string(&self.get_weapon_name(process, pawn))
    }

    fn get_gs_node(&self, process: &Process, pawn: u64) -> u64 {
        process.read(pawn + self.offsets.pawn.game_scene_node)
    }

    fn is_dormant(&self, process: &Process, pawn: u64) -> bool {
        let gs_node = self.get_gs_node(process, pawn);
        process.read::<u8>(gs_node + self.offsets.game_scene_node.dormant) != 0
    }

    fn get_position(&self, process: &Process, pawn: u64) -> Vec3 {
        let gs_node = self.get_gs_node(process, pawn);
        process.read(gs_node + self.offsets.game_scene_node.origin)
    }

    fn get_eye_position(&self, process: &Process, pawn: u64) -> Vec3 {
        let position = self.get_position(process, pawn);
        let eye_offset = process.read::<Vec3>(pawn + self.offsets.pawn.eye_offset);

        position + eye_offset
    }

    fn get_bone_position(&self, process: &Process, pawn: u64, bone_index: u64) -> Vec3 {
        let gs_node = self.get_gs_node(process, pawn);
        let bone_data =
            process.read::<u64>(gs_node + self.offsets.game_scene_node.model_state + 0x80);

        if bone_data == 0 {
            return Vec3::ZERO;
        }

        process.read(bone_data + (bone_index * 32))
    }

    fn get_bones(&self, process: &Process, pawn: u64) -> Vec<(Vec3, Vec3)> {
        let mut bones = HashMap::new();

        for bone in Bones::iter() {
            let position = self.get_bone_position(process, pawn, bone.u64());
            bones.insert(bone.u64(), position);
        }

        let mut connections = Vec::with_capacity(Bones::CONNECTIONS.len());

        for connection in Bones::CONNECTIONS {
            connections.push((
                *bones.get(&connection.0.u64()).unwrap(),
                *bones.get(&connection.1.u64()).unwrap(),
            ));
        }

        connections
    }

    fn get_shots_fired(&self, process: &Process, pawn: u64) -> i32 {
        process.read(pawn + self.offsets.pawn.shots_fired)
    }

    fn get_fov_multiplier(&self, process: &Process, pawn: u64) -> f32 {
        process.read(pawn + self.offsets.pawn.fov_multiplier)
    }

    fn get_spotted_mask(&self, process: &Process, pawn: u64) -> i64 {
        process.read(pawn + self.offsets.pawn.spotted_state + self.offsets.spotted_state.mask)
    }

    fn is_pawn_valid(&self, process: &Process, pawn: u64) -> bool {
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

    fn get_view_angles(&self, process: &Process, pawn: u64) -> Vec2 {
        process.read(pawn + self.offsets.pawn.view_angles)
    }

    fn get_aim_punch(&self, process: &Process, pawn: u64) -> Vec2 {
        let length = process.read::<u64>(pawn + self.offsets.pawn.aim_punch_cache);
        if length < 1 {
            return Vec2::ZERO;
        }

        let data_address = process.read::<u64>(pawn + self.offsets.pawn.aim_punch_cache + 0x08);

        process.read(data_address + (length - 1) * 12)
    }

    #[allow(unused)]
    fn is_bomb_planted(&self, process: &Process) -> bool {
        process.read::<u8>(self.offsets.direct.planted_c4 + self.offsets.bomb.is_ticking) != 0
    }

    #[allow(unused)]
    fn get_bomb_site(&self, process: &Process) -> i32 {
        process.read(self.offsets.direct.planted_c4 + self.offsets.bomb.bomb_site)
    }

    #[allow(unused)]
    fn get_bomb_blow_time(&self, process: &Process) -> u32 {
        process.read(self.offsets.direct.planted_c4 + self.offsets.bomb.blow_time)
    }

    // convars
    fn get_sensitivity(&self, process: &Process) -> f32 {
        process.read(self.offsets.convar.sensitivity + 0x40)
    }

    fn is_ffa(&self, process: &Process) -> bool {
        process.read::<u32>(self.offsets.convar.ffa + 0x40) == 1
    }

    // misc
    fn is_button_down(&self, process: &Process, button: &KeyCode) -> bool {
        if *button == KeyCode::None {
            return true;
        }
        // what the actual fuck is happening here?
        let value = process.read::<u32>(
            self.offsets.interface.input
                + (((button.u64() >> 5) * 4) + self.offsets.direct.button_state),
        );
        ((value >> (button.u64() & 31)) & 1) != 0
    }

    fn engine_ms(&self, process: &Process) -> u32 {
        let offset = process
            .read::<i32>(process.get_interface_function(self.offsets.interface.input, 16) + 2);
        process.read(self.offsets.interface.input + offset as u64)
    }

    fn distance_scale(&self, distance: f32) -> f32 {
        if distance > 500.0 {
            1.0
        } else {
            5.0 - (distance / 125.0)
        }
    }
}
