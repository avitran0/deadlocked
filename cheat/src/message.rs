use std::{fmt::Display, time::Duration};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::{Config, radar::RadarConfig},
    ui::grenades::GrenadeList,
};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum GameStatus {
    Working,
    NotStarted,
}

impl Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::Working => write!(f, "Working"),
            GameStatus::NotStarted => write!(f, "Not Started"),
        }
    }
}

#[derive(Clone)]
pub enum GameMessage {
    Config(Box<Config>),
    Grenades(Box<GrenadeList>),
}

#[derive(Clone)]
pub enum UiMessage {
    Status(GameStatus),
    FrameTime(Duration),
}

#[derive(Clone)]
pub enum RadarMessage {
    Config { config: RadarConfig, uuid: Uuid },
}

#[derive(Clone)]
pub enum RadarStatus {
    Connected,
    FailedToConnect,
    Disconnected,
    Disabled,
}
