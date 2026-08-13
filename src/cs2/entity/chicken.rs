use std::collections::HashMap;

use glam::Vec3;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChickenInfo {
    #[allow(dead_code)]
    pub position: Vec3,
    pub visible: bool,
    pub bones: HashMap<ChickenBones, Vec3>,
}

#[derive(Debug, Clone, Copy, EnumIter, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChickenBones {
    Pelvis = 2,

    Spine0 = 3,
    Spine1 = 4,

    Neck0 = 9,
    Neck1 = 10,
    Neck2 = 11,
    Head = 12,

    ClavL = 5,
    Wing0L = 6,
    Wing1L = 7,
    Wing2L = 8,

    ClavR = 24,
    Wing0R = 25,
    Wing1R = 26,
    Wing2R = 27,

    Leg0L = 30,
    Leg1L = 31,
    Leg2L = 32,
    FootL = 33,

    Leg0R = 41,
    Leg1R = 42,
    Leg2R = 43,
    FootR = 44,
}

impl ChickenBones {
    pub const CONNECTIONS: [(Self, Self); 22] = [
        // body / spine / head
        (Self::Pelvis, Self::Spine0),
        (Self::Spine0, Self::Spine1),
        (Self::Spine1, Self::Neck0),
        (Self::Neck0, Self::Neck1),
        (Self::Neck1, Self::Neck2),
        (Self::Neck2, Self::Head),
        // left wing
        (Self::Spine1, Self::ClavL),
        (Self::ClavL, Self::Wing0L),
        (Self::Wing0L, Self::Wing1L),
        (Self::Wing1L, Self::Wing2L),
        // right wing
        (Self::Spine1, Self::ClavR),
        (Self::ClavR, Self::Wing0R),
        (Self::Wing0R, Self::Wing1R),
        (Self::Wing1R, Self::Wing2R),
        // left leg
        (Self::Pelvis, Self::Leg0L),
        (Self::Leg0L, Self::Leg1L),
        (Self::Leg1L, Self::Leg2L),
        (Self::Leg2L, Self::FootL),
        // right leg
        (Self::Pelvis, Self::Leg0R),
        (Self::Leg0R, Self::Leg1R),
        (Self::Leg1R, Self::Leg2R),
        (Self::Leg2R, Self::FootR),
    ];

    pub fn usize(self) -> usize {
        self as usize
    }
}
