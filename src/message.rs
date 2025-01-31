use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{
    config::{AimbotConfig, AimbotStatus},
    mouse::MouseStatus,
};

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
    Config(AimbotConfig),
    Status(AimbotStatus),
    ChangeGame(Game),
    MouseStatus(MouseStatus),
    FrameTime(f64),
}
