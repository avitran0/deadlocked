use crate::cs2::{CS2, entity::base_entity::BaseEntity};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PlantedC4 {
    entity: BaseEntity,
}

impl PlantedC4 {
    pub fn new(handle: usize) -> Self {
        Self {
            entity: BaseEntity::new(handle),
        }
    }

    pub fn is_relevant(&self, cs2: &CS2) -> bool {
        !self.has_exploded(cs2) && !self.is_defused(cs2)
    }

    pub fn has_exploded(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(*self.entity + cs2.offsets.planted_c4.has_exploded)
            != 0
    }

    pub fn is_defused(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(*self.entity + cs2.offsets.planted_c4.is_defused)
            != 0
    }

    pub fn is_planted(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(*self.entity + cs2.offsets.planted_c4.is_ticking)
            != 0
    }

    pub fn is_being_defused(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(*self.entity + cs2.offsets.planted_c4.being_defused)
            != 0
    }

    pub fn time_to_explosion(&self, cs2: &CS2) -> f32 {
        let blow_time: f32 = cs2
            .process
            .read(*self.entity + cs2.offsets.planted_c4.blow_time);
        (blow_time - cs2.current_time()).max(0.0)
    }

    pub fn position(&self, cs2: &CS2) -> glam::Vec3 {
        self.entity.position(cs2)
    }

    pub fn time_to_defuse(&self, cs2: &CS2) -> f32 {
        let defuse_time: f32 = cs2
            .process
            .read(*self.entity + cs2.offsets.planted_c4.defuse_time_left);
        (defuse_time - cs2.current_time()).max(0.0)
    }
}
