use glam::vec2;

use crate::{
    config::Config,
    math::{angles_to_fov, vec2_clamp},
    mouse::Mouse,
};

use super::{CS2, player::Player};

impl CS2 {
    fn humanize_aim(&mut self, target_x: f32, target_y: f32, humanization_strength: f32) -> glam::Vec2 {
        use rand::Rng;
        
        let humanization_amount = (humanization_strength * 2.0) / 100.0;
        
        if humanization_amount <= 0.0 {
            self.target.previous_target = vec2(target_x, target_y);
            return vec2(target_x, target_y);
        }
        
        let mut rng = rand::rng();
        
        let movement_distance = (target_x * target_x + target_y * target_y).sqrt();
        
        let micro_scale = (movement_distance * 0.25).min(8.0) * humanization_amount;
        let micro_jitter_x = rng.random_range(-10.0..10.0) * micro_scale;
        let micro_jitter_y = rng.random_range(-10.0..10.0) * micro_scale;
        
        let jitter_scale = (movement_distance * 0.15).min(12.0) * humanization_amount;
        let jitter_x = rng.random_range(-10.0..10.0) * jitter_scale;
        let jitter_y = rng.random_range(-10.0..10.0) * jitter_scale;
        
        let perp_scale = 0.35 * rng.random_range(-10.0..10.0) * humanization_amount;
        let perp_x = -target_y * perp_scale;
        let perp_y = target_x * perp_scale;
        
        let base_smooth_factor = rng.random_range(0.4..10.0);
        let smooth_factor = 1.0 - ((1.0 - base_smooth_factor) * humanization_amount);
        let smoothed_x = target_x * smooth_factor + self.target.previous_target.x * (1.0 - smooth_factor);
        let smoothed_y = target_y * smooth_factor + self.target.previous_target.y * (1.0 - smooth_factor);
        
        let (final_smoothed_x, final_smoothed_y) = if rng.random_range(0.0..1.0) < 0.15 * humanization_amount {
            (self.target.previous_target.x, self.target.previous_target.y)
        } else {
            (smoothed_x, smoothed_y)
        };
        
        let humanized_x = final_smoothed_x + micro_jitter_x + jitter_x + perp_x;
        let humanized_y = final_smoothed_y + micro_jitter_y + jitter_y + perp_y;
        
        self.target.previous_target = vec2(target_x, target_y);
        
        vec2(humanized_x, humanized_y)
    }

    pub fn aimbot(&mut self, config: &Config, mouse: &mut Mouse) {
        let config = self.aimbot_config(config);

        if !config.enabled || self.target.player.is_none() {
            return;
        }
        let target = self.target.player.as_ref().unwrap();

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        if config.flash_check && local_player.is_flashed(self) {
            return;
        }

        if config.visibility_check && !target.visible(self, &local_player) {
            return;
        }

        let target_angle = {
            let mut smallest_fov = 360.0;
            let mut smallest_angle = glam::Vec2::ZERO;
            for bone in &config.bones {
                let bone_pos = target.bone_position(self, bone.u64());
                let angle =
                    self.angle_to_target(&local_player, &bone_pos, &self.target.previous_aim_punch);
                let fov = angles_to_fov(&local_player.view_angles(self), &angle);
                if fov < smallest_fov {
                    smallest_fov = fov;
                    smallest_angle = angle;
                }
            }
            smallest_angle
        };

        let view_angles = local_player.view_angles(self);
        if angles_to_fov(&view_angles, &target_angle)
            > (config.fov * self.distance_scale(self.target.distance))
        {
            return;
        }

        if !target.is_valid(self) {
            return;
        }

        if local_player.shots_fired(self) < config.start_bullet {
            return;
        }

        let mut aim_angles = view_angles - target_angle;
        if aim_angles.y < -180.0 {
            aim_angles.y += 360.0
        }
        vec2_clamp(&mut aim_angles);

        let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);

        let mut mouse_angles = vec2(
            aim_angles.y / sensitivity * 50.0,
            -aim_angles.x / sensitivity * 50.0,
        ) / (config.smooth + 1.0).clamp(1.0, 10.0);

        if config.humanization > 0.0 {
            mouse_angles = self.humanize_aim(mouse_angles.x, mouse_angles.y, config.humanization);
        }

        mouse.move_rel(&mouse_angles);
    }
}
