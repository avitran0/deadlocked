use egui::Color32;
use serde::{Deserialize, Serialize};

use crate::ui::color::Colors;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HudConfig {
    pub bomb_timer: bool,
    pub fov_circle: bool,
    pub sniper_crosshair: CrosshairConfig,
    pub dropped_weapons: bool,
    pub keybind_list: bool,
    pub spectator_list: bool,
    pub grenade_trails: TrailConfig,
    pub text_outline: bool,
    pub text_color: Color32,
    pub line_width: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub debug: bool,
}

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            bomb_timer: true,
            fov_circle: false,
            sniper_crosshair: CrosshairConfig::default(),
            dropped_weapons: true,
            keybind_list: false,
            spectator_list: false,
            grenade_trails: TrailConfig::default(),
            text_outline: true,
            text_color: Colors::TEXT,
            line_width: 2.0,
            font_size: 16.0,
            icon_size: 20.0,
            debug: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CrosshairConfig {
    pub enabled: bool,
    pub color: Color32,
    pub line_length: f32,
    pub line_width: f32,
    pub gap: f32,
}

impl Default for CrosshairConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            color: Color32::WHITE,
            line_length: 50.0,
            line_width: 2.0,
            gap: 20.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TrailConfig {
    pub enabled: bool,
    pub smoke: Color32,
    pub molotov: Color32,
    pub incendiary: Color32,
    pub flash: Color32,
    pub he: Color32,
    pub decoy: Color32,
}

impl Default for TrailConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            smoke: Color32::LIGHT_GRAY,
            molotov: Color32::RED,
            incendiary: Color32::ORANGE,
            flash: Color32::WHITE,
            he: Color32::DARK_GRAY,
            decoy: Color32::PURPLE,
        }
    }
}
