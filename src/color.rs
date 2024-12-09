#![allow(unused)]
use eframe::egui::Color32;

pub struct Colors;

impl Colors {
    pub const BACKDROP: Color32 = Color32::from_rgb(24, 24, 32);
    pub const BASE: Color32 = Color32::from_rgb(30, 30, 40);
    pub const HIGHLIGHT: Color32 = Color32::from_rgb(50, 50, 70);
    pub const SUBTEXT: Color32 = Color32::from_rgb(180, 180, 180);
    pub const TEXT: Color32 = Color32::from_rgb(255, 255, 255);
    pub const RED: Color32 = Color32::from_rgb(240, 100, 100);
    pub const ORANGE: Color32 = Color32::from_rgb(240, 140, 90);
    pub const YELLOW: Color32 = Color32::from_rgb(240, 200, 120);
    pub const GREEN: Color32 = Color32::from_rgb(160, 240, 130);
    pub const TEAL: Color32 = Color32::from_rgb(80, 200, 200);
    pub const BLUE: Color32 = Color32::from_rgb(100, 150, 240);
    pub const PURPLE: Color32 = Color32::from_rgb(180, 120, 240);
}
