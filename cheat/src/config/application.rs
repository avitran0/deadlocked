use std::{path::PathBuf, sync::LazyLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::{BASE_PATH, DEFAULT_CONFIG_NAME};

pub static APP_CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| BASE_PATH.join("deadlocked.toml"));

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApplicationConfig {
    pub first_launch: bool,
    pub radar_uuid: Uuid,
    pub config_name: String,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            first_launch: true,
            radar_uuid: Uuid::new_v4(),
            config_name: DEFAULT_CONFIG_NAME.to_owned(),
        }
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
