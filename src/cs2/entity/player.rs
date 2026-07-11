use std::collections::HashMap;

use glam::{Vec2, Vec3, vec2};

use crate::{
    constants::cs2,
    cs2::{bones::Bones, entity::weapon::Weapon},
    data::SoundType,
};

use super::{CS2, weapon_class::WeaponClass};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Player {
    controller: usize,
    pub(crate) pawn: usize,
}

impl Player {
    pub fn entity(entity: usize) -> Self {
        Self {
            controller: 0,
            pawn: entity,
        }
    }

    #[allow(unused)]
    pub fn index(cs2: &CS2, index: usize) -> Option<Self> {
        let controller = Self::get_client_entity(cs2, index)?;
        let pawn_handle: i32 = cs2.process.read(controller + cs2.offsets.controller.pawn);
        if pawn_handle == -1 {
            return None;
        }
        Self::get_entity(cs2, pawn_handle).map(|pawn| Self { controller, pawn })
    }

    pub fn local_player(cs2: &CS2) -> Option<Self> {
        let controller = cs2.process.read(cs2.offsets.direct.local_player);
        if controller == 0 {
            return None;
        }
        let pawn_handle: i32 = cs2.process.read(controller + cs2.offsets.controller.pawn);
        if pawn_handle == -1 {
            return None;
        }
        Self::get_entity(cs2, pawn_handle).map(|pawn| Self { controller, pawn })
    }

    pub fn from_controller(controller: usize, cs2: &CS2) -> Option<Self> {
        let pawn_handle: i32 = cs2.process.read(controller + cs2.offsets.controller.pawn);
        if pawn_handle == -1 {
            return None;
        }
        Self::get_entity(cs2, pawn_handle).map(|pawn| Self { controller, pawn })
    }

    pub fn pawn(pawn: usize) -> Self {
        Self {
            controller: 0,
            pawn,
        }
    }

    pub fn get_client_entity(cs2: &CS2, index: usize) -> Option<usize> {
        let bucket_index = index >> 9;
        let index_in_bucket = index & 0x1FF;
        let bucket_ptr: usize = cs2
            .process
            .read(cs2.offsets.interface.entity + 0x08 * bucket_index);
        if bucket_ptr == 0 {
            return None;
        }
        let entity = cs2
            .process
            .read(bucket_ptr + cs2.offsets.entity_identity.size as usize * index_in_bucket);
        if entity == 0 {
            return None;
        }
        Some(entity)
    }

    fn get_entity(cs2: &CS2, handle: i32) -> Option<usize> {
        let index = handle as usize & 0x7FFF;
        let bucket_index = index >> 9;
        let index_in_bucket = index & 0x1FF;
        let bucket_ptr: usize = cs2
            .process
            .read(cs2.offsets.interface.entity + 8 * bucket_index);
        if bucket_ptr == 0 {
            return None;
        }

        let entity = cs2
            .process
            .read(bucket_ptr + cs2.offsets.entity_identity.size as usize * index_in_bucket);
        if entity == 0 {
            return None;
        }
        Some(entity)
    }

    pub fn health(&self, cs2: &CS2) -> i32 {
        let health = cs2.process.read(self.pawn + cs2.offsets.pawn.health);
        if !(0..=100).contains(&health) {
            return 0;
        }
        health
    }

    pub fn armor(&self, cs2: &CS2) -> i32 {
        cs2.process.read(self.pawn + cs2.offsets.pawn.armor)
    }

    pub fn team(&self, cs2: &CS2) -> u8 {
        cs2.process.read(self.pawn + cs2.offsets.pawn.team)
    }

    pub fn life_state(&self, cs2: &CS2) -> u8 {
        cs2.process.read(self.pawn + cs2.offsets.pawn.life_state)
    }

    pub fn steam_id(&self, cs2: &CS2) -> u64 {
        cs2.process
            .read(self.controller + cs2.offsets.controller.steam_id)
    }

    pub fn name(&self, cs2: &CS2) -> String {
        cs2.process
            .read_string_uncached(self.controller + cs2.offsets.controller.name)
    }

    /// returns a pawn-only player
    pub fn spectator_target(&self, cs2: &CS2) -> Option<Self> {
        let observer_services: usize = cs2
            .process
            .read(self.pawn + cs2.offsets.pawn.observer_services);
        if observer_services == 0 {
            return None;
        }

        let target: i32 = cs2
            .process
            .read(observer_services + cs2.offsets.observer_services.target);
        if target == -1 {
            return None;
        }

        let pawn = Player::get_entity(cs2, target)?;
        Some(Player::pawn(pawn))
    }

    pub fn deathmatch_immunity(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(self.pawn + cs2.offsets.pawn.deathmatch_immunity)
            != 0
    }

    pub fn weapon_name(&self, cs2: &CS2) -> String {
        // CEntityInstance
        let Some(weapon_entity_instance) = self.weapon_address(cs2) else {
            return String::from(cs2::WEAPON_UNKNOWN);
        };
        // CEntityIdentity, 0x10 = m_pEntity
        let weapon_entity_identity: usize = cs2.process.read(weapon_entity_instance + 0x10);
        if weapon_entity_identity == 0 {
            return String::from(cs2::WEAPON_UNKNOWN);
        }
        // 0x20 = m_designerName (pointer -> string)
        let weapon_name_pointer = cs2.process.read(weapon_entity_identity + 0x20);
        if weapon_name_pointer == 0 {
            return String::from(cs2::WEAPON_UNKNOWN);
        }
        let name = cs2.process.read_string(weapon_name_pointer);
        name.replace("weapon_", "")
    }

    pub fn weapon_class(&self, cs2: &CS2) -> WeaponClass {
        WeaponClass::from_string(&self.weapon_name(cs2))
    }

    fn weapon_handle(&self, cs2: &CS2) -> Option<i32> {
        let weapon_services: usize = cs2
            .process
            .read(self.pawn + cs2.offsets.pawn.weapon_services);
        if weapon_services == 0 {
            return None;
        }

        Some(
            cs2.process
                .read(weapon_services + cs2.offsets.weapon_services.active_weapon),
        )
    }

    fn weapon_address(&self, cs2: &CS2) -> Option<usize> {
        let handle = self.weapon_handle(cs2)?;
        if handle == 0 {
            return None;
        }

        let index = handle as usize & 0xFFF;
        Player::get_client_entity(cs2, index)
    }

    pub fn weapon(&self, cs2: &CS2) -> Weapon {
        let Some(weapon_handle) = self.weapon_handle(cs2) else {
            return Weapon::Unknown;
        };

        Weapon::from_handle(weapon_handle, cs2).unwrap_or_default()
    }

    pub fn all_weapons(&self, cs2: &CS2) -> Vec<Weapon> {
        let mut weapons = vec![];
        let weapon_services: usize = cs2
            .process
            .read(self.pawn + cs2.offsets.pawn.weapon_services);
        if weapon_services == 0 {
            return weapons;
        }

        let length: i32 = cs2
            .process
            .read(weapon_services + cs2.offsets.weapon_services.weapons);
        let weapon_list: usize = cs2
            .process
            .read(weapon_services + cs2.offsets.weapon_services.weapons + 0x08);

        for i in 0..length as usize {
            let weapon_handle = cs2.process.read(weapon_list + 0x04 * i);

            let Some(weapon) = Weapon::from_handle(weapon_handle, cs2) else {
                continue;
            };

            weapons.push(weapon);
        }

        weapons
    }

    pub fn clip_ammo(&self, cs2: &CS2) -> i32 {
        let Some(weapon) = self.weapon_address(cs2) else {
            return 0;
        };

        cs2.process.read(weapon + cs2.offsets.weapon.clip_primary)
    }

    pub fn reserve_ammo(&self, cs2: &CS2) -> i32 {
        let Some(weapon) = self.weapon_address(cs2) else {
            return 0;
        };

        cs2.process.read(weapon + cs2.offsets.weapon.reserve_ammo)
    }

    fn game_scene_node(&self, cs2: &CS2) -> usize {
        cs2.process
            .read(self.pawn + cs2.offsets.pawn.game_scene_node)
    }

    fn is_dormant(&self, cs2: &CS2) -> bool {
        let gs_node = self.game_scene_node(cs2);
        cs2.process
            .read::<u8>(gs_node + cs2.offsets.game_scene_node.dormant)
            != 0
    }

    pub fn position(&self, cs2: &CS2) -> Vec3 {
        let gs_node = self.game_scene_node(cs2);
        cs2.process
            .read(gs_node + cs2.offsets.game_scene_node.origin)
    }

    pub fn eye_position(&self, cs2: &CS2) -> Vec3 {
        let position = self.position(cs2);
        let eye_offset: Vec3 = cs2.process.read(self.pawn + cs2.offsets.pawn.eye_offset);

        position + eye_offset
    }

    pub fn bone_position(&self, cs2: &CS2, bone_index: u64) -> Vec3 {
        let gs_node = self.game_scene_node(cs2);
        let bone_data: usize = cs2.process.read(
            gs_node
                + cs2.offsets.game_scene_node.model_state
                + cs2.offsets.model_state.skeleton_instance,
        );

        if bone_data == 0 {
            return Vec3::ZERO;
        }

        cs2.process.read(bone_data + (bone_index as usize * 32))
    }

    pub fn all_bones(&self, cs2: &CS2) -> HashMap<Bones, Vec3> {
        use strum::IntoEnumIterator;

        let mut bones = HashMap::with_capacity(20);
        let gs_node = self.game_scene_node(cs2);
        let bone_data: usize = cs2.process.read(
            gs_node
                + cs2.offsets.game_scene_node.model_state
                + cs2.offsets.model_state.skeleton_instance,
        );

        if bone_data == 0 {
            return bones;
        }

        let bones_data: [u8; 32 * 32] = cs2.process.read_or_zeroed(bone_data);

        for bone in Bones::iter() {
            let start = bone.u64() as usize * 32;
            let pos = bytemuck::from_bytes(&bones_data[start..start + 3 * 4]);
            bones.insert(bone, *pos);
        }

        bones
    }

    pub fn shots_fired(&self, cs2: &CS2) -> i32 {
        cs2.process.read(self.pawn + cs2.offsets.pawn.shots_fired)
    }

    pub fn fov_multiplier(&self, cs2: &CS2) -> f32 {
        cs2.process
            .read(self.pawn + cs2.offsets.pawn.fov_multiplier)
    }

    pub fn spotted_mask(&self, cs2: &CS2) -> i64 {
        cs2.process
            .read(self.pawn + cs2.offsets.pawn.spotted_state + cs2.offsets.spotted_state.mask)
    }

    pub fn is_valid(&self, cs2: &CS2) -> bool {
        if self.is_dormant(cs2) {
            return false;
        }

        if self.health(cs2) <= 0 {
            return false;
        }

        if self.life_state(cs2) != 0 {
            return false;
        }

        if self.deathmatch_immunity(cs2) {
            return false;
        }

        true
    }

    pub fn is_flashed(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<f32>(self.pawn + cs2.offsets.pawn.flash_duration)
            > 0.2
    }

    pub fn is_scoped(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(self.pawn + cs2.offsets.pawn.is_scoped)
            != 0
    }

    pub fn color(&self, cs2: &CS2) -> i32 {
        cs2.process
            .read(self.controller + cs2.offsets.controller.color)
    }

    pub fn rotation(&self, cs2: &CS2) -> f32 {
        cs2.process
            .read(self.pawn + cs2.offsets.pawn.eye_angles + 0x04)
    }

    pub fn view_angles(&self, cs2: &CS2) -> Vec2 {
        cs2.process.read(self.pawn + cs2.offsets.pawn.view_angles)
    }

    pub fn aim_punch(&self, cs2: &CS2) -> Vec2 {
        let aim_punch_services: usize = cs2
            .process
            .read(self.pawn + cs2.offsets.pawn.aim_punch_services);
        if aim_punch_services == 0 {
            return Vec2::ZERO;
        }

        let length: usize = cs2
            .process
            .read(aim_punch_services + cs2.offsets.aim_punch_services.aim_punch_cache);
        if length < 1 {
            return Vec2::ZERO;
        }

        let data_address: usize = cs2
            .process
            .read(aim_punch_services + cs2.offsets.aim_punch_services.aim_punch_cache + 0x08);
        if data_address > usize::MAX - 50000 {
            return Vec2::ZERO;
        }

        cs2.process.read(data_address + (length - 1) * 12)
    }

    pub fn has_defuser(&self, cs2: &CS2) -> bool {
        let item_services: usize = cs2.process.read(self.pawn + cs2.offsets.pawn.item_services);
        if item_services == 0 {
            return false;
        }

        cs2.process
            .read::<u8>(item_services + cs2.offsets.item_services.has_defuser)
            != 0
    }

    pub fn has_helmet(&self, cs2: &CS2) -> bool {
        let item_services: usize = cs2.process.read(self.pawn + cs2.offsets.pawn.item_services);
        if item_services == 0 {
            return false;
        }

        cs2.process
            .read::<u8>(item_services + cs2.offsets.item_services.has_helmet)
            != 0
    }

    pub fn has_bomb(&self, cs2: &CS2) -> bool {
        let weapons = self.all_weapons(cs2);
        weapons.contains(&Weapon::C4)
    }

    fn action_tracking_services(&self, cs2: &CS2) -> usize {
        cs2.process
            .read(self.controller + cs2.offsets.controller.action_tracking_services)
    }

    #[allow(dead_code)]
    pub fn round_kills(&self, cs2: &CS2) -> Option<i32> {
        let action_tracking_services = self.action_tracking_services(cs2);
        if action_tracking_services == 0 {
            return None;
        }

        Some(
            cs2.process
                .read(action_tracking_services + cs2.offsets.action_tracking.round_kills),
        )
    }

    #[allow(dead_code)]
    pub fn round_damage(&self, cs2: &CS2) -> Option<f32> {
        let action_tracking_services = self.action_tracking_services(cs2);
        if action_tracking_services == 0 {
            return None;
        }

        Some(
            cs2.process
                .read(action_tracking_services + cs2.offsets.action_tracking.round_damage),
        )
    }

    pub fn visible(&self, cs2: &CS2, local_player: &Player) -> bool {
        if let Some(bvh) = &cs2.bvh {
            let eye_pos = local_player.eye_position(cs2);
            const CHECKED_BONES: [Bones; 5] = [
                Bones::Head,
                Bones::LeftFoot,
                Bones::RightFoot,
                Bones::LeftHand,
                Bones::RightHand,
            ];
            if !CHECKED_BONES
                .iter()
                .any(|bone| bvh.has_line_of_sight(eye_pos, self.bone_position(cs2, bone.u64())))
            {
                return false;
            }
        } else {
            let spotted_mask = self.spotted_mask(cs2);
            if (spotted_mask & (1 << cs2.target.local_pawn_index)) == 0 {
                return false;
            }
        }
        true
    }

    pub fn crosshair_entity(&self, cs2: &CS2) -> Option<Self> {
        let index: i32 = cs2
            .process
            .read(self.pawn + cs2.offsets.pawn.crosshair_entity);
        if index == -1 {
            return None;
        }

        let entity = Player::get_client_entity(cs2, index as usize)?;
        let player = Player {
            controller: 0,
            pawn: entity,
        };
        if !player.is_valid(cs2) {
            return None;
        }
        Some(player)
    }

    pub fn velocity(&self, cs2: &CS2) -> Vec3 {
        cs2.process.read(self.pawn + cs2.offsets.pawn.velocity)
    }

    fn is_in_air(&self, cs2: &CS2) -> bool {
        let flags = cs2.process.read::<i32>(self.pawn + cs2.offsets.pawn.flags);
        // FL_ONGROUND = (1 << 0)
        (flags & 1) == 0
    }

    pub fn is_making_sound(&self, cs2: &CS2) -> Option<SoundType> {
        if self.shots_fired(cs2) > 0 {
            return Some(SoundType::Gunshot);
        }

        let velocity = self.velocity(cs2);
        let speed = vec2(velocity.x, velocity.y).length();
        let current_weapon = self.weapon(cs2);

        let is_jumping = velocity.z > 100.0 && self.is_in_air(cs2);
        // knife walking speed is 250 units/s
        let is_walking = speed > 100.0;
        let is_standing = speed < 10.0;

        // check for scoping (only for snipers)
        let is_scoped = self.is_scoped(cs2);

        if is_walking || is_standing {
            return None;
        }

        // awp and scout are not the only snipers...
        if is_scoped && WeaponClass::from_string(current_weapon.as_ref()) == WeaponClass::Sniper {
            Some(SoundType::Weapon)
        } else if speed > 150.0 || is_jumping || velocity.z < -200.0 {
            Some(SoundType::Footstep)
        } else {
            None
        }
    }

    pub fn no_flash(&self, cs2: &CS2, flash_alpha: f32) {
        let flash_alpha = flash_alpha.clamp(0.0, 255.0);
        let current_alpha: f32 = cs2.process.read(self.pawn + cs2.offsets.pawn.flash_alpha);
        if current_alpha != flash_alpha {
            cs2.process
                .write(self.pawn + cs2.offsets.pawn.flash_alpha, flash_alpha);
        }
    }

    pub fn set_fov(&self, cs2: &CS2, value: u32) {
        let camera_service = cs2
            .process
            .read::<usize>(self.pawn + cs2.offsets.pawn.camera_services);
        if camera_service == 0 {
            return;
        }
        let current: u32 = cs2
            .process
            .read(camera_service + cs2.offsets.camera_services.fov);
        if current != 0 && current != value {
            cs2.process
                .write(self.controller + cs2.offsets.controller.desired_fov, value);
        }
    }
}

impl CS2 {
    #[allow(unused)]
    pub fn cache_players(&mut self) {
        if !self.process.is_valid() {
            self.players.clear();
            return;
        };

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        self.weapon = local_player.weapon(self);

        self.players.clear();

        for i in 0..=64 {
            let player = match Player::index(self, i) {
                Some(player) => player,
                None => continue,
            };

            if !player.is_valid(self) {
                continue;
            }

            if player == local_player {
                self.target.local_pawn_index = i as u64 - 1;
            } else {
                self.players.push(player);
            }
        }
    }
}
