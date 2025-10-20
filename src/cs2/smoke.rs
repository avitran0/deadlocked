use egui::{Color32, Rgba};
use glam::Vec3;
use serde::Serialize;

use crate::cs2::{CS2, entity::GrenadeInfo, player::Player};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Smoke {
    controller: u64,
}

impl Smoke {
    pub fn new(controller: u64) -> Self {
        Self { controller }
    }

    pub fn info(&self, cs2: &CS2) -> SmokeInfo {
        SmokeInfo {
            entity: self.controller,
            position: Player::entity(self.controller).position(cs2),
        }
    }

    pub fn disable(&self, cs2: &CS2) {
        let disabled = cs2
            .process
            .read::<u8>(self.controller + cs2.offsets.smoke.did_smoke_effect)
            != 0;
        if !disabled {
            cs2.process
                .write(self.controller + cs2.offsets.smoke.did_smoke_effect, true);
        }
    }

    pub fn color(&self, cs2: &CS2, color: &Color32) {
        let offset = self.controller + cs2.offsets.smoke.smoke_color;
        let current_color: [f32; 3] = cs2.process.read(offset);
        let color = Rgba::from(*color);
        let wanted_color = [color.r() * 255.0, color.g() * 255.0, color.b() * 255.0];
        if current_color != wanted_color {
            cs2.process.write(offset, wanted_color);
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SmokeInfo {
    pub entity: u64,
    pub position: Vec3,
}

impl SmokeInfo {
    pub fn grenade(&self) -> GrenadeInfo {
        GrenadeInfo {
            entity: self.entity,
            position: self.position,
            name: "Smoke",
        }
    }
}
