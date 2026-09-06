use shared::{BoneTransform, ChickenBones, ChickenInfo};
use strum::IntoEnumIterator;

use crate::{
    constants::cs2::CHICKEN_SKELETON_BONE_COUNT,
    cs2::{CS2, entity::player::Player},
};

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

        let skeleton = self.skeleton_transforms(cs2, CHICKEN_SKELETON_BONE_COUNT);
        let bones = ChickenBones::iter()
            .filter_map(|bone| Some((bone, skeleton.get(bone.usize())?.position)))
            .collect();

        ChickenInfo {
            position: entity.position(cs2),
            visible,
            bones,
            skeleton,
        }
    }

    // same 32-byte-per-joint layout as Player::skeleton_transforms()
    fn skeleton_transforms(&self, cs2: &CS2, count: usize) -> Vec<BoneTransform> {
        let entity = Player::entity(self.controller);
        let gs_node = entity.game_scene_node(cs2);
        let bone_data: usize = cs2
            .process
            .read(gs_node + cs2.offsets.game_scene_node.model_state + 0x80);

        if bone_data == 0 {
            return vec![BoneTransform::default(); count];
        }

        (0..count)
            .map(|index| {
                let slot = bone_data + index * 32;
                BoneTransform {
                    position: cs2.process.read(slot),
                    rotation: cs2.process.read(slot + 16),
                }
            })
            .collect()
    }
}
