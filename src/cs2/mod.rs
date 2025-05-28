use glam::{Vec2, Vec3};
use log::{debug, info, warn};
use player::Player;
use rcs::Recoil;

use crate::{
    aimbot::Aimbot,
    config::Config,
    constants::cs2,
    cs2::{offsets::Offsets, target::Target},
    key_codes::KeyCode,
    math::{angles_from_vector, vec2_clamp},
    mouse::Mouse,
    process::Process,
};

mod aimbot;
mod bones;
mod esp;
mod fov_changer;
mod glow;
mod noflash;
mod offsets;
mod player;
mod rcs;
mod target;
mod weapon_class;

#[derive(Debug)]
pub struct CS2 {
    is_valid: bool,
    process: Process,
    offsets: Offsets,
    target: Target,
    players: Vec<Player>,
    recoil: Recoil,
}

impl Aimbot for CS2 {
    fn is_valid(&self) -> bool {
        self.is_valid && self.process.is_valid()
    }

    fn setup(&mut self) {
        let Some(process) = Process::open(cs2::PROCESS_NAME) else {
            self.is_valid = false;
            return;
        };
        info!("process found, pid: {}", process.pid);
        self.process = process;

        self.offsets = match self.find_offsets() {
            Some(offsets) => offsets,
            None => {
                self.process = Process::new(-1);
                self.is_valid = false;
                return;
            }
        };
        info!("offsets found");

        self.is_valid = true;
    }

    fn run(&mut self, config: &Config, mouse: &mut Mouse) {
        if !self.process.is_valid() {
            self.is_valid = false;
            return;
        }

        self.cache_players();

        self.glow(config);
        self.no_flash(config);
        self.fov_changer(config);
        self.esp(config);

        if config.aimbot.rcs {
            self.rcs(mouse);
        }

        self.find_target();

        self.aimbot(config, mouse);
    }
}

impl CS2 {
    pub fn new() -> Self {
        Self {
            is_valid: false,
            process: Process::new(-1),
            offsets: Offsets::default(),
            target: Target::default(),
            players: Vec::with_capacity(64),
            recoil: Recoil::default(),
        }
    }

    fn angle_to_target(&self, local_player: &Player, position: &Vec3, aim_punch: &Vec2) -> Vec2 {
        let eye_position = local_player.eye_position(self);
        let forward = (position - eye_position).normalize();

        let mut angles = angles_from_vector(&forward) - aim_punch;
        vec2_clamp(&mut angles);

        angles
    }

    fn find_offsets(&self) -> Option<Offsets> {
        let mut offsets = Offsets::default();

        offsets.library.client = self.process.module_base_address(cs2::CLIENT_LIB)?;
        offsets.library.engine = self.process.module_base_address(cs2::ENGINE_LIB)?;
        offsets.library.tier0 = self.process.module_base_address(cs2::TIER0_LIB)?;
        offsets.library.input = self.process.module_base_address(cs2::INPUT_LIB)?;
        offsets.library.sdl = self.process.module_base_address(cs2::SDL_LIB)?;

        let Some(resource_offset) = self
            .process
            .get_interface_offset(offsets.library.engine, "GameResourceServiceClientV0")
        else {
            warn!("could not get offset for GameResourceServiceClient");
            return None;
        };
        offsets.interface.resource = resource_offset;

        offsets.interface.entity = self.process.read(offsets.interface.resource + 0x50);
        offsets.interface.player = offsets.interface.entity + 0x10;

        let Some(cvar_address) = self
            .process
            .get_interface_offset(offsets.library.tier0, "VEngineCvar0")
        else {
            warn!("could not get convar interface offset");
            return None;
        };
        offsets.interface.cvar = cvar_address;
        let Some(input_address) = self
            .process
            .get_interface_offset(offsets.library.input, "InputSystemVersion0")
        else {
            warn!("could not get input interface offset");
            return None;
        };
        offsets.interface.input = input_address;

        let Some(local_player) = self.process.scan_pattern(
            &[
                0x48, 0x83, 0x3D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x95, 0xC0, 0xC3,
            ],
            "xxx????xxxxx".as_bytes(),
            offsets.library.client,
        ) else {
            warn!("could not find local player offset");
            return None;
        };
        offsets.direct.local_player = self.process.get_relative_address(local_player, 0x03, 0x08);
        offsets.direct.button_state = self.process.read::<u32>(
            self.process
                .get_interface_function(offsets.interface.input, 19)
                + 0x14,
        ) as u64;

        let is_other_enemy = match self.process.scan_pattern(
            &[
                0x31, 0xc0, 0x48, 0x85, 0xf6, 0x0f, 0x84, 0x00, 0x00, 0x00, 0x00, 0x55, 0x48, 0x89,
                0xe5, 0x41, 0x54, 0x53,
            ],
            "xxxxxxx????xxxxxxx".as_bytes(),
            offsets.library.client,
        ) {
            Some(func) => func,
            None => {
                // if byte was already patched
                let Some(is_other_enemy) = self.process.scan_pattern(
                    &[
                        0x31, 0xc0, 0xC3, 0x85, 0xf6, 0x0f, 0x84, 0x00, 0x00, 0x00, 0x00, 0x55,
                        0x48, 0x89, 0xe5, 0x41, 0x54, 0x53,
                    ],
                    "xxxxxxx????xxxxxxx".as_bytes(),
                    offsets.library.client,
                ) else {
                    warn!("could not get IsOtherEnemy function offset");
                    return None;
                };
                is_other_enemy
            }
        };
        // offset by two bytes, because the test instruction is two bytes after the beginning
        offsets.direct.is_other_enemy = is_other_enemy + 2;

        let Some(planted_c4) = self.process.scan_pattern(
            &[0x00, 0x00, 0x00, 0x00, 0x8B, 0x10, 0x85, 0xD2, 0x0F, 0x8F],
            "????xxxxxx".as_bytes(),
            offsets.library.client,
        ) else {
            warn!("could not find planted c4 offset");
            return None;
        };
        offsets.direct.planted_c4 = planted_c4;

        let Some(ffa_address) = self
            .process
            .get_convar(offsets.interface.cvar, "mp_teammates_are_enemies")
        else {
            warn!("could not get mp_tammates_are_enemies convar offset");
            return None;
        };
        offsets.convar.ffa = ffa_address;
        let Some(sensitivity_address) = self
            .process
            .get_convar(offsets.interface.cvar, "sensitivity")
        else {
            warn!("could not get sensitivity convar offset");
            return None;
        };
        offsets.convar.sensitivity = sensitivity_address;

        let client_module_size = self.process.module_size(offsets.library.client);
        let client_dump = self.process.dump_module(offsets.library.client);

        let base = offsets.library.client;
        for index in (0..=(client_module_size - 8)).rev().step_by(8) {
            let Some((netvar_name, network_enable)) =
                self.netvar_name(&client_dump, index, base, client_module_size)
            else {
                continue;
            };
            self.process_netvar(
                &mut offsets,
                &client_dump,
                netvar_name,
                network_enable,
                index,
            );

            if offsets.all_found() {
                debug!("offsets: {:?}", offsets);
                return Some(offsets);
            }
        }

        warn!("not all offsets found: {:?}", offsets);
        None
    }

    fn netvar_name(
        &self,
        client_dump: &[u8],
        index: u64,
        base: u64,
        size: u64,
    ) -> Option<(String, bool)> {
        let mut ne_pointer = self.process.read_vec::<u64>(client_dump, index);

        if (base..base + size).contains(&ne_pointer) {
            ne_pointer = self.process.read_vec(client_dump, ne_pointer - base);
        }

        let network_enable = if (base..base + size).contains(&ne_pointer) {
            let name = self.process.read_string_vec(client_dump, ne_pointer - base);
            name.to_lowercase() == "MNetworkEnable".to_lowercase()
        } else {
            false
        };

        let name_pointer = self.process.read_vec::<u64>(
            client_dump,
            index + if network_enable { 0x08 } else { 0x00 },
        );

        if !(base..base + size).contains(&name_pointer) {
            return None;
        }

        Some((
            self.process
                .read_string_vec(client_dump, name_pointer - base),
            network_enable,
        ))
    }

    fn process_netvar(
        &self,
        offsets: &mut Offsets,
        client_dump: &[u8],
        netvar_name: String,
        network_enable: bool,
        index: u64,
    ) {
        match netvar_name.as_str() {
            "m_sSanitizedPlayerName" => {
                if !network_enable || offsets.controller.name != 0 {
                    return;
                }
                offsets.controller.name =
                    self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "m_hPawn" => {
                if !network_enable || offsets.controller.pawn != 0 {
                    return;
                }
                offsets.controller.pawn =
                    self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "m_iDesiredFOV" => {
                if offsets.controller.desired_fov != 0 {
                    return;
                }
                offsets.controller.desired_fov =
                    self.process.read_vec::<u32>(client_dump, index + 0x8) as u64;
            }
            "m_iHealth" => {
                if !network_enable || offsets.pawn.health != 0 {
                    return;
                }
                offsets.pawn.health =
                    self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "m_ArmorValue" => {
                if !network_enable || offsets.pawn.armor != 0 {
                    return;
                }
                offsets.pawn.armor = self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "m_iTeamNum" => {
                if !network_enable || offsets.pawn.team != 0 {
                    return;
                }
                offsets.pawn.team = self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "m_lifeState" => {
                if !network_enable || offsets.pawn.life_state != 0 {
                    return;
                }
                offsets.pawn.life_state =
                    self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "m_pClippingWeapon" => {
                if offsets.pawn.weapon != 0 {
                    return;
                }
                offsets.pawn.weapon =
                    self.process.read_vec::<u32>(client_dump, index + 0x10) as u64;
            }
            "m_flFOVSensitivityAdjust" => {
                if offsets.pawn.fov_multiplier != 0 {
                    return;
                }
                offsets.pawn.fov_multiplier =
                    self.process.read_vec::<u32>(client_dump, index + 0x08) as u64;
            }
            "m_pGameSceneNode" => {
                if offsets.pawn.game_scene_node != 0 {
                    return;
                }
                offsets.pawn.game_scene_node =
                    self.process.read_vec::<u32>(client_dump, index + 0x10) as u64;
            }
            "m_vecViewOffset" => {
                if !network_enable || offsets.pawn.eye_offset != 0 {
                    return;
                }
                offsets.pawn.eye_offset =
                    self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "m_vecAbsVelocity" => {
                if offsets.pawn.velocity != 0 {
                    return;
                }
                offsets.pawn.velocity =
                    self.process.read_vec::<u32>(client_dump, index + 0x08) as u64;
            }
            "m_aimPunchCache" => {
                if !network_enable || offsets.pawn.aim_punch_cache != 0 {
                    return;
                }
                offsets.pawn.aim_punch_cache =
                    self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "m_iShotsFired" => {
                if !network_enable || offsets.pawn.shots_fired != 0 {
                    return;
                }
                offsets.pawn.shots_fired =
                    self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "v_angle" => {
                if offsets.pawn.view_angles != 0 {
                    return;
                }
                offsets.pawn.view_angles =
                    self.process.read_vec::<u32>(client_dump, index + 0x08) as u64;
            }
            "m_entitySpottedState" => {
                if !network_enable || offsets.pawn.spotted_state != 0 {
                    return;
                }
                let offset = self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
                if !(10000..=14000).contains(&offset) {
                    return;
                }
                offsets.pawn.spotted_state = offset;
            }
            "m_Glow" => {
                if !network_enable || offsets.pawn.glow != 0 {
                    return;
                }
                offsets.pawn.glow = self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "m_flFlashMaxAlpha" => {
                if offsets.pawn.flash_alpha != 0 {
                    return;
                }
                offsets.pawn.flash_alpha =
                    self.process.read_vec::<u32>(client_dump, index + 0x10) as u64;
            }
            "m_flFlashDuration" => {
                if offsets.pawn.flash_duration != 0 {
                    return;
                }
                offsets.pawn.flash_duration =
                    self.process.read_vec::<u32>(client_dump, index + 0x10) as u64;
            }
            "m_pCameraServices" => {
                if !network_enable || offsets.pawn.camera_services != 0 {
                    return;
                }
                offsets.pawn.camera_services =
                    self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "m_bDormant" => {
                if offsets.game_scene_node.dormant != 0 {
                    return;
                }
                offsets.game_scene_node.dormant =
                    self.process.read_vec::<u32>(client_dump, index + 0x08) as u64;
            }
            "m_vecAbsOrigin" => {
                if !network_enable || offsets.game_scene_node.origin != 0 {
                    return;
                }
                offsets.game_scene_node.origin =
                    self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "m_modelState" => {
                if offsets.game_scene_node.model_state != 0 {
                    return;
                }
                offsets.game_scene_node.model_state =
                    self.process.read_vec::<u32>(client_dump, index + 0x08) as u64;
            }
            "m_bSpotted" => {
                if offsets.spotted_state.spotted != 0 {
                    return;
                }
                offsets.spotted_state.spotted =
                    self.process.read_vec::<u32>(client_dump, index + 0x10) as u64;
            }
            "m_bSpottedByMask" => {
                if !network_enable || offsets.spotted_state.mask != 0 {
                    return;
                }
                offsets.spotted_state.mask =
                    self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "m_bGlowing" => {
                if offsets.glow.is_glowing != 0 {
                    return;
                }
                offsets.glow.is_glowing =
                    self.process.read_vec::<u32>(client_dump, index + 0x08) as u64;
            }
            "m_iGlowType" => {
                if offsets.glow.glow_type != 0 {
                    return;
                }
                offsets.glow.glow_type =
                    self.process.read_vec::<u32>(client_dump, index + 0x08) as u64;
            }
            "m_glowColorOverride" => {
                if !network_enable || offsets.glow.color_override != 0 {
                    return;
                }
                offsets.glow.color_override =
                    self.process.read_vec::<u32>(client_dump, index + 0x18) as u64;
            }
            "m_iFOV" => {
                if offsets.camera_services.fov != 0 {
                    return;
                }
                offsets.camera_services.fov =
                    self.process.read_vec::<u32>(client_dump, index + 0x08) as u64;
            }
            _ => {}
        }
    }

    // convars
    fn get_sensitivity(&self) -> f32 {
        self.process.read(self.offsets.convar.sensitivity + 0x40)
    }

    fn is_ffa(&self) -> bool {
        self.process.read::<u32>(self.offsets.convar.ffa + 0x40) == 1
    }

    // misc
    fn is_button_down(&self, button: &KeyCode) -> bool {
        if *button == KeyCode::None {
            return true;
        }
        // what the actual fuck is happening here?
        let value = self.process.read::<u32>(
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
