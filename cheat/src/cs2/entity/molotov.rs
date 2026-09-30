use shared::MolotovInfo;

use crate::cs2::{CS2, entity::base_entity::BaseEntity};

#[derive(Clone, PartialEq)]
pub struct Molotov {
    entity: BaseEntity,
}

impl Molotov {
    pub fn new(entity: usize) -> Self {
        Self {
            entity: BaseEntity::new(entity),
        }
    }

    pub fn info(&self, cs2: &CS2) -> MolotovInfo {
        MolotovInfo {
            entity: *self.entity,
            position: self.entity.position(cs2),
            is_incendiary: self.is_incendiary(cs2),
        }
    }

    pub fn is_incendiary(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(*self.entity + cs2.offsets.molotov.is_incendiary)
            != 0
    }
}
