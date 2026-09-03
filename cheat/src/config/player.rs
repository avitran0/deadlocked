use egui::Color32;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::cs2::key_codes::KeyCode;

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum DrawMode {
    None,
    Health,
    Color,
}

impl std::fmt::Display for DrawMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => "None",
            Self::Health => "Health",
            Self::Color => "Color",
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum TracersMode {
    Health,
    Distance,
    Color
}

impl std::fmt::Display for TracersMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Health => "Health",
            Self::Distance => "Distance",
            Self::Color => "Color",
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum BoxMode {
    Gap,
    Full,
}

impl std::fmt::Display for BoxMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gap => "Gap",
            Self::Full => "Full",
        }
        .fmt(f)
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerConfig {
    pub enabled: bool,
    pub chicken: bool,
    pub esp_hotkey: KeyCode,
    pub tracers: bool,
    pub show_friendlies: bool,
    pub draw_box: DrawMode,
    pub box_mode: BoxMode,
    pub tracers_mode: TracersMode,
    pub box_visible_color: Color32,
    pub box_invisible_color: Color32,
    pub tracers_color: Color32,
    pub draw_skeleton: DrawMode,
    pub skeleton_color: Color32,
    pub head_circle: bool,
    pub health_bar: bool,
    pub armor_bar: bool,
    pub player_name: bool,
    pub weapon_icon: bool,
    pub tags: bool,
    pub visible_only: bool,
    pub sound: SoundConfig,

    pub max_tracers_dd_distance: i32,
    pub tracers_y_value: f32
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            chicken: true,
            esp_hotkey: KeyCode::X,
            tracers: false,
            show_friendlies: false,
            draw_box: DrawMode::Color,
            box_mode: BoxMode::Gap,
            tracers_mode: TracersMode::Color,
            box_visible_color: Color32::WHITE,
            box_invisible_color: Color32::RED,
            tracers_color: Color32::PURPLE,
            draw_skeleton: DrawMode::Health,
            skeleton_color: Color32::WHITE,
            head_circle: true,
            health_bar: true,
            armor_bar: true,
            player_name: true,
            weapon_icon: true,
            tags: true,
            visible_only: false,
            sound: SoundConfig::default(),

            max_tracers_dd_distance: 3000,
            tracers_y_value: 75.0
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SoundConfig {
    pub enabled: bool,
    pub footstep_diameter: f32,
    pub gunshot_diameter: f32,
    pub weapon_diameter: f32,
    pub fadeout_start: f32,
    pub fadeout_duration: f32,
    pub show_visible: bool,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            footstep_diameter: crate::constants::cs2::SOUND_ESP_FOOTSTEP_DIAMETER_DEFAULT,
            gunshot_diameter: crate::constants::cs2::SOUND_ESP_GUNSHOT_DIAMETER_DEFAULT,
            weapon_diameter: crate::constants::cs2::SOUND_ESP_WEAPON_DIAMETER_DEFAULT,
            fadeout_start: 1.0,
            fadeout_duration: 1.0,
            show_visible: true,
        }
    }
}
