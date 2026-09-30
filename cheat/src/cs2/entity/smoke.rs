use crate::cs2::{
    CS2,
    entity::{GrenadeInfo, base_entity::BaseEntity, grenade_info},
};
use egui::{Color32, Rgba};

#[derive(Clone, PartialEq)]
pub struct Smoke {
    entity: BaseEntity,
}

impl Smoke {
    pub fn new(entity: usize) -> Self {
        Self {
            entity: BaseEntity::new(entity),
        }
    }

    pub fn info(&self, cs2: &CS2) -> GrenadeInfo {
        grenade_info(self.entity, "Smoke", cs2)
    }

    pub fn disable(&self, cs2: &CS2) {
        let disabled = cs2
            .process
            .read::<u8>(*self.entity + cs2.offsets.smoke.did_smoke_effect)
            != 0;
        if !disabled {
            cs2.process
                .write(*self.entity + cs2.offsets.smoke.did_smoke_effect, 1u8);
        }
    }

    pub fn color(&self, cs2: &CS2, color: &Color32) {
        let offset = *self.entity + cs2.offsets.smoke.smoke_color;
        let current_color: [f32; 3] = cs2.process.read(offset);
        let color = Rgba::from(*color);
        let wanted_color = [color.r() * 255.0, color.g() * 255.0, color.b() * 255.0];
        if current_color != wanted_color {
            cs2.process.write(offset, wanted_color);
        }
    }
}
