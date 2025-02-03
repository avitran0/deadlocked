use crate::{config::Config, constants::Constants};

use super::{player::Player, CS2};

impl CS2 {
    pub fn fov_changer(&self, config: &Config) {
        let process = match &self.process {
            Some(process) => process,
            None => return,
        };

        let local_player = match Player::local_player(process, &self.offsets) {
            Some(player) => player,
            None => return,
        };

        if !config.misc.fov_changer {
            local_player.set_fov(process, &self.offsets, Constants::DEFAULT_FOV);
            return;
        }

        local_player.set_fov(process, &self.offsets, config.misc.desired_fov);
    }
}
