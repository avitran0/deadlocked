use crate::{
    config::Config,
    cs2::{CS2, entity::player::Player},
};

impl CS2 {
    pub fn fov_changer(&mut self, config: &Config) {
        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        let value = config.misc.desired_fov.clamp(1, 179);
        if config.misc.fov_changer {
            if self.original_desired_fov.is_none() {
                self.original_desired_fov = Some(local_player.desired_fov(self));
            }
            if !local_player.is_scoped(self) {
                local_player.set_fov(self, value);
            }
        } else if let Some(original) = self.original_desired_fov.take() {
            local_player.set_desired_fov(self, original);
        }
    }
}
