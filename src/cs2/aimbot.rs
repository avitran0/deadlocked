use glam::vec2;
use std::time::Instant;

use crate::{
    config::Config,
    math::{angles_to_fov, vec2_clamp},
    mouse::Mouse,
};

use super::{CS2, player::Player};

impl CS2 {
    fn humanize_aim(&mut self, target_x: f32, target_y: f32, humanization_strength: f32) -> glam::Vec2 {
        use rand::Rng;
        
        let humanization_amount = (humanization_strength * 2.0) / 50.0;
        
        if humanization_amount <= 0.0 {
            self.target.previous_target = vec2(target_x, target_y);
            return vec2(target_x, target_y);
        }

        if humanization_amount < 0.05 {
            return vec2(target_x, target_y);
        }

        let now = Instant::now();
        let time_since_last = self.target.last_adjustment_time
            .map(|t| now.duration_since(t).as_secs_f32())
            .unwrap_or(0.016);
        
        let mut rng = rand::rng();
        
        let should_adjust = if humanization_amount < 0.1 {
            true
        } else {
            let temporal_variance = rng.random_range(0.8..1.2);
            time_since_last >= (self.target.adjustment_interval * temporal_variance)
        };
        
        if !should_adjust {
            return self.target.previous_target;
        }
        
        self.target.last_adjustment_time = Some(now);
        self.target.adjustment_interval = rng.random_range(0.012..0.025);
        
        let movement_distance = (target_x * target_x + target_y * target_y).sqrt();
        
        let jitter_range_scale = (humanization_amount * 10.0).min(10.0);
        
        let noise_variance = rng.random_range(0.7..1.3);
        let micro_scale = (movement_distance * 0.15).min(5.0) * humanization_amount * noise_variance;
        let micro_jitter_x = rng.random_range(-jitter_range_scale..jitter_range_scale) * micro_scale;
        let micro_jitter_y = rng.random_range(-jitter_range_scale..jitter_range_scale) * micro_scale;
        
        let jitter_variance = rng.random_range(0.6..1.4);
        let jitter_scale = (movement_distance * 0.08).min(8.0) * humanization_amount * jitter_variance;
        let jitter_x = rng.random_range(-jitter_range_scale..jitter_range_scale) * jitter_scale;
        let jitter_y = rng.random_range(-jitter_range_scale..jitter_range_scale) * jitter_scale;
        
        let perp_scale = 0.2 * humanization_amount;
        let perp_random = rng.random_range(-1.0..1.0);
        let perp_x = -target_y * perp_scale * perp_random;
        let perp_y = target_x * perp_scale * perp_random;
        
        let smooth_variance = rng.random_range(0.3..1.2);
        let base_smooth_factor = rng.random_range(0.4..10.0) * smooth_variance;
        let smooth_factor = 1.0 - ((1.0 - base_smooth_factor) * humanization_amount);
        let smoothed_x = target_x * smooth_factor + self.target.previous_target.x * (1.0 - smooth_factor);
        let smoothed_y = target_y * smooth_factor + self.target.previous_target.y * (1.0 - smooth_factor);
        
        let hesitation_threshold = 0.15 * humanization_amount * rng.random_range(0.5..1.5);
        let (final_smoothed_x, final_smoothed_y) = if rng.random_range(0.0..1.0) < hesitation_threshold {
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

        if config.humanization >= 1.0 {
            use rand::Rng;
            let now = std::time::Instant::now();
            
            if self.target.target_acquired_time.is_none() {
                self.target.target_acquired_time = Some(now);
                let mut rng = rand::rng();
                self.target.reaction_delay = rng.random_range(0.15..0.30);
            }
            
            if let Some(acquired_time) = self.target.target_acquired_time {
                let elapsed = now.duration_since(acquired_time).as_secs_f32();
                if elapsed < self.target.reaction_delay {
                    return;
                }
            }
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

        let smooth_value = if config.humanization > 0.0 {
            config.humanization
        } else {
            config.smooth
        };

        let mut mouse_angles = vec2(
            aim_angles.y / sensitivity * 50.0,
            -aim_angles.x / sensitivity * 50.0,
        ) / (smooth_value + 1.0).clamp(1.0, 10.0);

        if config.humanization > 0.0 {
            mouse_angles = self.humanize_aim(mouse_angles.x, mouse_angles.y, config.humanization);
        }

        if config.humanization >= 3.0 {
            let current_distance = mouse_angles.length();
            if self.target.initial_distance == 0.0 || current_distance > self.target.initial_distance {
                self.target.initial_distance = current_distance;
                self.target.aim_progress = 0.0;
            }

            if self.target.initial_distance > 0.0 {
                let progress = 1.0 - (current_distance / self.target.initial_distance).clamp(0.0, 1.0);
                self.target.aim_progress = progress;

                let ease_factor = if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    1.0 - (-2.0 * progress + 2.0).powi(2) / 2.0
                };

                let humanization_scale = ((config.humanization - 3.0) / 7.0).clamp(0.0, 1.0);
                let min_velocity = 0.5 + (0.4 * (1.0 - humanization_scale));
                let velocity_multiplier = (ease_factor * 2.0 + (1.0 - humanization_scale)).clamp(min_velocity, 1.0);
                mouse_angles *= velocity_multiplier;
            }
        }

        mouse.move_rel(&mouse_angles);
    }
}
