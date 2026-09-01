use std::{fmt::Display, time::Duration};

use serde::{Deserialize, Serialize};

use crate::config::Config;

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
pub struct GameMessage(pub Box<Config>);

#[derive(Clone)]
pub enum UiMessage {
    Status(GameStatus),
    FrameTime(Duration),
}

#[derive(Clone)]
pub enum RadarMessage {
    Enabled(bool),
    SetUrl(String),
}
