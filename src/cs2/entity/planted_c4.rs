use glam::Vec3;

use crate::cs2::{CS2, entity::player::Player};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlantedC4 {
    handle: u64,
}

impl PlantedC4 {
    pub fn new(handle: u64) -> Self {
        Self { handle }
    }

    pub fn is_relevant(&self, cs2: &CS2) -> bool {
        !self.has_exploded(cs2) && !self.is_defused(cs2)
    }

    pub fn has_exploded(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(self.handle + cs2.offsets.planted_c4.has_exploded)
            != 0
    }

    pub fn is_defused(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(self.handle + cs2.offsets.planted_c4.is_defused)
            != 0
    }

    pub fn is_planted(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(self.handle + cs2.offsets.planted_c4.is_ticking)
            != 0
    }

    pub fn is_being_defused(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(self.handle + cs2.offsets.planted_c4.being_defused)
            != 0
    }

    pub fn time_to_explosion(&self, cs2: &CS2) -> f32 {
        let blow_time: f32 = cs2
            .process
            .read(self.handle + cs2.offsets.planted_c4.blow_time);

        blow_time - cs2.current_time()
    }

    pub fn position(&self, cs2: &CS2) -> Vec3 {
        let planted_c4 = Player::pawn(self.handle);
        planted_c4.position(cs2)
    }

    pub fn time_to_defuse(&self, cs2: &CS2) -> f32 {
        let defuse_time_left: f32 = cs2
            .process
            .read(self.handle + cs2.offsets.planted_c4.defuse_time_left);
        defuse_time_left - cs2.current_time()
    }
}
