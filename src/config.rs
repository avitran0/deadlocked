use std::{fs::read_to_string, ops::Range, path::Path, time::Duration};

use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{
    color::{Color, Colors},
    key_codes::KeyCode,
};

const REFRESH_RATE: u64 = 100;
pub const LOOP_DURATION: Duration = Duration::from_millis(1000 / REFRESH_RATE);
pub const SLEEP_DURATION: Duration = Duration::from_secs(1);
pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const VERSION: &str = concat!("version: ", env!("CARGO_PKG_VERSION"));

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
    pub triggerbot: bool,
    pub triggerbot_hotkey: KeyCode,
    pub triggerbot_range: Range<u32>,
}

impl Default for AimbotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hotkey: KeyCode::Mouse5,
            start_bullet: 2,
            aim_lock: false,
            visibility_check: true,
            fov: 2.5,
            smooth: 5.0,
            multibone: true,
            rcs: false,
            triggerbot: false,
            triggerbot_hotkey: KeyCode::Mouse4,
            triggerbot_range: 100..300,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, EnumIter, PartialEq)]
pub enum DrawMode {
    #[default]
    None,
    Color,
    Health,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualsConfig {
    pub enabled: bool,
    pub draw_box: DrawMode,
    pub box_color: Color,
    pub draw_skeleton: DrawMode,
    pub skeleton_color: Color,
    pub draw_health: bool,
    pub draw_armor: bool,
    pub debug: bool,
}

impl Default for VisualsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            draw_box: DrawMode::Color,
            box_color: Color::from_egui_color(Colors::TEXT),
            draw_skeleton: DrawMode::Health,
            skeleton_color: Color::from_egui_color(Colors::TEXT),
            draw_health: true,
            draw_armor: true,
            debug: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub aimbot: AimbotConfig,
    pub visuals: VisualsConfig,
}

fn get_config_path() -> String {
    String::from(
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(CONFIG_FILE_NAME)
            .to_str()
            .unwrap(),
    )
}

pub fn parse_config() -> Config {
    let config_path = get_config_path();
    let path = Path::new(config_path.as_str());
    if !path.exists() {
        return Config::default();
    }

    let config_string = read_to_string(get_config_path()).expect("could not read config file");

    toml::from_str(&config_string).unwrap_or_default()
}

pub fn write_config(config: &Config) {
    let out = toml::to_string(&config).expect("could not write config file");
    std::fs::write(get_config_path(), out).expect("could not write config file");
}
