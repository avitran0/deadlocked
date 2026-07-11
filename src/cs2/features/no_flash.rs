use crate::{
    config::Config,
    cs2::{CS2, entity::player::Player},
};

impl CS2 {
    pub fn no_flash(&self, config: &Config) {
        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        if config.misc.no_flash {
            local_player.no_flash(self, config.misc.max_flash_alpha);
        }
    }
}
