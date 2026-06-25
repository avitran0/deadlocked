use std::{fmt::Display, time::Duration};

use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct GameMessage(pub Box<Config>);

#[derive(Debug, Clone)]
pub enum UiMessage {
    Status(GameStatus),
    FrameTime(Duration),
}
