use std::ops::Range;

use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{config::AimbotStatus, key_codes::KeyCode, mouse::MouseStatus};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, EnumIter)]
pub enum Game {
    CS2,
    Deadlock,
}

impl Game {
    pub fn string(&self) -> &str {
        match self {
            Game::CS2 => "CS2",
            Game::Deadlock => "Deadlock",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    ConfigEnableAimbot(bool),
    ConfigHotkey(KeyCode),
    ConfigStartBullet(i32),
    ConfigAimLock(bool),
    ConfigVisibilityCheck(bool),
    ConfigFOV(f32),
    ConfigSmooth(f32),
    ConfigMultibone(bool),
    ConfigEnableRCS(bool),
    ConfigEnableTriggerbot(bool),
    ConfigTriggerbotHotkey(KeyCode),
    ConfigTriggerbotRange(Range<u32>),
    Status(AimbotStatus),
    ChangeGame(Game),
    MouseStatus(MouseStatus),
    FrameTime(f64),
}
