use std::{
    fs::File,
    time::{Duration, Instant},
};

use glam::vec3;
use rand::{rng, Rng};

use crate::{
    config::Config,
    cs2::bones::Bones,
    input_device::{mouse_left_press, mouse_left_release},
    math::angles_to_direction,
};

use super::{player::Player, CS2};

impl CS2 {
    pub fn triggerbot(&mut self, config: &Config, mouse: &mut File) {
        let process = match &self.process {
            Some(process) => process,
            None => return,
        };

        if !config.triggerbot.enabled
            || self.target.player.is_none()
            || !self.is_button_down(process, &config.triggerbot.hotkey)
        {
            return;
        }
        let target = self.target.player.as_ref().unwrap();

        let local_player = match Player::local_player(process, &self.offsets) {
            Some(player) => player,
            None => return,
        };

        if let Some(last_shot) = self.last_shot {
            if Instant::now() > last_shot {
                mouse_left_press(mouse);
                mouse_left_release(mouse);
                self.last_shot = None;
            }
        }

        if config.triggerbot.flash_check && local_player.is_flashed(process, &self.offsets) {
            return;
        }

        if config.triggerbot.visibility_check {
            let spotted_mask = target.spotted_mask(process, &self.offsets);
            if (spotted_mask & (1 << self.target.local_pawn_index)) == 0 {
                return;
            }
        }

        const RADIUS: f32 = 4.0;
        let bone_pos = if self.target.bone_index == Bones::Head.u64() {
            target.bone_position(process, &self.offsets, self.target.bone_index)
                + vec3(0.0, 0.0, 4.0)
        } else {
            target.bone_position(process, &self.offsets, self.target.bone_index)
        };
        let player_pos = local_player.eye_position(process, &self.offsets);
        let view_direction = angles_to_direction(&local_player.view_angles(process, &self.offsets));
        let to_target = bone_pos - player_pos;

        let max_angle = RADIUS / to_target.length();
        let actual_angle = to_target.normalize().dot(view_direction).acos();
        if actual_angle <= max_angle && self.last_shot.is_none() {
            if config.triggerbot.delay_range.is_empty() {
                self.last_shot = Some(Instant::now());
            } else {
                self.last_shot = Some(
                    Instant::now()
                        + Duration::from_millis(
                            rng().random_range(config.triggerbot.delay_range.clone()),
                        ),
                );
            }
        }
    }
}
