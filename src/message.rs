use std::fmt::Display;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config::Config, mouse::DeviceStatus};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameStatus {
    Working,
    GameNotStarted,
}

impl Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::Working => write!(f, "Working"),
            GameStatus::GameNotStarted => write!(f, "Game Not Started"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RadarStatus {
    Disconnected,
    Connected(Uuid),
}

impl Display for RadarStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RadarStatus::Connected(_) => write!(f, "Connected"),
            RadarStatus::Disconnected => write!(f, "Disconnected"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Config(Box<Config>),
    GameStatus(GameStatus),
    MouseStatus(DeviceStatus),
    RadarStatus(RadarStatus),
    ChangeRadarUrl(String),
    RadarSetEnabled(bool),
    SelectMouse(String),
}

#[derive(Clone, Debug)]
pub enum Target {
    Gui,
    Game,
    Radar,
}

#[derive(Clone, Debug)]
pub struct Envelope {
    pub target: Target,
    pub message: Message,
}
