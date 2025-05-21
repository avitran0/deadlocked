use crate::config::Config;

use super::{player::Player, CS2};

impl CS2 {
    pub fn no_flash(&self, config: &Config) {
        if !config.misc.no_flash {
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        local_player.no_flash(self, config.misc.max_flash_alpha);
    }
}
