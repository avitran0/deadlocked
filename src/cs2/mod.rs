use std::{fs::File, time::Instant};

use constants::Constants;
use glam::{Vec2, Vec3};
use log::{debug, info, warn};
use player::Player;

use crate::{
    aimbot::Aimbot,
    config::AimbotConfig,
    cs2::{offsets::Offsets, target::Target},
    key_codes::KeyCode,
    math::{angles_from_vector, vec2_clamp},
    proc::{get_pid, open_process, read_string_vec, read_vec, validate_pid},
    process::Process,
};

mod aimbot;
mod bones;
mod constants;
pub mod offsets;
mod player;
mod rcs;
mod target;
mod triggerbot;
mod weapon_class;

#[derive(Debug)]
pub struct CS2 {
    is_valid: bool,
    process: Option<Process>,
    offsets: Offsets,
    target: Target,

    previous_aim_punch: Vec2,
    unaccounted_aim_punch: Vec2,
    last_shot: Option<Instant>,
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

    fn run(&mut self, config: &AimbotConfig, mouse: &mut File) {
        if self.process.is_none() {
            self.is_valid = false;
            return;
        }

        if config.rcs {
            self.rcs(mouse);
        }

        self.find_target();

        self.aimbot(config, mouse);
        self.triggerbot(config, mouse);
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
            last_shot: None,
        }
    }

    fn get_target_angle(
        &self,
        process: &Process,
        local_player: &Player,
        position: &Vec3,
        aim_punch: &Vec2,
    ) -> Vec2 {
        let eye_position = local_player.eye_position(process, &self.offsets);
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

        // seems to be in .text section (executable instructions)
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

        let glow_manager = process.scan_pattern(
            &[
                0x48, 0x8B, 0x05, 0x00, 0x00, 0x00, 0x00, 0xC3, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x48, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00, 0x48, 0xC7, 0x47,
            ],
            "xxx????xxxxx????xxx????xxx".as_bytes(),
            offsets.library.client,
        );
        if glow_manager.is_none() {
            warn!("could not find glow manager offset");
        }
        offsets.direct.glow_manager = process.get_relative_address(glow_manager?, 0x03, 0x07);

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

    // bomb
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

    fn distance_scale(&self, distance: f32) -> f32 {
        if distance > 500.0 {
            1.0
        } else {
            5.0 - (distance / 125.0)
        }
    }
}
