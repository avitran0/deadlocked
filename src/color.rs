#![allow(unused)]
use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Color {
    r: u8,
    b: u8,
    g: u8,
    a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_egui(color: &Color32) -> Self {
        Self {
            r: color.r(),
            g: color.g(),
            b: color.b(),
            a: color.a(),
        }
    }

    pub const fn egui_color(&self) -> Color32 {
        Color32::from_rgba_premultiplied(self.r, self.g, self.b, self.a)
    }

    pub const fn to_hex(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.b as u32) << 16) | ((self.g as u32) << 8) | (self.r as u32)
    }
}

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
