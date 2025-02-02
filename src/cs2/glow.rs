use crate::config::Config;

use super::{player::Player, CS2};

impl CS2 {
    pub fn glow(&self, config: &Config) {
        let process = match &self.process {
            Some(process) => process,
            None => return,
        };

        if !config.glow {
            return;
        }

        let local_player = match Player::local_player(process, &self.offsets) {
            Some(player) => player,
            None => return,
        };

        let team = local_player.team(process, &self.offsets);
        for player in &self.players {
            let color = if player.team(process, &self.offsets) == team {
                config.glow_friendly_color.to_hex()
            } else {
                config.glow_enemy_color.to_hex()
            };

            process.write(
                player.pawn() + self.offsets.pawn.glow + self.offsets.glow.is_glowing,
                1u8,
            );
            process.write(
                player.pawn() + self.offsets.pawn.glow + self.offsets.glow.glow_type,
                3,
            );
            process.write(
                player.pawn() + self.offsets.pawn.glow + self.offsets.glow.color_override,
                color,
            );
        }
    }
}
