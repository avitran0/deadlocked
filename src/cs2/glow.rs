use crate::config::Config;

use super::{player::Player, CS2};

impl CS2 {
    pub fn glow(&self, config: &Config) {
        let process = match &self.process {
            Some(process) => process,
            None => return,
        };

        if !config.misc.glow {
            return;
        }

        let local_player = match Player::local_player(process, &self.offsets) {
            Some(player) => player,
            None => return,
        };

        let team = local_player.team(process, &self.offsets);
        for player in &self.players {
            let player_team = player.team(process, &self.offsets);
            if !config.misc.friendly_glow && player_team == team {
                continue;
            }
            let color = if player_team == team {
                config.misc.friendly_color.to_hex()
            } else {
                config.misc.enemy_color.to_hex()
            };

            player.glow(process, &self.offsets, color);
        }
    }
}
