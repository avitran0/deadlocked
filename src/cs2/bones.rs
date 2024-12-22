#![allow(unused)]

use strum::EnumIter;

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Bones {
    Pelvis = 0,
    Spine1 = 2,
    Spine2 = 4,
    Neck = 5,
    Head = 6,
    LeftShoulder = 8,
    LeftElbow = 9,
    LeftHand = 10,
    RightShoulder = 13,
    RightElbow = 14,
    RightHand = 15,
    LeftHip = 22,
    LeftKnee = 23,
    LeftFoot = 24,
    RightHip = 25,
    RightKnee = 26,
    RightFoot = 27,
}

impl Bones {
    pub fn u64(self) -> u64 {
        self as u64
    }

    pub const CONNECTIONS: [(Bones, Bones); 16] = [
        (Bones::Pelvis, Bones::Spine1),
        (Bones::Spine1, Bones::Spine2),
        (Bones::Spine2, Bones::Neck),
        (Bones::Neck, Bones::Head),
        (Bones::Neck, Bones::LeftShoulder),
        (Bones::LeftShoulder, Bones::LeftElbow),
        (Bones::LeftElbow, Bones::LeftHand),
        (Bones::Neck, Bones::RightShoulder),
        (Bones::RightShoulder, Bones::RightElbow),
        (Bones::RightElbow, Bones::RightHand),
        (Bones::Pelvis, Bones::LeftHip),
        (Bones::LeftHip, Bones::LeftKnee),
        (Bones::LeftKnee, Bones::LeftFoot),
        (Bones::Pelvis, Bones::RightHip),
        (Bones::RightHip, Bones::RightKnee),
        (Bones::RightKnee, Bones::RightFoot),
    ];
}
