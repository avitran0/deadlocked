use crate::{config::Config, constants::cs2};

use super::{player::Player, CS2};

impl CS2 {
    pub fn fov_changer(&self, config: &Config) {
        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        if !config.misc.fov_changer {
            local_player.set_fov(self, cs2::DEFAULT_FOV);
            return;
        }

        local_player.set_fov(self, config.misc.desired_fov.clamp(1, 179));
    }
}
