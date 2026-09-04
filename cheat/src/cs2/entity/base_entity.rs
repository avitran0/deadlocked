use std::ops::Deref;

use crate::cs2::{
    CS2,
    class::{life_state::LifeState, net_vec::NetworkVelocityVector},
};
use glam::{Mat4, Quat, Vec3};
use shared::Team;

/// Common handle-backed functionality shared by all client entities.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BaseEntity {
    handle: usize,
}

impl BaseEntity {
    pub(crate) fn new(handle: usize) -> Self {
        Self { handle }
    }

    fn valid_ptr(ptr: usize) -> bool {
        ptr != 0 && ptr >> 48 == 0
    }

    pub fn health(&self, cs2: &CS2) -> i32 {
        cs2.process.read(self.handle + cs2.offsets.entity.health)
    }

    pub fn max_health(&self, cs2: &CS2) -> i32 {
        cs2.process
            .read(self.handle + cs2.offsets.entity.max_health)
    }

    pub fn team(&self, cs2: &CS2) -> Team {
        cs2.process
            .read_as::<u8, Team>(self.handle + cs2.offsets.entity.team)
    }

    pub fn life_state(&self, cs2: &CS2) -> LifeState {
        cs2.process
            .read_as::<u8, LifeState>(self.handle + cs2.offsets.entity.life_state)
    }

    pub fn game_scene_node(&self, cs2: &CS2) -> usize {
        let node = cs2
            .process
            .read::<usize>(self.handle + cs2.offsets.entity.game_scene_node);
        if Self::valid_ptr(node) { node } else { 0 }
    }

    pub fn position(&self, cs2: &CS2) -> Vec3 {
        let node = self.game_scene_node(cs2);
        if node == 0 {
            return Vec3::ZERO;
        }
        let Some(addr) = node.checked_add(cs2.offsets.game_scene_node.origin) else {
            return Vec3::ZERO;
        };
        cs2.process.read(addr)
    }

    pub fn has_valid_position(&self, cs2: &CS2) -> bool {
        self.game_scene_node(cs2) != 0
    }

    pub fn collision_bounds(&self, cs2: &CS2) -> (Vec3, Vec3) {
        let collision: usize = cs2.process.read(self.handle + cs2.offsets.entity.collision);
        if collision == 0 {
            return (Vec3::ZERO, Vec3::ZERO);
        }
        (
            cs2.process.read(collision + cs2.offsets.collision.mins),
            cs2.process.read(collision + cs2.offsets.collision.maxs),
        )
    }

    pub fn collision_transform(&self, cs2: &CS2) -> Mat4 {
        let node = self.game_scene_node(cs2);
        if node == 0 {
            return Mat4::IDENTITY;
        }
        let position: Vec3 = self.position(cs2);
        let values: [f32; 4] = cs2
            .process
            .read(node + cs2.offsets.game_scene_node.node_to_world + 0x10);
        let rotation = Quat::from_xyzw(values[0], values[1], values[2], values[3]);
        let rotation = if rotation.is_finite() && rotation.length_squared() > f32::EPSILON {
            rotation.normalize()
        } else {
            Quat::IDENTITY
        };
        Mat4::from_scale_rotation_translation(Vec3::ONE, rotation, position)
    }

    #[allow(dead_code)]
    pub fn velocity(&self, cs2: &CS2) -> Vec3 {
        NetworkVelocityVector::read(cs2, self.handle + cs2.offsets.entity.velocity).to_vec()
    }
}

impl Deref for BaseEntity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
