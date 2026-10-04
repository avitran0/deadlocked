use egui::Color32;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use super::text::OverlayTextConfig;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HudConfig {
    pub bomb_options: BombTimerConfig,
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
    pub bvh_debug: BvhDebugConfig,
    pub overlay_text: OverlayTextConfig,
}

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            bomb_options: BombTimerConfig::default(),
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
            bvh_debug: BvhDebugConfig::default(),
            overlay_text: OverlayTextConfig::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BvhDebugConfig {
    pub enabled: bool,
    pub visible_only: bool,
    pub range: f32,
}

impl Default for BvhDebugConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            visible_only: true,
            range: 500.0,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "PascalCase")]
pub enum BombRenderMode {
    #[default]
    Default,
    Gradient,
    #[strum(serialize = "Custom Colors")]
    CustomColors,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BombTimerConfig {
    pub enabled: bool,
    pub render_mode: BombRenderMode,
    pub not_defusable_color: Color32,
    pub defusable_with_kit_color: Color32,
    pub defusable_without_kit_color: Color32,
    pub successfully_defused_color: Color32,
    pub defuse_failed_color: Color32,
}
impl Default for BombTimerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            render_mode: BombRenderMode::Default,
            not_defusable_color: Color32::RED,
            defusable_with_kit_color: Color32::YELLOW,
            defusable_without_kit_color: Color32::CYAN,
            successfully_defused_color: Color32::GREEN,
            defuse_failed_color: Color32::RED,
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
    pub hit_custom_sound_path: String,
    pub kill_custom_sound_path: String,
    pub hit_volume: f32,
    pub kill_volume: f32,
}

impl Default for HitmarkerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: 0.5,
            line_length: 7.0,
            line_width: 2.0,
            gap: 5.0,
            color: Color32::WHITE,
            kill_color: Color32::RED,
            hit_sound_enabled: true,
            kill_sound_enabled: true,
            hit_sound: HitmarkerSound::Beep,
            kill_sound: HitmarkerSound::RubberTire,
            hit_custom_sound_path: String::new(),
            kill_custom_sound_path: String::new(),
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
            smoke: Color32::GREEN,
            molotov: Color32::LIGHT_RED,
            incendiary: Color32::LIGHT_RED,
            flash: Color32::CYAN,
            he: Color32::RED,
            decoy: Color32::PURPLE,
        }
    }
}
