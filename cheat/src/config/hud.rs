use egui::Color32;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use super::text::OverlayTextConfig;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HudConfig {
    pub bomb_timer: bool,
    pub hitmarker: HitmarkerConfig,
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
}

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            bomb_timer: true,
            hitmarker: HitmarkerConfig::default(),
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
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HitmarkerConfig {
    pub enabled: bool,
    pub duration: f32,
    pub line_length: f32,
    pub line_width: f32,
    pub gap: f32,
    pub color: Color32,
    pub kill_color: Color32,
    pub hit_sound_enabled: bool,
    pub kill_sound_enabled: bool,
    pub hit_sound: HitmarkerSound,
    pub kill_sound: HitmarkerSound,
    pub custom_sound_path: String,
    pub hit_volume: f32,
    pub kill_volume: f32,
}

impl Default for HitmarkerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: 0.15,
            line_length: 7.0,
            line_width: 2.0,
            gap: 5.0,
            color: Color32::WHITE,
            kill_color: Color32::RED,
            hit_sound_enabled: true,
            kill_sound_enabled: true,
            hit_sound: HitmarkerSound::Beep,
            kill_sound: HitmarkerSound::RubberTire,
            custom_sound_path: String::new(),
            hit_volume: 1.0,
            kill_volume: 1.0,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "PascalCase")]
pub enum HitmarkerSound {
    Beep,
    Bell,
    #[strum(serialize = "Bullet Casing")]
    BulletCasing,
    Click,
    Clink,
    #[strum(serialize = "Knife Impact")]
    KnifeImpact,
    #[strum(serialize = "Rubber Tire")]
    RubberTire,
    Sine,
    #[default]
    #[strum(serialize = "Water Drip")]
    WaterDrip,
    Wood,
    Custom,
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
