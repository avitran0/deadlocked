use std::{path::PathBuf, sync::LazyLock};

use serde::{Deserialize, Serialize};

use crate::config::BASE_PATH;

pub static APP_CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| BASE_PATH.join("deadlocked.toml"));

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApplicationConfig {
    pub first_launch: bool,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self { first_launch: true }
    }
}

pub fn read_app_config() -> ApplicationConfig {
    if !APP_CONFIG_PATH.exists() {
        return ApplicationConfig::default();
    }

    let Ok(config_string) = std::fs::read_to_string(APP_CONFIG_PATH.as_path()) else {
        return ApplicationConfig::default();
    };

    let config = toml::from_str(&config_string);
    config.unwrap_or_default()
}

#[allow(dead_code)]
pub fn write_app_config(config: &ApplicationConfig) {
    let out = toml::to_string(&config).unwrap();
    let _ = std::fs::write(APP_CONFIG_PATH.as_path(), out);
}
