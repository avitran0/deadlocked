use std::ops::Deref;

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::cs2::CS2;

/// Common handle-backed functionality shared by all client entities.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BaseEntity {
    handle: usize,
}

impl BaseEntity {
    pub(crate) fn new(handle: usize) -> Self {
        Self { handle }
    }

    pub fn game_scene_node(&self, cs2: &CS2) -> usize {
        cs2.process
            .read(self.handle + cs2.offsets.pawn.game_scene_node)
    }

    pub fn position(&self, cs2: &CS2) -> Vec3 {
        cs2.process
            .read(self.game_scene_node(cs2) + cs2.offsets.game_scene_node.origin)
    }

    #[allow(dead_code)]
    pub fn velocity(&self, cs2: &CS2) -> Vec3 {
        cs2.process.read(self.handle + cs2.offsets.pawn.velocity)
    }
}

impl std::ops::Add<usize> for BaseEntity {
    type Output = usize;

    fn add(self, rhs: usize) -> Self::Output {
        self.handle + rhs
    }
}

impl Deref for BaseEntity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
