use crate::config::Config;

use super::{player::Player, CS2};

impl CS2 {
    pub fn glow(&self, config: &Config) {
        if !config.misc.glow {
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        let team = local_player.team(self);
        for player in &self.players {
            let player_team = player.team(self);
            if !config.misc.friendly_glow && player_team == team {
                continue;
            }
            let color = if player_team == team {
                config.misc.friendly_color.to_hex()
            } else {
                config.misc.enemy_color.to_hex()
            };

            player.glow(self, color);
        }
    }
}
