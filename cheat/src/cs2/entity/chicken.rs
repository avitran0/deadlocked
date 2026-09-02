use std::collections::HashMap;

use glam::Vec3;
use shared::{ChickenBones, ChickenInfo};
use strum::IntoEnumIterator;

use crate::cs2::{CS2, entity::player::Player};

#[derive(Debug, Clone, PartialEq)]
pub struct Chicken {
    controller: usize,
}

impl Chicken {
    pub fn new(controller: usize) -> Self {
        Self { controller }
    }

    pub fn info(&self, cs2: &CS2) -> ChickenInfo {
        let entity = Player::entity(self.controller);

        let visible = if let Some(local_player) = Player::local_player(cs2) {
            entity.visible(cs2, &local_player)
        } else {
            false
        };

        ChickenInfo {
            position: entity.position(cs2),
            visible,
            bones: self.bones(cs2),
        }
    }

    fn bones(&self, cs2: &CS2) -> HashMap<ChickenBones, Vec3> {
        let mut bones = HashMap::with_capacity(ChickenBones::iter().len());

        let entity = Player::entity(self.controller);
        let gs_node = entity.game_scene_node(cs2);
        let bone_data: usize = cs2
            .process
            .read(gs_node + cs2.offsets.game_scene_node.model_state + 0x80);

        if bone_data == 0 {
            return bones;
        }

        let bone_data: Vec<Vec3> = cs2.process.read_typed_vec(bone_data, 32, 48);

        for bone in ChickenBones::iter() {
            bones.insert(bone, bone_data[bone.usize()]);
        }
        bones
    }
}
