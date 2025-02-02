use crate::config::Config;

use super::{player::Player, CS2};

impl CS2 {
    pub fn no_flash(&self, config: &Config) {
        let process = match &self.process {
            Some(process) => process,
            None => return,
        };

        if !config.no_flash {
            return;
        }

        let local_player = match Player::local_player(process, &self.offsets) {
            Some(player) => player,
            None => return,
        };

        if process.read::<f32>(local_player.pawn() + self.offsets.pawn.flash_alpha)
            != config.max_flash_alpha
        {
            process.write(
                local_player.pawn() + self.offsets.pawn.flash_alpha,
                config.max_flash_alpha.clamp(0.0, 1.0),
            );
        }
    }
}
