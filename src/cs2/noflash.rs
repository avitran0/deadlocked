use crate::config::Config;

use super::{player::Player, CS2};

impl CS2 {
    pub fn no_flash(&self, config: &Config) {
        let process = match &self.process {
            Some(process) => process,
            None => return,
        };

        if !config.misc.no_flash {
            return;
        }

        let local_player = match Player::local_player(process, &self.offsets) {
            Some(player) => player,
            None => return,
        };

        local_player.no_flash(process, &self.offsets, config.misc.max_flash_alpha);
    }
}
