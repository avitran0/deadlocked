use egui::Color32;
use serde::{Deserialize, Serialize};

use super::text::{OverlayTextConfig, TextPosition};

#[derive(Clone, Serialize, Deserialize)]
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
    pub line_width: f32,
    pub debug: bool,
    pub overlay_text: OverlayTextConfig,
    pub minimap: MinimapConfig,
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
            line_width: 2.0,
            debug: false,
            overlay_text: OverlayTextConfig::default(),
            minimap: MinimapConfig::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MinimapConfig {
    pub enabled: bool,
    pub size: f32,
    pub range: f32,
    pub position: TextPosition,
    pub opacity: f32,
    pub icon_size: f32,
    pub rotate_with_view: bool,
    pub show_names: bool,
    pub show_bomb: bool,
    pub show_teammates: bool,
    pub show_smoke: bool,
    pub show_molotov: bool,
    pub show_rings: bool,
    pub clamp_to_edge: bool,
    pub background: Color32,
    pub enemy_color: Color32,
    pub teammate_color: Color32,
    pub bomb_color: Color32,
    pub local_color: Color32,
    pub smoke_color: Color32,
    pub molotov_color: Color32,
}

impl Default for MinimapConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            size: 220.0,
            range: 2500.0,
            position: TextPosition::TopRight,
            opacity: 0.82,
            icon_size: 7.0,
            rotate_with_view: true,
            show_names: false,
            show_bomb: true,
            show_teammates: true,
            show_smoke: true,
            show_molotov: true,
            show_rings: true,
            clamp_to_edge: true,
            background: Color32::from_rgba_unmultiplied(6, 10, 16, 210),
            enemy_color: Color32::from_rgb(255, 72, 72),
            teammate_color: Color32::from_rgb(80, 196, 255),
            bomb_color: Color32::from_rgb(255, 196, 48),
            local_color: Color32::from_rgb(255, 255, 255),
            smoke_color: Color32::from_rgb(170, 180, 190),
            molotov_color: Color32::from_rgb(255, 110, 40),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TrailConfig {
    pub enabled: bool,
    pub inferno_poly: bool,
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
            inferno_poly: true,
            smoke: Color32::LIGHT_GRAY,
            molotov: Color32::RED,
            incendiary: Color32::ORANGE,
            flash: Color32::WHITE,
            he: Color32::DARK_GRAY,
            decoy: Color32::PURPLE,
        }
    }
}
