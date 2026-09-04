use crate::{
    config::{Config, aim::KeyMode},
    cs2::{CS2, key_codes::KeyCode},
};

pub struct EspToggle {
    pub active: bool,
}

impl Default for EspToggle {
    fn default() -> Self {
        Self { active: true }
    }
}

impl CS2 {
    pub fn esp_toggle(&mut self, config: &Config) {
        let hotkey = config.player.esp_hotkey;
        if hotkey == KeyCode::None {
            return;
        }

        Self::check_hotkey(&self.input, KeyMode::Toggle, hotkey, &mut self.esp.active);
    }

    pub fn esp_enabled(&self, config: &Config) -> bool {
        config.player.enabled && self.esp.active
    }
}
