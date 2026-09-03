use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RadarConfig {
    pub enabled: bool,
    pub url: String,
}

impl Default for RadarConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            url: "server.avitrano.com".to_owned(),
        }
    }
}
