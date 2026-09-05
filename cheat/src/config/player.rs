use egui::Color32;
use serde::{Deserialize, Serialize};
use shared::{Data, PlayerData};
use strum::EnumIter;

use crate::cs2::key_codes::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum DrawMode {
    None,
    Health,
    Color,
    Distance,
    Visibility,
}

impl std::fmt::Display for DrawMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => "None",
            Self::Health => "Health",
            Self::Color => "Color",
            Self::Distance => "Distance",
            Self::Visibility => "Visibility",
        }
        .fmt(f)
    }
}

/// same as `DrawMode` minus `None`; Model/Hitbox ESP have their own
/// separate on/off switch already, so it'd just duplicate `Health`
#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum MeshColorMode {
    Health,
    Color,
    Distance,
    Visibility,
}

impl std::fmt::Display for MeshColorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Health => "Health",
            Self::Color => "Color",
            Self::Distance => "Distance",
            Self::Visibility => "Visibility",
        }
        .fmt(f)
    }
}

impl From<MeshColorMode> for DrawMode {
    fn from(mode: MeshColorMode) -> Self {
        match mode {
            MeshColorMode::Health => DrawMode::Health,
            MeshColorMode::Color => DrawMode::Color,
            MeshColorMode::Distance => DrawMode::Distance,
            MeshColorMode::Visibility => DrawMode::Visibility,
        }
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

/// how the player model mesh gets drawn (see ui::mesh)
#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum ModelEspMode {
    Off,
    Highlight,
    Wireframe,
    Solid,
}

impl std::fmt::Display for ModelEspMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => "Off",
            Self::Highlight => "Highlight",
            Self::Wireframe => "Wireframe",
            Self::Solid => "Solid",
        }
        .fmt(f)
    }
}

/// per-joint mesh visibility against cover, not just the whole player
#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum MeshPartVisibilityMode {
    Off,
    /// darkens body parts behind cover
    Shade,
    /// hides body parts behind cover entirely
    Cull,
}

impl std::fmt::Display for MeshPartVisibilityMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => "Off",
            Self::Shade => "Shade",
            Self::Cull => "Cull",
        }
        .fmt(f)
    }
}

/// color set shared by box/skeleton/model/hitbox ESP, or overridden per-feature
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorValues {
    pub enemy_color: Color32,
    pub ally_color: Color32,
    /// `Distance` mode gradient: `near_color` at 0 units to `far_color` at `distance_max`
    pub distance_near_color: Color32,
    pub distance_far_color: Color32,
    pub distance_max: f32,
    /// overrides the mode above for the bomb carrier, visible or not
    pub c4_carrier_highlight: bool,
    pub c4_carrier_color: Color32,
}

impl Default for ColorValues {
    fn default() -> Self {
        Self {
            enemy_color: Color32::from_rgb(255, 60, 60),
            ally_color: Color32::from_rgb(90, 170, 255),
            distance_near_color: Color32::from_rgb(60, 255, 90),
            distance_far_color: Color32::from_rgb(255, 60, 60),
            distance_max: 2000.0,
            c4_carrier_highlight: true,
            c4_carrier_color: Color32::from_rgb(255, 200, 40),
        }
    }
}

impl ColorValues {
    /// resolves a `DrawMode` to a color; `health_color` is the caller's own
    /// health-based color for `None`/`Health`
    pub fn resolve(
        &self,
        mode: DrawMode,
        player: &PlayerData,
        data: &Data,
        health_color: Color32,
    ) -> Color32 {
        if self.c4_carrier_highlight && player.has_bomb {
            return self.c4_carrier_color;
        }

        match mode {
            DrawMode::None | DrawMode::Health => health_color,
            DrawMode::Color => self.team_color(player, data),
            // same team color, dimmed while not visible
            DrawMode::Visibility => {
                let color = self.team_color(player, data);
                if player.visible {
                    color
                } else {
                    shade_color(color, 0.4)
                }
            }
            DrawMode::Distance => {
                let distance = data.local_player.position.distance(player.position);
                let t = (distance / self.distance_max.max(1.0)).clamp(0.0, 1.0);
                lerp_color(self.distance_near_color, self.distance_far_color, t)
            }
        }
    }

    fn team_color(&self, player: &PlayerData, data: &Data) -> Color32 {
        if player.team == data.local_player.team {
            self.ally_color
        } else {
            self.enemy_color
        }
    }
}

/// linearly interpolates two colors, `t` clamped to `[0, 1]`
fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let channel = |from: u8, to: u8| (from as f32 + (to as f32 - from as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        channel(a.r(), b.r()),
        channel(a.g(), b.g()),
        channel(a.b(), b.b()),
        channel(a.a(), b.a()),
    )
}

/// darkens a color's rgb by `factor` (0 = black, 1 = unchanged), alpha untouched
fn shade_color(color: Color32, factor: f32) -> Color32 {
    let factor = factor.clamp(0.0, 1.0);
    let channel = |c: u8| (c as f32 * factor).round() as u8;
    Color32::from_rgba_unmultiplied(
        channel(color.r()),
        channel(color.g()),
        channel(color.b()),
        color.a(),
    )
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
    pub draw_skeleton: DrawMode,
    /// render style; color comes from `model_color_mode` below instead
    pub draw_model: ModelEspMode,
    pub model_color_mode: MeshColorMode,
    pub model_part_visibility: MeshPartVisibilityMode,
    pub hitbox_esp: ModelEspMode,
    pub hitbox_color_mode: MeshColorMode,
    /// used by any ESP below that hasn't enabled its own override
    pub colors: ColorValues,
    pub box_colors_override: bool,
    pub box_colors: ColorValues,
    pub skeleton_colors_override: bool,
    pub skeleton_colors: ColorValues,
    pub model_colors_override: bool,
    pub model_colors: ColorValues,
    /// tints the mesh's silhouette edge; alpha 0 = off
    pub model_outline_color: Color32,
    pub hitbox_colors_override: bool,
    pub hitbox_colors: ColorValues,
    pub head_circle: bool,
    pub health_bar: bool,
    pub armor_bar: bool,
    pub player_name: bool,
    pub weapon_icon: bool,
    pub tags: bool,
    pub visible_only: bool,
    pub sound: SoundConfig,
}

impl PlayerConfig {
    pub fn box_colors(&self) -> &ColorValues {
        if self.box_colors_override {
            &self.box_colors
        } else {
            &self.colors
        }
    }

    pub fn skeleton_colors(&self) -> &ColorValues {
        if self.skeleton_colors_override {
            &self.skeleton_colors
        } else {
            &self.colors
        }
    }

    pub fn model_colors(&self) -> &ColorValues {
        if self.model_colors_override {
            &self.model_colors
        } else {
            &self.colors
        }
    }

    pub fn hitbox_colors(&self) -> &ColorValues {
        if self.hitbox_colors_override {
            &self.hitbox_colors
        } else {
            &self.colors
        }
    }
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
            draw_skeleton: DrawMode::Health,
            draw_model: ModelEspMode::Off,
            model_color_mode: MeshColorMode::Color,
            model_part_visibility: MeshPartVisibilityMode::Off,
            hitbox_esp: ModelEspMode::Off,
            hitbox_color_mode: MeshColorMode::Color,
            colors: ColorValues::default(),
            box_colors_override: false,
            box_colors: ColorValues::default(),
            skeleton_colors_override: false,
            skeleton_colors: ColorValues::default(),
            model_colors_override: false,
            model_colors: ColorValues::default(),
            model_outline_color: Color32::TRANSPARENT,
            hitbox_colors_override: false,
            hitbox_colors: ColorValues::default(),
            head_circle: true,
            health_bar: true,
            armor_bar: true,
            player_name: true,
            weapon_icon: true,
            tags: true,
            visible_only: false,
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
