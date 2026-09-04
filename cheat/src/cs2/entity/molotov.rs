use shared::MolotovInfo;

use crate::cs2::{CS2, entity::base_entity::BaseEntity};

#[derive(Clone, PartialEq)]
pub struct Molotov {
    controller: BaseEntity,
}

impl Molotov {
    pub fn new(controller: usize) -> Self {
        Self {
            controller: BaseEntity::new(controller),
        }
    }

    pub fn info(&self, cs2: &CS2) -> MolotovInfo {
        MolotovInfo {
            entity: *self.controller,
            position: self.controller.position(cs2),
            is_incendiary: self.is_incendiary(cs2),
        }
    }

    pub fn controller_valid(&self, cs2: &CS2) -> bool {
        self.controller.has_valid_position(cs2)
    }

    pub fn is_incendiary(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(*self.controller + cs2.offsets.molotov.is_incendiary)
            != 0
    }
}
