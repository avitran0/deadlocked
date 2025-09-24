use crate::{
    config::Config,
    cs2::CS2,
};

#[derive(Debug)]
pub struct Wallhack {
    previous_button_state: bool,
    pub(crate) hotkey_toggle: bool,
}

impl Default for Wallhack {
    fn default() -> Self {
        Self {
            previous_button_state: false,
            hotkey_toggle: true,
        }
    }
}

impl CS2 {
    pub fn wallhack(&mut self, config: &Config) {
        let hotkey = &config.player.wallhack_hotkey;
        
        let button_state = self.is_button_down(hotkey);
        if button_state && !self.wallhack.previous_button_state {
            self.wallhack.hotkey_toggle = !self.wallhack.hotkey_toggle;
        }
        self.wallhack.previous_button_state = button_state;
    }
    
    pub fn wallhack_enabled(&self, config: &Config) -> bool {
        config.player.enabled && self.wallhack.hotkey_toggle
    }
}