use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, EnumIter, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Bones {
    Hip = 1,
    Spine1 = 2,
    Spine2 = 3,
    Spine3 = 4,
    Spine4 = 5,
    Neck = 6,
    Head = 7,
    LeftShoulder = 9,
    LeftElbow = 10,
    LeftHand = 11,
    RightShoulder = 13,
    RightElbow = 14,
    RightHand = 15,
    LeftHip = 17,
    LeftKnee = 18,
    LeftFoot = 19,
    RightHip = 20,
    RightKnee = 21,
    RightFoot = 22,
}

impl Bones {
    pub const CONNECTIONS: [(Self, Self); 18] = [
        // spine
        (Self::Hip, Self::Spine1),
        (Self::Spine1, Self::Spine2),
        (Self::Spine2, Self::Spine3),
        (Self::Spine3, Self::Spine4),
        (Self::Spine4, Self::Neck),
        (Self::Neck, Self::Head),
        // left arm
        (Self::Neck, Self::LeftShoulder),
        (Self::LeftShoulder, Self::LeftElbow),
        (Self::LeftElbow, Self::LeftHand),
        // right arm
        (Self::Neck, Self::RightShoulder),
        (Self::RightShoulder, Self::RightElbow),
        (Self::RightElbow, Self::RightHand),
        // left leg
        (Self::Hip, Self::LeftHip),
        (Self::LeftHip, Self::LeftKnee),
        (Self::LeftKnee, Self::LeftFoot),
        // right leg
        (Self::Hip, Self::RightHip),
        (Self::RightHip, Self::RightKnee),
        (Self::RightKnee, Self::RightFoot),
    ];

    pub fn u64(self) -> u64 {
        self as u64
    }
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
