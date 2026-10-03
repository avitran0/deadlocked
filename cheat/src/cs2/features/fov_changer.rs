use crate::{
    config::Config,
    constants::cs2::{FOV_MAX, FOV_MIN},
    cs2::{CS2, entity::player::Player},
};

impl CS2 {
    pub fn fov_changer(&mut self, config: &Config) {
        let value = config.misc.desired_fov.clamp(FOV_MIN, FOV_MAX);
        if config.misc.fov_changer {
            self.fov_writer.set(true, value);
            if self.original_desired_fov.is_none() {
                if let Some(local_player) = Player::local_player(self) {
                    self.original_desired_fov = Some(local_player.desired_fov(self));
                }
            }
        } else {
            self.fov_writer.set(false, value);
            if let Some(original) = self.original_desired_fov.take() {
                if let Some(local_player) = Player::local_player(self) {
                    local_player.set_fov(self, original);
                }
            }
        }
    }
}
