use glam::{Vec4, Mat4, Vec3};

use crate::{config::{AimbotConfig, AimbotStatus}, mouse::MouseStatus};

#[derive(Clone, Debug)]
pub enum Message {
    AimbotConfig(AimbotConfig),
    Status(AimbotStatus),
    MouseStatus(MouseStatus),
    FrameTime(f64),
    PlayerInfo(Vec<PlayerInfo>),
    GameInfo((Mat4, Vec4)),
}

#[derive(Clone, Debug, Default)]
pub struct PlayerInfo {
    pub health: i32,
    pub armor: i32,
    pub position: Vec3,
    pub head: Vec3,
    pub bones: Vec<(Vec3, Vec3)>,
}
