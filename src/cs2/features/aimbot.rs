use glam::vec2;

use crate::{
    config::{Config, KeyMode},
    cs2::{CS2, entity::player::Player},
    math::{angles_to_fov, vec2_clamp},
    os::mouse::Mouse,
};

#[derive(Debug, Default)]
pub struct Aimbot {
    previous_button_state: bool,
    pub active: bool,
}

impl CS2 {
    pub fn aimbot(&mut self, config: &Config, mouse: &mut Mouse) {
        let hotkey = config.aim.aimbot_hotkey;
        let config = self.aimbot_config(config);

        if !config.enabled || (self.target.player.is_none() && self.target_grenade.is_none()) {
            return;
        }

        let grenade = self.target_grenade.is_some();

        let button_state = self.is_button_down(&hotkey);
        if config.mode == KeyMode::Hold && !button_state {
            return;
        } else {
            if button_state && !self.aim.previous_button_state {
                self.aim.active = !self.aim.active;
            }
            self.aim.previous_button_state = button_state;
            if !self.aim.active {
                return;
            }
        }

        let target = self.target.player.as_ref();

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        if config.flash_check && local_player.is_flashed(self) {
            return;
        }

        if !grenade && config.visibility_check && !target.unwrap().visible(self, &local_player, None) {
            return;
        }

        let target_angle = {
            let mut smallest_fov = 360.0;
            let mut smallest_angle = glam::Vec2::ZERO;
            if grenade {
                let angle = self.target_grenade.as_ref().unwrap().view_angles;
                let fov = angles_to_fov(&local_player.view_angles(self), &angle);
                if fov < smallest_fov {
                    smallest_angle = angle;
                }
            } else {
                for bone in &config.bones {
                    let bone_pos = target.unwrap().bone_position(self, bone.u64());
                    let angle = self.angle_to_target(
                        &local_player,
                        &bone_pos,
                        &self.target.previous_aim_punch,
                    );
                    let fov = angles_to_fov(&local_player.view_angles(self), &angle);
                    if fov < smallest_fov {
                        smallest_fov = fov;
                        smallest_angle = angle;
                    }
                }
            }
            smallest_angle
        };

        let view_angles = local_player.view_angles(self);
        if angles_to_fov(&view_angles, &target_angle)
            > (config.fov
                * if config.distance_adjusted_fov {
                    self.distance_scale(self.target.distance)
                } else {
                    1.0
                })
        {
            return;
        }

        if !grenade && !target.unwrap().is_valid(self) {
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

        let mouse_angles = vec2(
            aim_angles.y / sensitivity * 50.0,
            -aim_angles.x / sensitivity * 50.0,
        ) / (if grenade { 1.0 } else { config.smooth + 1.0 }).clamp(1.0, 20.0);

        mouse.move_rel(&mouse_angles);
    }
}
