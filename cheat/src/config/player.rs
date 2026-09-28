use egui::Color32;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use crate::cs2::key_codes::KeyCode;

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize, Display)]
pub enum DrawMode {
    None,
    Health,
    Color,
    PlayerColor,
}

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize, Display)]
pub enum SnaplineMode {
    None,
    Health,
    Distance,
    Color,
    PlayerColor,
}

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize, Display)]
pub enum SnaplineAnchor {
    Center,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize, Display)]
pub enum BoxMode {
    Gap,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Serialize, Deserialize, Display)]
pub enum ModelRenderMode {
    Filled,
    Wireframe,
}

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize, Display)]
pub enum VisibilityMode {
    All,
    InvisibleOnly,
    VisibleOnly,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerConfig {
    pub enabled: bool,
    pub chicken: bool,
    pub esp_hotkey: KeyCode,
    pub show_friendlies: bool,
    pub draw_box: DrawMode,
    pub box_mode: BoxMode,
    pub box_visible_color: Color32,
    pub box_invisible_color: Color32,
    pub snaplines: SnaplineMode,
    pub snapline_color: Color32,
    pub snapline_anchor: SnaplineAnchor,
    pub draw_skeleton: DrawMode,
    pub skeleton_color: Color32,
    pub draw_model: DrawMode,
    pub model_mode: ModelRenderMode,
    pub model_visible_color: Color32,
    pub model_invisible_color: Color32,
    pub head_circle: bool,
    pub health_bar: bool,
    pub armor_bar: bool,
    pub player_name: bool,
    pub weapon_icon: bool,
    pub tags: bool,
    pub visibility: VisibilityMode,
    pub sound: SoundConfig,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            chicken: true,
            esp_hotkey: KeyCode::X,
            show_friendlies: false,
            draw_box: DrawMode::Color,
            box_mode: BoxMode::Gap,
            box_visible_color: Color32::WHITE,
            box_invisible_color: Color32::RED,
            snaplines: SnaplineMode::None,
            snapline_color: Color32::PURPLE,
            snapline_anchor: SnaplineAnchor::Center,
            draw_skeleton: DrawMode::Health,
            skeleton_color: Color32::WHITE,
            draw_model: DrawMode::Health,
            model_mode: ModelRenderMode::Filled,
            model_visible_color: Color32::from_rgba_unmultiplied(255, 255, 255, 127),
            model_invisible_color: Color32::from_rgba_unmultiplied(255, 0, 0, 127),
            head_circle: true,
            health_bar: true,
            armor_bar: true,
            player_name: true,
            weapon_icon: true,
            tags: true,
            visibility: VisibilityMode::All,
            sound: SoundConfig::default(),
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
