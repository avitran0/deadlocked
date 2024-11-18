use std::{collections::HashMap, fs::read_to_string, time::Duration};

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{key_codes::KeyCode, message::Game};

pub const DEBUG_WITHOUT_MOUSE: bool = false;

const REFRESH_RATE: u64 = 100;
pub const LOOP_DURATION: Duration = Duration::from_millis(1000 / REFRESH_RATE);
pub const SLEEP_DURATION: Duration = Duration::from_secs(1);
pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const ZOOM: f32 = 1.5;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AimbotStatus {
    Working,
    GameNotStarted,
}

impl AimbotStatus {
    pub fn string(&self) -> &str {
        match self {
            AimbotStatus::Working => "Working",
            AimbotStatus::GameNotStarted => "Game Not Started",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AimbotConfig {
    pub aimbot: bool,
    pub hotkey: KeyCode,
    pub start_bullet: i32,
    pub aim_lock: bool,
    pub visibility_check: bool,
    pub fov: f32,
    pub smooth: f32,
    pub multibone: bool,
    pub rcs: bool,
}

impl Default for AimbotConfig {
    fn default() -> Self {
        Self {
            aimbot: true,
            hotkey: KeyCode::MouseLeft,
            start_bullet: 2,
            aim_lock: false,
            visibility_check: true,
            fov: 2.5,
            smooth: 5.0,
            multibone: true,
            rcs: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub games: HashMap<Game, AimbotConfig>,
    pub current_game: Game,
}

impl Default for Config {
    fn default() -> Self {
        let mut games = HashMap::new();
        for game in Game::iter() {
            games.insert(game, AimbotConfig::default());
        }
        Self {
            games,
            current_game: Game::CS2,
        }
    }
}

fn get_config_path() -> String {
    format!(
        "{}/{}",
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_str()
            .unwrap(),
        CONFIG_FILE_NAME
    )
}

pub fn parse_config() -> Config {
    let config_path = get_config_path();
    let path = std::path::Path::new(config_path.as_str());
    if !path.exists() {
        return Config::default();
    }

    let config_string = read_to_string(get_config_path()).unwrap();

    toml::from_str(config_string.as_str()).unwrap_or_default()
}

pub fn write_config(config: &Config) {
    let out = toml::to_string(&config).unwrap();
    std::fs::write(get_config_path(), out).unwrap();
}
