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
    pub gui_always_render: bool,
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
            gui_always_render: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MinimapConfig {
    pub enabled: bool,
    pub position: TextPosition,
    pub size: f32,
    pub zoom: f32,
    pub center_on_self: bool,
    pub marker_size: f32,
    pub hide_native_minimap: bool,
    pub bomb_carrier_gold: bool,
}

impl Default for MinimapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            position: TextPosition::TopLeft,
            size: 320.0,
            zoom: 1.0,
            center_on_self: false,
            marker_size: 8.0,
            hide_native_minimap: false,
            bomb_carrier_gold: true,
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
