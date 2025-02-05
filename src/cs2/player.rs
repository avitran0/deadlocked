use glam::{Vec2, Vec3};

use crate::{constants::Constants, process::Process};

use super::{offsets::Offsets, weapon_class::WeaponClass, CS2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Player {
    controller: u64,
    pawn: u64,
}

impl Player {
    pub fn index(process: &Process, offsets: &Offsets, index: u64) -> Option<Self> {
        let controller = Self::get_client_entity(process, offsets, index)?;
        Self::get_pawn(process, offsets, controller).map(|pawn| Self { controller, pawn })
    }

    pub fn local_player(process: &Process, offsets: &Offsets) -> Option<Self> {
        let controller = process.read(offsets.direct.local_player);
        if controller == 0 {
            return None;
        }
        Self::get_pawn(process, offsets, controller).map(|pawn| Self { controller, pawn })
    }

    fn get_client_entity(process: &Process, offsets: &Offsets, index: u64) -> Option<u64> {
        // wtf is this doing, and how?
        let v1 = process.read::<u64>(offsets.interface.entity + 0x08 * (index >> 9) + 0x10);
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

    fn get_pawn(process: &Process, offsets: &Offsets, controller: u64) -> Option<u64> {
        let v1 = process.read::<i32>(controller + offsets.controller.pawn);
        if v1 == -1 {
            return None;
        }

        // what the fuck is this doing?
        let v2 = process.read::<u64>(offsets.interface.player + 8 * ((v1 as u64 & 0x7fff) >> 9));
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

    pub fn health(&self, process: &Process, offsets: &Offsets) -> i32 {
        let health = process.read(self.pawn + offsets.pawn.health);
        if !(0..=100).contains(&health) {
            return 0;
        }
        health
    }

    #[allow(unused)]
    pub fn armor(&self, process: &Process, offsets: &Offsets) -> i32 {
        process.read(self.pawn + offsets.pawn.armor)
    }

    pub fn team(&self, process: &Process, offsets: &Offsets) -> u8 {
        process.read(self.pawn + offsets.pawn.team)
    }

    pub fn life_state(&self, process: &Process, offsets: &Offsets) -> u8 {
        process.read(self.pawn + offsets.pawn.life_state)
    }

    pub fn weapon_name(&self, process: &Process, offsets: &Offsets) -> String {
        // CEntityInstance
        let weapon_entity_instance = process.read::<u64>(self.pawn + offsets.pawn.weapon);
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

    pub fn weapon_class(&self, process: &Process, offsets: &Offsets) -> WeaponClass {
        WeaponClass::from_string(&self.weapon_name(process, offsets))
    }

    fn game_scene_node(&self, process: &Process, offsets: &Offsets) -> u64 {
        process.read(self.pawn + offsets.pawn.game_scene_node)
    }

    fn is_dormant(&self, process: &Process, offsets: &Offsets) -> bool {
        let gs_node = self.game_scene_node(process, offsets);
        process.read::<u8>(gs_node + offsets.game_scene_node.dormant) != 0
    }

    pub fn position(&self, process: &Process, offsets: &Offsets) -> Vec3 {
        let gs_node = self.game_scene_node(process, offsets);
        process.read(gs_node + offsets.game_scene_node.origin)
    }

    pub fn eye_position(&self, process: &Process, offsets: &Offsets) -> Vec3 {
        let position = self.position(process, offsets);
        let eye_offset = process.read::<Vec3>(self.pawn + offsets.pawn.eye_offset);

        position + eye_offset
    }

    pub fn bone_position(&self, process: &Process, offsets: &Offsets, bone_index: u64) -> Vec3 {
        let gs_node = self.game_scene_node(process, offsets);
        let bone_data = process.read::<u64>(gs_node + offsets.game_scene_node.model_state + 0x80);

        if bone_data == 0 {
            return Vec3::ZERO;
        }

        process.read(bone_data + (bone_index * 32))
    }

    pub fn shots_fired(&self, process: &Process, offsets: &Offsets) -> i32 {
        process.read(self.pawn + offsets.pawn.shots_fired)
    }

    pub fn fov_multiplier(&self, process: &Process, offsets: &Offsets) -> f32 {
        process.read(self.pawn + offsets.pawn.fov_multiplier)
    }

    pub fn spotted_mask(&self, process: &Process, offsets: &Offsets) -> i64 {
        process.read(self.pawn + offsets.pawn.spotted_state + offsets.spotted_state.mask)
    }

    pub fn is_valid(&self, process: &Process, offsets: &Offsets) -> bool {
        if self.is_dormant(process, offsets) {
            return false;
        }

        if self.health(process, offsets) <= 0 {
            return false;
        }

        if self.life_state(process, offsets) != 0 {
            return false;
        }

        true
    }

    pub fn is_flashed(&self, process: &Process, offsets: &Offsets) -> bool {
        process.read::<f32>(self.pawn + offsets.pawn.flash_duration) > 0.2
    }

    pub fn view_angles(&self, process: &Process, offsets: &Offsets) -> Vec2 {
        process.read(self.pawn + offsets.pawn.view_angles)
    }

    pub fn aim_punch(&self, process: &Process, offsets: &Offsets) -> Vec2 {
        let length = process.read::<u64>(self.pawn + offsets.pawn.aim_punch_cache);
        if length < 1 {
            return Vec2::ZERO;
        }

        let data_address = process.read::<u64>(self.pawn + offsets.pawn.aim_punch_cache + 0x08);

        process.read(data_address + (length - 1) * 12)
    }

    #[allow(unused)]
    pub fn velocity(&self, process: &Process, offsets: &Offsets) -> Vec3 {
        process.read(self.pawn + offsets.pawn.velocity)
    }

    pub fn glow(&self, process: &Process, offsets: &Offsets, color: u32) {
        process.write(self.pawn + offsets.pawn.glow + offsets.glow.is_glowing, 1u8);
        process.write(self.pawn + offsets.pawn.glow + offsets.glow.glow_type, 3);
        process.write(
            self.pawn + offsets.pawn.glow + offsets.glow.color_override,
            color,
        );
    }

    pub fn no_flash(&self, process: &Process, offsets: &Offsets, flash_alpha: f32) {
        let flash_alpha = flash_alpha.clamp(0.0, 1.0);
        if process.read::<f32>(self.pawn + offsets.pawn.flash_alpha) != flash_alpha {
            process.write(self.pawn + offsets.pawn.flash_alpha, flash_alpha);
        }
    }

    pub fn set_fov(&self, process: &Process, offsets: &Offsets, value: u32) {
        let camera_service = process.read::<u64>(self.pawn + offsets.pawn.camera_services);
        if camera_service == 0 {
            return;
        }
        if process.read::<u32>(camera_service + offsets.camera_services.fov) != value {
            process.write(self.controller + offsets.controller.desired_fov, value);
        }
    }
}

impl CS2 {
    pub fn cache_players(&mut self) {
        let process = match &self.process {
            Some(process) => process,
            None => {
                self.players.clear();
                return;
            }
        };

        let local_player = match Player::local_player(process, &self.offsets) {
            Some(player) => player,
            None => return,
        };

        self.players.clear();
        for i in 0..=64 {
            let player = match Player::index(process, &self.offsets, i) {
                Some(player) => player,
                None => continue,
            };

            if !player.is_valid(process, &self.offsets) {
                continue;
            }

            if player == local_player {
                self.target.local_pawn_index = i - 1;
            } else {
                self.players.push(player);
            }
        }
    }
}
