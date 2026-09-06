#![allow(unused)]
use egui::Color32;
use serde::{Deserialize, Serialize};

pub struct Colors;

impl Colors {
    pub const BACKDROP: Color32 = Color32::from_rgb(24, 24, 28);
    pub const BASE: Color32 = Color32::from_rgb(30, 30, 35);
    pub const HIGHLIGHT: Color32 = Color32::from_rgb(50, 50, 60);
    pub const SUBTEXT: Color32 = Color32::from_rgb(140, 140, 140);
    pub const TEXT: Color32 = Color32::from_rgb(255, 255, 255);
    pub const RED: Color32 = Color32::from_rgb(240, 100, 100);
    pub const ORANGE: Color32 = Color32::from_rgb(240, 140, 90);
    pub const YELLOW: Color32 = Color32::from_rgb(240, 200, 120);
    pub const GREEN: Color32 = Color32::from_rgb(160, 240, 130);
    pub const TEAL: Color32 = Color32::from_rgb(80, 200, 200);
    pub const BLUE: Color32 = Color32::from_rgb(100, 150, 240);
    pub const PURPLE: Color32 = Color32::from_rgb(180, 120, 240);

    pub const ACCENT_COLORS: [(&str, Color32); 7] = [
        ("Red", Self::RED),
        ("Orange", Self::ORANGE),
        ("Yellow", Self::YELLOW),
        ("Green", Self::GREEN),
        ("Teal", Self::TEAL),
        ("Blue", Self::BLUE),
        ("Purple", Self::PURPLE),
    ];
}

/// red at 0 health, green at full, yellow in between
pub fn health_color(health: i32, max_health: i32, alpha: u8) -> Color32 {
    let max_health = max_health.max(1);
    let health = health.clamp(0, max_health);
    let percent = health as f32 / max_health as f32;

    let (r, g) = if percent <= 0.5 {
        let factor = percent * 2.0;
        (255, (255.0 * factor) as u8)
    } else {
        let factor = 1.0 - (percent - 0.5) * 2.0;
        ((255.0 * factor) as u8, 255)
    };

    Color32::from_rgba_unmultiplied(r, g, 0, alpha)
}
