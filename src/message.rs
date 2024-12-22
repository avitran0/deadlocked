use glam::{IVec4, Mat4, Vec3};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{
    color::Color,
    config::{AimbotStatus, VisualsConfig},
    key_codes::KeyCode,
    mouse::MouseStatus,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, EnumIter)]
pub enum Game {
    CS2,
    Deadlock,
}

impl Game {
    #[allow(unused)]
    pub fn lower_string(&self) -> &str {
        match self {
            Game::CS2 => "cs2",
            Game::Deadlock => "deadlock",
        }
    }

    pub fn string(&self) -> &str {
        match self {
            Game::CS2 => "CS2",
            Game::Deadlock => "Deadlock",
        }
    }
}

#[derive(Clone, Debug)]
pub enum AimbotMessage {
    ConfigEnableAimbot(bool),
    ConfigHotkey(KeyCode),
    ConfigStartBullet(i32),
    ConfigAimLock(bool),
    ConfigVisibilityCheck(bool),
    ConfigFOV(f32),
    ConfigSmooth(f32),
    ConfigMultibone(bool),
    ConfigEnableRCS(bool),
    Status(AimbotStatus),
    ChangeGame(Game),
    MouseStatus(MouseStatus),
    FrameTime(f64),
    Quit,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum DrawStyle {
    #[default]
    None,
    Color,
    Health,
}

#[derive(Clone, Debug)]
pub enum VisualsMessage {
    PlayerInfo(Vec<PlayerInfo>),
    EntityInfo(Vec<EntityInfo>),
    ViewMatrix(Mat4),
    WindowSize(IVec4),
    EnableVisuals(bool),
    DrawBox(DrawStyle),
    BoxColor(Color),
    DrawSkeleton(DrawStyle),
    SkeletonColor(Color),
    DrawHealth(bool),
    DrawArmor(bool),
    ArmorColor(Color),
    DrawWeapon(bool),
    DroppedItems(bool),
    VisibilityCheck(bool),

    VisualsFps(u64),
    DebugWindow(bool),
    Config(VisualsConfig),
    Quit,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerInfo {
    pub health: i32,
    pub armor: i32,
    pub weapon: String,
    pub position: Vec3,
    pub head: Vec3,
    pub bones: Vec<(Vec3, Vec3)>,
    pub visible: bool,
}

#[derive(Clone, Debug, Default)]
pub struct EntityInfo {
    pub name: String,
    pub position: Vec3,
    pub distance: f32,
}
