use glam::Vec3;
use shared::InfernoInfo;

use crate::cs2::{CS2, entity::base_entity::BaseEntity};

#[derive(Clone, PartialEq)]
pub struct Inferno {
    controller: BaseEntity,
}

impl Inferno {
    pub fn new(controller: usize) -> Self {
        Self {
            controller: BaseEntity::new(controller),
        }
    }

    pub fn info(&self, cs2: &CS2) -> InfernoInfo {
        // todo: m_nInfernoType?
        InfernoInfo {
            entity: *self.controller,
            position: self.controller.position(cs2),
            hull: self.hull(cs2),
        }
    }

    pub fn controller_valid(&self, cs2: &CS2) -> bool {
        self.controller.has_valid_position(cs2)
    }

    pub fn is_burning(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(*self.controller + cs2.offsets.inferno.is_burning)
            != 0
    }

    pub fn hull(&self, cs2: &CS2) -> Vec<Vec3> {
        if !self.is_burning(cs2) {
            return Vec::new();
        }
        let count: i32 = cs2
            .process
            .read(*self.controller + cs2.offsets.inferno.fire_count);
        if !(0..=64).contains(&count) {
            return Vec::new();
        }
        cs2.process.read_typed_vec(
            *self.controller + cs2.offsets.inferno.fire_positions,
            std::mem::size_of::<Vec3>(),
            count as usize,
        )
    }
}
