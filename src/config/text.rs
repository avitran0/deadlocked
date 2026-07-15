use egui::{Align2, Color32};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::ui::color::Colors;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter)]
pub enum TextPosition {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter)]
pub enum TextAlign {
    LeftTop,
    LeftCenter,
    LeftBottom,
    CenterTop,
    CenterCenter,
    CenterBottom,
    RightTop,
    RightCenter,
    RightBottom,
}

impl TextAlign {
    pub fn to_align2(self) -> Align2 {
        match self {
            TextAlign::LeftTop => Align2::LEFT_TOP,
            TextAlign::LeftCenter => Align2::LEFT_CENTER,
            TextAlign::LeftBottom => Align2::LEFT_BOTTOM,
            TextAlign::CenterTop => Align2::CENTER_TOP,
            TextAlign::CenterCenter => Align2::CENTER_CENTER,
            TextAlign::CenterBottom => Align2::CENTER_BOTTOM,
            TextAlign::RightTop => Align2::RIGHT_TOP,
            TextAlign::RightCenter => Align2::RIGHT_CENTER,
            TextAlign::RightBottom => Align2::RIGHT_BOTTOM,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TextCategory {
    pub font_size: f32,
    pub color: Color32,
    pub position: TextPosition,
    pub align: TextAlign,
}

impl Default for TextCategory {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            color: Colors::TEXT,
            position: TextPosition::Center,
            align: TextAlign::CenterCenter,
        }
    }
}

impl TextCategory {
    pub fn new(font_size: f32, color: Color32, position: TextPosition, align: TextAlign) -> Self {
        Self {
            font_size,
            color,
            position,
            align,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OverlayTextConfig {
    pub status_text: TextCategory,
    pub player_name: TextCategory,
    pub player_tags: TextCategory,
    pub weapon_icon: TextCategory,
    pub ammo_text: TextCategory,
    pub weapon_name: TextCategory,
    pub bomb_timer: TextCategory,
    pub grenade_name: TextCategory,
    pub grenade_lineup: TextCategory,
    pub keybind_list: TextCategory,
    pub spectator_list: TextCategory,
}

impl Default for OverlayTextConfig {
    fn default() -> Self {
        Self {
            status_text: TextCategory::new(
                16.0,
                Colors::TEXT,
                TextPosition::CenterRight,
                TextAlign::LeftTop,
            ),
            player_name: TextCategory::new(
                14.0,
                Colors::TEXT,
                TextPosition::TopRight,
                TextAlign::LeftTop,
            ),
            player_tags: TextCategory::new(
                20.0,
                Colors::TEXT,
                TextPosition::TopRight,
                TextAlign::LeftTop,
            ),
            weapon_icon: TextCategory::new(
                20.0,
                Colors::TEXT,
                TextPosition::BottomCenter,
                TextAlign::CenterTop,
            ),
            ammo_text: TextCategory::new(
                16.0,
                Colors::TEXT,
                TextPosition::BottomCenter,
                TextAlign::CenterTop,
            ),
            weapon_name: TextCategory::new(
                16.0,
                Colors::TEXT,
                TextPosition::Center,
                TextAlign::CenterCenter,
            ),
            bomb_timer: TextCategory::new(
                24.0,
                Colors::TEXT,
                TextPosition::Center,
                TextAlign::CenterCenter,
            ),
            grenade_name: TextCategory::new(
                16.0,
                Colors::TEXT,
                TextPosition::Center,
                TextAlign::CenterCenter,
            ),
            grenade_lineup: TextCategory::new(
                14.0,
                Colors::TEXT,
                TextPosition::Center,
                TextAlign::CenterTop,
            ),
            keybind_list: TextCategory::new(
                16.0,
                Colors::TEXT,
                TextPosition::CenterLeft,
                TextAlign::LeftTop,
            ),
            spectator_list: TextCategory::new(
                16.0,
                Colors::TEXT,
                TextPosition::CenterLeft,
                TextAlign::LeftTop,
            ),
        }
    }
}
