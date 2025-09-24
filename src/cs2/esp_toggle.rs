use crate::{config::Config, cs2::CS2};

#[derive(Debug)]
pub struct EspToggle {
    previous_button_state: bool,
    pub hotkey_toggle: bool,
}

impl Default for EspToggle {
    fn default() -> Self {
        Self {
            previous_button_state: false,
            hotkey_toggle: true,
        }
    }
}

impl CS2 {
    pub fn esp_toggle(&mut self, config: &Config) {
        let hotkey = &config.player.esp_hotkey;

        let button_state = self.is_button_down(hotkey);
        if button_state && !self.wallhack.previous_button_state {
            self.wallhack.hotkey_toggle = !self.wallhack.hotkey_toggle;
        }
        self.wallhack.previous_button_state = button_state;
    }

    pub fn esp_enabled(&self, config: &Config) -> bool {
        config.player.enabled && self.wallhack.hotkey_toggle
    }
}
