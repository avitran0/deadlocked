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
