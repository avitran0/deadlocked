use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// live per-bone transform read from the game's skeleton instance
#[derive(Debug, Clone, Copy, Default)]
pub struct BoneTransform {
    pub position: Vec3,
    pub rotation: Quat,
}

/// a real bullet-hit capsule, in the parent bone's local space
#[derive(Debug, Clone, Copy)]
pub struct HitboxCapsule {
    pub bone: Bones,
    pub radius: f32,
    pub point0: Vec3,
    pub point1: Vec3,
}

impl HitboxCapsule {
    pub fn world_points(&self, transform: BoneTransform) -> (Vec3, Vec3) {
        (
            transform.position + transform.rotation * self.point0,
            transform.position + transform.rotation * self.point1,
        )
    }

    pub fn world_center(&self, transform: BoneTransform) -> Vec3 {
        let (p0, p1) = self.world_points(transform);
        (p0 + p1) * 0.5
    }
}

// extracted from the game's own compiled hitbox set, shared by every agent
pub const HITBOXES: [HitboxCapsule; 19] = [
    HitboxCapsule {
        bone: Bones::Head,
        radius: 4.3,
        point0: Vec3::new(-1.0, 1.8, 0.0),
        point1: Vec3::new(3.5, 0.2, 0.0),
    },
    HitboxCapsule {
        bone: Bones::Neck,
        radius: 3.5,
        point0: Vec3::new(0.0, -0.4, 0.0),
        point1: Vec3::new(1.4, -0.2, 0.0),
    },
    HitboxCapsule {
        bone: Bones::Hip,
        radius: 6.0,
        point0: Vec3::new(-2.7, 1.1, -3.2),
        point1: Vec3::new(-2.7, 1.1, 3.2),
    },
    HitboxCapsule {
        bone: Bones::Spine1,
        radius: 6.0,
        point0: Vec3::new(1.4, 0.8, 3.1),
        point1: Vec3::new(1.4, 0.8, -3.1),
    },
    HitboxCapsule {
        bone: Bones::Spine2,
        radius: 6.5,
        point0: Vec3::new(3.8, 0.8, -2.4),
        point1: Vec3::new(3.8, 0.4, 2.4),
    },
    HitboxCapsule {
        bone: Bones::Spine3,
        radius: 6.2,
        point0: Vec3::new(4.8, 0.15, -4.1),
        point1: Vec3::new(4.8, 0.15, 4.1),
    },
    HitboxCapsule {
        bone: Bones::Spine4,
        radius: 5.0,
        point0: Vec3::new(2.5, -0.6, -6.0),
        point1: Vec3::new(2.5, -0.6, 6.0),
    },
    HitboxCapsule {
        bone: Bones::LeftHip,
        radius: 5.0,
        point0: Vec3::new(1.3, -0.2, 0.0),
        point1: Vec3::new(16.5, -0.7, 0.0),
    },
    HitboxCapsule {
        bone: Bones::RightHip,
        radius: 5.0,
        point0: Vec3::new(-1.3, 0.0, -0.6),
        point1: Vec3::new(-16.5, 0.0, -0.7),
    },
    HitboxCapsule {
        bone: Bones::LeftKnee,
        radius: 4.0,
        point0: Vec3::new(0.1, -0.4, 0.2),
        point1: Vec3::new(17.0, -0.4, 0.7),
    },
    HitboxCapsule {
        bone: Bones::RightKnee,
        radius: 4.0,
        point0: Vec3::new(-0.1, 0.0, -0.2),
        point1: Vec3::new(-17.0, 0.4, -0.7),
    },
    HitboxCapsule {
        bone: Bones::LeftFoot,
        radius: 2.6,
        point0: Vec3::new(-0.0, -3.43, -0.52),
        point1: Vec3::new(8.0, 0.74, 0.33),
    },
    HitboxCapsule {
        bone: Bones::RightFoot,
        radius: 2.6,
        point0: Vec3::new(-7.98, -0.75, -0.27),
        point1: Vec3::new(-0.02, 3.44, 0.58),
    },
    HitboxCapsule {
        bone: Bones::LeftHand,
        radius: 2.3,
        point0: Vec3::new(0.0, 0.3, 0.0),
        point1: Vec3::new(3.59, 1.15, 0.11),
    },
    HitboxCapsule {
        bone: Bones::RightHand,
        radius: 2.3,
        point0: Vec3::new(0.0, -0.3, 0.02),
        point1: Vec3::new(-3.44, -1.17, -0.09),
    },
    HitboxCapsule {
        bone: Bones::LeftShoulder,
        radius: 3.3,
        point0: Vec3::new(0.0, 0.0, 0.0),
        point1: Vec3::new(11.2, 0.0, 0.0),
    },
    HitboxCapsule {
        bone: Bones::LeftElbow,
        radius: 3.0,
        point0: Vec3::new(0.0, 0.0, 0.0),
        point1: Vec3::new(10.0, 0.0, 0.0),
    },
    HitboxCapsule {
        bone: Bones::RightShoulder,
        radius: 3.3,
        point0: Vec3::new(0.0, 0.0, 0.0),
        point1: Vec3::new(-11.2, 0.0, 0.0),
    },
    HitboxCapsule {
        bone: Bones::RightElbow,
        radius: 3.0,
        point0: Vec3::new(0.0, 0.0, 0.0),
        point1: Vec3::new(-10.0, 0.0, -0.5),
    },
];

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

    pub fn hitbox(self) -> HitboxCapsule {
        HITBOXES
            .into_iter()
            .find(|hitbox| hitbox.bone == self)
            .expect("every Bones variant has a matching entry in HITBOXES")
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
