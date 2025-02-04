use std::fs::File;

use glam::vec2;

use crate::{
    config::Config,
    math::{aim_smooth, angles_to_fov, vec2_clamp},
    mouse::mouse_move,
};

use super::{bones::Bones, player::Player, CS2};

impl CS2 {
    pub fn aimbot(&mut self, config: &Config, mouse: &mut File) {
        let process = match &self.process {
            Some(process) => process,
            None => {
                self.is_valid = false;
                return;
            }
        };

        if !config.aimbot.enabled
            || self.target.player.is_none()
            || !self.is_button_down(process, &config.aimbot.hotkey)
        {
            return;
        }
        let target = self.target.player.as_ref().unwrap();

        let local_player = match Player::local_player(process, &self.offsets) {
            Some(player) => player,
            None => return,
        };

        if config.aimbot.flash_check && local_player.is_flashed(process, &self.offsets) {
            return;
        }

        if config.aimbot.visibility_check {
            let spotted_mask = target.spotted_mask(process, &self.offsets);
            if (spotted_mask & (1 << self.target.local_pawn_index)) == 0 {
                return;
            }
        }

        let target_angle = if config.aimbot.multibone {
            self.target.angle
        } else {
            let head_position = target.bone_position(process, &self.offsets, Bones::Head.u64());
            self.get_target_angle(
                process,
                &local_player,
                &head_position,
                &self.target.previous_aim_punch,
            )
        };

        let view_angles = local_player.view_angles(process, &self.offsets);
        if angles_to_fov(&view_angles, &target_angle)
            > (config.aimbot.fov * self.distance_scale(self.target.distance))
        {
            return;
        }

        if !target.is_valid(process, &self.offsets) {
            return;
        }

        if local_player.shots_fired(process, &self.offsets) < config.aimbot.start_bullet {
            return;
        }

        let mut aim_angles = view_angles - target_angle;
        if aim_angles.y < -180.0 {
            aim_angles.y += 360.0
        }
        vec2_clamp(&mut aim_angles);

        let sensitivity =
            self.get_sensitivity(process) * local_player.fov_multiplier(process, &self.offsets);

        let xy = vec2(
            aim_angles.y / sensitivity * 50.0,
            -aim_angles.x / sensitivity * 50.0,
        );
        let smooth_angles = if !config.aimbot.aim_lock && config.aimbot.smooth > 1.0 {
            aim_smooth(&xy, config.aimbot.smooth)
        } else {
            xy
        };

        mouse_move(mouse, smooth_angles)
    }
}
