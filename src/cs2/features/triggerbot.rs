use std::time::{Duration, Instant};

use glam::{Vec2, vec2};
use rand::rng;

use crate::{
    config::{Config, KeyMode},
    cs2::{
        CS2,
        bones::Bones,
        entity::{player::Player, weapon_class::WeaponClass},
        key_codes::KeyCode,
    },
    math::{angles_to_fov, vec2_clamp},
    os::mouse::Mouse,
};

#[derive(Debug, Default)]
pub struct Triggerbot {
    shot_start: Option<Instant>,
    shot_end: Option<Instant>,
    cooldown_end: Option<Instant>,
    pending_silent_angle: Option<Vec2>,
    pub active: bool,
}

#[derive(Debug, Clone, Copy)]
struct TriggerCandidate {
    angle: Vec2,
    fov: f32,
    from_crosshair: bool,
}

impl CS2 {
    pub fn triggerbot(&mut self, config: &Config, mouse: &mut Mouse) {
        let hotkey = config.aim.triggerbot_hotkey;
        let trigger_config = self.triggerbot_config(config).clone();

        if !trigger_config.enabled {
            return;
        }

        match trigger_config.mode {
            KeyMode::Hold => {
                if !self.input.is_key_pressed(hotkey) {
                    return;
                }
            }
            KeyMode::Toggle => {
                if self.input.key_just_pressed(hotkey) {
                    self.trigger.active = !self.trigger.active;
                }
                if !self.trigger.active {
                    return;
                }
            }
            KeyMode::Shoot => {
                if !self.input.is_key_pressed(KeyCode::MouseLeft) {
                    return;
                }
            }
        }

        let now = Instant::now();
        if let Some(cooldown_end) = self.trigger.cooldown_end
            && now < cooldown_end
        {
            return;
        }

        if self.trigger.shot_start.is_some() || self.trigger.shot_end.is_some() {
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        if trigger_config.flash_check && local_player.is_flashed(self) {
            return;
        }

        let weapon_class = local_player.weapon_class(self);
        if trigger_config.scope_check
            && weapon_class == WeaponClass::Sniper
            && !local_player.is_scoped(self)
        {
            return;
        }

        if trigger_config.velocity_check {
            let velocity = local_player.velocity(self);
            let horizontal_speed = vec2(velocity.x, velocity.y).length();
            if horizontal_speed > trigger_config.velocity_threshold {
                return;
            }
        }

        let eye_position = local_player.eye_position(self);
        let view_angles = local_player.view_angles(self);
        let aim_punch = local_player.aim_punch(self) * 2.0;
        let local_team = local_player.team(self);
        let bone_list: &[Bones] = if trigger_config.head_only {
            &[Bones::Head]
        } else {
            &[
                Bones::Head,
                Bones::Neck,
                Bones::Spine3,
                Bones::Spine1,
                Bones::Hip,
            ]
        };

        let build_candidate =
            |player: Player, from_crosshair: bool, cs2: &CS2| -> Option<TriggerCandidate> {
                if !player.is_valid(cs2) {
                    return None;
                }

                if !cs2.is_ffa() && player.team(cs2) == local_team {
                    return None;
                }

                let mut best_fov = f32::MAX;
                let mut best_angle = Vec2::ZERO;
                let mut found = false;
                for bone in bone_list {
                    let target_bone = player.bone_position(cs2, bone.u64());
                    if target_bone == glam::Vec3::ZERO {
                        continue;
                    }
                    if trigger_config.smoke_wall_check
                        && !cs2.is_path_clear(eye_position, target_bone)
                    {
                        continue;
                    }

                    let angle = cs2.angle_to_target(&local_player, &target_bone, &aim_punch);
                    let fov = angles_to_fov(&view_angles, &angle);
                    if fov < best_fov {
                        best_fov = fov;
                        best_angle = angle;
                        found = true;
                    }
                }

                if !found {
                    return None;
                }

                Some(TriggerCandidate {
                    angle: best_angle,
                    fov: best_fov,
                    from_crosshair,
                })
            };

        let mut candidate = local_player
            .crosshair_entity(self)
            .and_then(|player| build_candidate(player, true, self));

        if candidate.is_none() && trigger_config.aim_assist {
            let mut best_candidate = None;
            for player in &self.players {
                let Some(player_candidate) = build_candidate(*player, false, self) else {
                    continue;
                };
                if player_candidate.fov > trigger_config.aim_fov {
                    continue;
                }
                if best_candidate
                    .as_ref()
                    .map(|best: &TriggerCandidate| player_candidate.fov < best.fov)
                    .unwrap_or(true)
                {
                    best_candidate = Some(player_candidate);
                }
            }
            candidate = best_candidate;
        }

        let Some(candidate) = candidate else {
            return;
        };

        if trigger_config.aim_assist && candidate.fov <= trigger_config.aim_fov {
            let mut aim_delta = view_angles - candidate.angle;
            if aim_delta.y < -180.0 {
                aim_delta.y += 360.0;
            }
            vec2_clamp(&mut aim_delta);

            let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);
            let smooth = (trigger_config.aim_smooth + 1.0).clamp(1.0, 20.0);
            let mouse_angles = vec2(
                aim_delta.y / sensitivity * 50.0,
                -aim_delta.x / sensitivity * 50.0,
            ) / smooth;

            if mouse_angles.length_squared() > f32::EPSILON {
                mouse.move_rel(&mouse_angles);
            }
        }

        let should_fire = candidate.from_crosshair
            || (trigger_config.aim_assist && candidate.fov <= trigger_config.fire_fov);
        if !should_fire {
            return;
        }

        self.counter_strafe_on_shot(config, mouse, &local_player);

        let min_delay = *trigger_config.delay.start();
        let max_delay = *trigger_config.delay.end();
        let delay_ms = if min_delay == max_delay {
            min_delay
        } else {
            let mean = (min_delay + max_delay) as f32 / 2.0;
            let std_dev = ((max_delay - min_delay) as f32 / 2.0).max(1.0);
            let Ok(normal) = rand_distr::Normal::new(mean, std_dev) else {
                return;
            };
            use rand_distr::Distribution as _;
            normal
                .sample(&mut rng())
                .round()
                .clamp(min_delay as f32, max_delay as f32) as u64
        };

        let tap_fire = weapon_class == WeaponClass::Pistol
            || weapon_class == WeaponClass::Shotgun
            || weapon_class == WeaponClass::Sniper;
        let press_duration_ms = if tap_fire {
            trigger_config.shot_duration.clamp(5, 25)
        } else {
            trigger_config.shot_duration.clamp(5, 250)
        };
        let cooldown_ms = if tap_fire { 16 } else { 8 };

        let delay = Duration::from_millis(delay_ms);
        let start = now + delay;
        self.trigger.shot_start = Some(start);
        self.trigger.shot_end = Some(start + Duration::from_millis(press_duration_ms));
        self.trigger.cooldown_end =
            Some(start + Duration::from_millis(press_duration_ms + cooldown_ms));
        self.trigger.pending_silent_angle = if config.misc.silent_aim {
            Some(candidate.angle)
        } else {
            None
        };
    }

    pub fn triggerbot_shoot(&mut self, mouse: &mut Mouse) {
        let now = Instant::now();

        if let Some(shot_time) = self.trigger.shot_start
            && now >= shot_time
        {
            if let Some(shot_angle) = self.trigger.pending_silent_angle.take()
                && let Some(local_player) = Player::local_player(self)
            {
                local_player.set_view_angles(self, shot_angle);
            }
            mouse.left_press();
            self.trigger.shot_start = None;
        }

        if let Some(shot_end) = self.trigger.shot_end
            && now >= shot_end
        {
            mouse.left_release();
            self.trigger.shot_end = None;
        }

        if let Some(cooldown_end) = self.trigger.cooldown_end
            && now >= cooldown_end
        {
            self.trigger.cooldown_end = None;
        }
    }
}
