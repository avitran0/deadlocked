use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use egui::Color32;
use serde::{Deserialize, Serialize};

use crate::{
    config::{
        aim::AimConfig, hud::HudConfig, player::PlayerConfig, radar::RadarConfig,
        r#unsafe::UnsafeConfig,
    },
    font::Font,
    ui::color::Colors,
};

pub mod aim;
pub mod application;
pub mod hud;
pub mod player;
pub mod radar;
pub mod text;
pub mod r#unsafe;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub aim: AimConfig,
    pub player: PlayerConfig,
    pub hud: HudConfig,
    pub misc: UnsafeConfig,
    pub radar: RadarConfig,
    pub accent_color: Color32,
    pub fps: u32,
    pub font: Font,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aim: AimConfig::default(),
            player: PlayerConfig::default(),
            hud: HudConfig::default(),
            misc: UnsafeConfig::default(),
            radar: RadarConfig::default(),
            accent_color: Colors::BLUE,
            fps: 120,
            font: Font::FiraSans,
        }
    }
}

pub const DEFAULT_CONFIG_NAME: &str = "deadlocked.toml";

pub static BASE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = std::env::var_os("XDG_CONFIG_HOME")
        .and_then(|p| {
            if p.is_empty() {
                None
            } else {
                Some(PathBuf::from(p))
            }
        })
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|base| base.join("deadlocked"))
        .unwrap_or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
        });
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
});

pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = BASE_PATH.join("configs");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
});

pub fn parse_config(path: &Path) -> Config {
    if !path.exists() || path.is_dir() {
        return Config::default();
    }

    let Ok(config_string) = std::fs::read_to_string(path) else {
        return Config::default();
    };

    let config = toml::from_str(&config_string).and_then(|mut config: toml::Value| {
        // Migrate the former shared sound volume without changing existing profiles' volume.
        if let Some(sound) = config
            .get_mut("player")
            .and_then(toml::Value::as_table_mut)
            .and_then(|player| player.get_mut("sound"))
            .and_then(toml::Value::as_table_mut)
            && let Some(volume) = sound.remove("volume")
        {
            sound.entry("hit_volume").or_insert_with(|| volume.clone());
            sound.entry("kill_volume").or_insert(volume);
        }
        config.try_into()
    });
    if config.is_err() {
        utils::warn!("config file invalid");
    } else if let Some(file_name) = path.file_name() {
        utils::info!("loaded config {:?}", file_name);
    }
    config.unwrap_or_default()
}

pub fn write_config(config: &Config, path: &Path) {
    let out = toml::to_string(&config).unwrap();
    let _ = std::fs::write(path, out);
}

pub fn delete_config(path: &Path) {
    if !path.exists() {
        return;
    }

    if std::fs::remove_file(path).is_ok()
        && let Some(file_name) = path.file_name()
    {
        utils::info!("deleted config {:?}", file_name);
    }
}

pub fn available_configs() -> Vec<PathBuf> {
    let mut files = Vec::with_capacity(8);
    let Ok(dir) = std::fs::read_dir::<&Path>(CONFIG_PATH.as_ref()) else {
        return files;
    };

    for path in dir {
        let Ok(file) = path else {
            continue;
        };
        let Ok(file_type) = file.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }
        let file_name = file.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if !file_name.ends_with(".toml") {
            continue;
        }
        files.push(file.path())
    }
    if files.is_empty() {
        let path = CONFIG_PATH.join(DEFAULT_CONFIG_NAME);
        write_config(&Config::default(), &path);
        files.push(path);
    }
    files
}
