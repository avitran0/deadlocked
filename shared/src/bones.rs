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

    /// closest point on this capsule's real segment to a ray
    /// (`ray_origin + t * ray_dir`, `t >= 0`, `ray_dir` must be unit
    /// length), and the perpendicular distance between them. lets the
    /// aimbot/triggerbot test against the capsule's actual shape instead of
    /// approximating it as a single point + an angular FOV radius.
    ///
    /// adapted from the standard closest-point-between-two-segments
    /// algorithm (Ericson, "Real-Time Collision Detection", 5.1.9), with the
    /// ray's parameter only clamped at its lower bound (0) since a ray has
    /// no upper bound, unlike the capsule's segment parameter which is
    /// clamped to [0, 1] same as a normal segment-segment test
    pub fn closest_approach(
        &self,
        transform: BoneTransform,
        ray_origin: Vec3,
        ray_dir: Vec3,
    ) -> (Vec3, f32) {
        let (a, b) = self.world_points(transform);
        let d1 = ray_dir;
        let d2 = b - a;
        let r = ray_origin - a;

        let dot_aa = d1.dot(d1);
        let dot_ee = d2.dot(d2);
        let dot_ef = d2.dot(r);

        const EPS: f32 = 1e-6;

        let (s, t) = if dot_aa <= EPS && dot_ee <= EPS {
            (0.0, 0.0)
        } else if dot_aa <= EPS {
            (0.0, (dot_ef / dot_ee).clamp(0.0, 1.0))
        } else {
            let dot_c = d1.dot(r);
            if dot_ee <= EPS {
                (f32::max(-dot_c / dot_aa, 0.0), 0.0)
            } else {
                let dot_b = d1.dot(d2);
                let denom = dot_aa * dot_ee - dot_b * dot_b;
                let mut s = if denom.abs() > EPS {
                    f32::max((dot_b * dot_ef - dot_c * dot_ee) / denom, 0.0)
                } else {
                    0.0
                };
                let mut t = (dot_b * s + dot_ef) / dot_ee;
                if t < 0.0 {
                    t = 0.0;
                    s = f32::max(-dot_c / dot_aa, 0.0);
                } else if t > 1.0 {
                    t = 1.0;
                    s = f32::max((dot_b - dot_c) / dot_aa, 0.0);
                }
                (s, t)
            }
        };

        let closest_on_ray = ray_origin + d1 * s;
        let closest_on_capsule = a + d2 * t;
        let distance = (closest_on_capsule - closest_on_ray).length();
        (closest_on_capsule, distance)
    }
}

// values extracted from the game's own compiled hitbox set (agents share one
// skeleton, so this table is identical for every agent skin), not approximated
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

#[cfg(test)]
mod tests {
    use super::*;

    fn segment_capsule() -> HitboxCapsule {
        // an arbitrary segment along Z, from z=0 to z=10; radius unused by
        // closest_approach itself, only by callers comparing against it
        HitboxCapsule {
            bone: Bones::Head,
            radius: 4.0,
            point0: Vec3::new(0.0, 0.0, 0.0),
            point1: Vec3::new(0.0, 0.0, 10.0),
        }
    }

    fn identity_transform() -> BoneTransform {
        BoneTransform {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        }
    }

    #[test]
    fn ray_directly_intersects_segment_midpoint() {
        let capsule = segment_capsule();
        let ray_origin = Vec3::new(5.0, 0.0, 5.0);
        let ray_dir = Vec3::new(-1.0, 0.0, 0.0);

        let (point, distance) = capsule.closest_approach(identity_transform(), ray_origin, ray_dir);

        assert!(point.abs_diff_eq(Vec3::new(0.0, 0.0, 5.0), 1e-4));
        assert!(
            distance < 1e-4,
            "ray passes through the segment, expected ~0, got {distance}"
        );
    }

    #[test]
    fn ray_misses_perpendicular_by_a_known_offset() {
        let capsule = segment_capsule();
        // shifted +3 in Y from the segment, so it never touches it; closest
        // approach is a pure 3-unit perpendicular gap
        let ray_origin = Vec3::new(5.0, 3.0, 5.0);
        let ray_dir = Vec3::new(-1.0, 0.0, 0.0);

        let (point, distance) = capsule.closest_approach(identity_transform(), ray_origin, ray_dir);

        assert!(point.abs_diff_eq(Vec3::new(0.0, 0.0, 5.0), 1e-4));
        assert!((distance - 3.0).abs() < 1e-4);
    }

    #[test]
    fn ray_beyond_segment_end_clamps_to_endpoint() {
        let capsule = segment_capsule();
        let ray_origin = Vec3::new(5.0, 0.0, 20.0);
        let ray_dir = Vec3::new(-1.0, 0.0, 0.0);

        let (point, distance) = capsule.closest_approach(identity_transform(), ray_origin, ray_dir);

        // capsule clamps to its z=10 endpoint; the ray's own closest point
        // ends up at (0,0,20) (5 units along its direction), so the real
        // perpendicular gap between those two points is a pure 10-unit z
        // difference, not the distance from the endpoint back to the ray's
        // origin
        assert!(point.abs_diff_eq(Vec3::new(0.0, 0.0, 10.0), 1e-4));
        assert!((distance - 10.0).abs() < 1e-4);
    }

    #[test]
    fn ray_pointing_away_clamps_to_its_own_origin() {
        let capsule = segment_capsule();
        // segment spans z=0..10; ray starts before it and points further
        // away (more negative z), so the true closest approach along the
        // ray (not the infinite line) is at the ray's own origin
        let ray_origin = Vec3::new(0.0, 0.0, -5.0);
        let ray_dir = Vec3::new(0.0, 0.0, -1.0);

        let (point, distance) = capsule.closest_approach(identity_transform(), ray_origin, ray_dir);

        assert!(point.abs_diff_eq(Vec3::new(0.0, 0.0, 0.0), 1e-4));
        assert!((distance - 5.0).abs() < 1e-4);
    }
}
