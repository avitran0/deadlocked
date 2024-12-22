use std::{collections::HashMap, fs::read_to_string, time::Duration};

use eframe::egui::Color32;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{
    color::Color,
    key_codes::KeyCode,
    message::{DrawStyle, Game},
};

pub const DEBUG_WITHOUT_MOUSE: bool = false;

const REFRESH_RATE: u64 = 100;
pub const LOOP_DURATION: Duration = Duration::from_millis(1000 / REFRESH_RATE);
pub const SLEEP_DURATION: Duration = Duration::from_secs(1);
pub const CONFIG_FILE_NAME: &str = "config.toml";

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
    pub enabled: bool,
    pub hotkey: KeyCode,
    pub start_bullet: i32,
    pub aim_lock: bool,
    pub visibility_check: bool,
    pub fov: f32,
    pub smooth: f32,
    pub multibone: bool,
    pub rcs: bool,
    pub visuals_enabled: bool,
    pub dropped_items: bool,
}

impl Default for AimbotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hotkey: KeyCode::MouseLeft,
            start_bullet: 2,
            aim_lock: false,
            visibility_check: true,
            fov: 2.5,
            smooth: 5.0,
            multibone: true,
            rcs: false,
            visuals_enabled: true,
            dropped_items: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualsConfig {
    pub enabled: bool,
    pub draw_box: DrawStyle,
    pub box_color: Color,
    pub draw_skeleton: DrawStyle,
    pub skeleton_color: Color,
    pub draw_health: bool,
    pub draw_armor: bool,
    pub armor_color: Color,
    pub draw_weapon: bool,
    pub dropped_items: bool,
    pub visibility_check: bool,
    pub fps: u64,
    pub debug_window: bool,
}

impl Default for VisualsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            draw_box: DrawStyle::Color,
            box_color: Color::from_egui_color(Color32::WHITE),
            draw_skeleton: DrawStyle::Health,
            skeleton_color: Color::from_egui_color(Color32::WHITE),
            draw_health: true,
            draw_armor: true,
            armor_color: Color::from_egui_color(Color32::BLUE),
            draw_weapon: true,
            dropped_items: true,
            visibility_check: false,
            fps: 120,
            debug_window: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameConfig {
    pub aimbot: AimbotConfig,
    pub visuals: VisualsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub games: HashMap<Game, GameConfig>,
    pub current_game: Game,
}

impl Default for Config {
    fn default() -> Self {
        let mut games = HashMap::new();
        for game in Game::iter() {
            games.insert(game, GameConfig::default());
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
