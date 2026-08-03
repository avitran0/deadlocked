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

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum BoxMode {
    Gap,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum VisibilityMode {
    All,
    #[serde(alias = "Visible Only")]
    VisibleOnly,
    #[serde(alias = "Invisible Only")]
    InvisibleOnly,
}

impl VisibilityMode {
    pub fn includes(self, visible: bool) -> bool {
        match self {
            Self::All => true,
            Self::VisibleOnly => visible,
            Self::InvisibleOnly => !visible,
        }
    }

    pub fn segment_range(self, start_visible: bool, end_visible: bool) -> Option<(f32, f32)> {
        if self == Self::All {
            return Some((0.0, 1.0));
        }

        match (self.includes(start_visible), self.includes(end_visible)) {
            (true, true) => Some((0.0, 1.0)),
            (true, false) => Some((0.0, 0.5)),
            (false, true) => Some((0.5, 1.0)),
            (false, false) => None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerConfig {
    pub enabled: bool,
    pub esp_hotkey: KeyCode,
    pub show_friendlies: bool,
    pub draw_box: DrawMode,
    pub box_mode: BoxMode,
    pub box_visible_color: Color32,
    pub box_invisible_color: Color32,
    pub draw_skeleton: DrawMode,
    pub skeleton_color: Color32,
    pub head_circle: bool,
    pub health_bar: bool,
    pub armor_bar: bool,
    pub player_name: bool,
    pub weapon_icon: bool,
    pub tags: bool,
    pub visibility_mode: VisibilityMode,
    pub sound: SoundConfig,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            esp_hotkey: KeyCode::X,
            show_friendlies: false,
            draw_box: DrawMode::Color,
            box_mode: BoxMode::Gap,
            box_visible_color: Color32::WHITE,
            box_invisible_color: Color32::RED,
            draw_skeleton: DrawMode::Health,
            skeleton_color: Color32::WHITE,
            head_circle: true,
            health_bar: true,
            armor_bar: true,
            player_name: true,
            weapon_icon: true,
            tags: true,
            visibility_mode: VisibilityMode::All,
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

#[cfg(test)]
mod tests {
    use super::VisibilityMode;

    #[derive(serde::Deserialize, serde::Serialize)]
    struct VisibilityConfig {
        mode: VisibilityMode,
    }

    #[test]
    fn visibility_modes_filter_players() {
        assert!(VisibilityMode::All.includes(true));
        assert!(VisibilityMode::All.includes(false));
        assert!(VisibilityMode::VisibleOnly.includes(true));
        assert!(!VisibilityMode::VisibleOnly.includes(false));
        assert!(!VisibilityMode::InvisibleOnly.includes(true));
        assert!(VisibilityMode::InvisibleOnly.includes(false));

        assert_eq!(
            VisibilityMode::VisibleOnly.segment_range(true, false),
            Some((0.0, 0.5))
        );
        assert_eq!(
            VisibilityMode::InvisibleOnly.segment_range(true, false),
            Some((0.5, 1.0))
        );
        assert_eq!(
            VisibilityMode::VisibleOnly.segment_range(false, false),
            None
        );
    }

    #[test]
    fn visibility_modes_accept_spaced_aliases() {
        let visible: VisibilityConfig = toml::from_str(r#"mode = "Visible Only""#).unwrap();
        let invisible: VisibilityConfig = toml::from_str(r#"mode = "Invisible Only""#).unwrap();

        assert_eq!(visible.mode, VisibilityMode::VisibleOnly);
        assert_eq!(invisible.mode, VisibilityMode::InvisibleOnly);
        assert!(
            toml::to_string(&invisible)
                .unwrap()
                .contains(r#"mode = "InvisibleOnly""#)
        );
    }
}
