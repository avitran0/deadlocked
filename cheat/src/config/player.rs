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

/// how the real player model mesh gets drawn (see ui::mesh); a real,
/// textured "full color" mode would need extracting and sampling each
/// agent's actual materials, not just geometry, which is a much bigger,
/// separate project than these
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

/// how Model ESP treats parts of the mesh a wall is actually blocking line
/// of sight to, per joint (not just the whole player at once)
#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum MeshPartVisibilityMode {
    /// no per-part distinction, same as before: only the whole player's
    /// visibility dims the entire mesh
    Off,
    /// additionally darkens whichever body parts are actually behind cover
    Shade,
    /// hides whichever body parts are actually behind cover entirely,
    /// leaving only the exposed silhouette rendered
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

/// every color value any ESP's `DrawMode` can resolve to. one definition
/// shared by box/skeleton/model/hitbox ESP: each either points at
/// `PlayerConfig::colors` (the shared/global set) or, if it enables its own
/// `_colors_override`, at its own separate instance of this same struct -
/// no separate one-off color fields duplicated per feature.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorValues {
    pub enemy_color: Color32,
    pub ally_color: Color32,
    pub visible_color: Color32,
    pub hidden_color: Color32,
    /// `Distance` mode's gradient: `near_color` at 0 units, `far_color` at
    /// `distance_max` units and beyond
    pub distance_near_color: Color32,
    pub distance_far_color: Color32,
    pub distance_max: f32,
    /// overrides whatever the mode above would otherwise produce for
    /// whoever is holding the bomb, while they're visible
    pub c4_carrier_highlight: bool,
    pub c4_carrier_color: Color32,
}

impl Default for ColorValues {
    fn default() -> Self {
        Self {
            enemy_color: Color32::from_rgb(255, 60, 60),
            ally_color: Color32::from_rgb(90, 170, 255),
            visible_color: Color32::from_rgb(255, 255, 255),
            hidden_color: Color32::from_rgb(120, 120, 120),
            distance_near_color: Color32::from_rgb(60, 255, 90),
            distance_far_color: Color32::from_rgb(255, 60, 60),
            distance_max: 2000.0,
            c4_carrier_highlight: true,
            c4_carrier_color: Color32::from_rgb(255, 200, 40),
        }
    }
}

impl ColorValues {
    /// resolves a `DrawMode` to an actual color using these values. the
    /// caller supplies its own health-based color for `None`/`Health` since
    /// each ESP feature computes that a little differently (e.g. chicken
    /// ESP always passes a flat 100/100 rather than a real health value).
    /// the bomb carrier's highlight overrides whatever the mode would
    /// otherwise produce, while they're visible.
    pub fn resolve(
        &self,
        mode: DrawMode,
        player: &PlayerData,
        data: &Data,
        health_color: Color32,
    ) -> Color32 {
        if self.c4_carrier_highlight && player.has_bomb && player.visible {
            return self.c4_carrier_color;
        }

        match mode {
            DrawMode::None | DrawMode::Health => health_color,
            DrawMode::Color => {
                if player.team == data.local_player.team {
                    self.ally_color
                } else {
                    self.enemy_color
                }
            }
            DrawMode::Visibility => {
                if player.visible {
                    self.visible_color
                } else {
                    self.hidden_color
                }
            }
            DrawMode::Distance => {
                let distance = data.local_player.position.distance(player.position);
                let t = (distance / self.distance_max.max(1.0)).clamp(0.0, 1.0);
                lerp_color(self.distance_near_color, self.distance_far_color, t)
            }
        }
    }
}

/// linearly interpolates two colors channel-by-channel, `t` clamped to
/// `[0, 1]`; used by `DrawMode::Distance`'s near/far gradient
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
    /// render style (off/highlight/wireframe/solid); the actual color comes
    /// from `model_color_mode` + `model_colors()` below, same as box/
    /// skeleton's own `DrawMode`
    pub draw_model: ModelEspMode,
    pub model_color_mode: DrawMode,
    pub model_part_visibility: MeshPartVisibilityMode,
    /// render style for Hitbox ESP's capsules, same options as Model ESP
    pub hitbox_esp: ModelEspMode,
    pub hitbox_color_mode: DrawMode,
    /// shared/global colors, used by any ESP below that hasn't enabled its
    /// own override
    pub colors: ColorValues,
    pub box_colors_override: bool,
    pub box_colors: ColorValues,
    pub skeleton_colors_override: bool,
    pub skeleton_colors: ColorValues,
    pub model_colors_override: bool,
    pub model_colors: ColorValues,
    /// tints the mesh's silhouette edge toward this color, blended in by
    /// the color's own alpha (0 = off, no edge tint at all)
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
            model_color_mode: DrawMode::Color,
            model_part_visibility: MeshPartVisibilityMode::Off,
            hitbox_esp: ModelEspMode::Off,
            hitbox_color_mode: DrawMode::Color,
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
