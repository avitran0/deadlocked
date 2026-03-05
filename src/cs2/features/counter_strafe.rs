use std::time::{Duration, Instant};

use glam::vec2;

use crate::{
    config::Config,
    cs2::{CS2, entity::player::Player, key_codes::KeyCode},
    os::mouse::{Mouse, MovementKey},
};

#[derive(Debug, Default)]
pub struct CounterStrafe {
    last_tap: Option<Instant>,
    release_w: Option<Instant>,
    release_a: Option<Instant>,
    release_s: Option<Instant>,
    release_d: Option<Instant>,
}

impl CS2 {
    pub fn counter_strafe_tick(&mut self, config: &Config, mouse: &mut Mouse) {
        self.counter_strafe_release_pending(mouse);

        if !config.misc.auto_counter_strafe || !self.input.is_key_pressed(KeyCode::MouseLeft) {
            return;
        }
        let Some(local_player) = Player::local_player(self) else {
            return;
        };
        self.counter_strafe_on_shot(config, mouse, &local_player);
    }

    pub(crate) fn counter_strafe_on_shot(
        &mut self,
        config: &Config,
        mouse: &mut Mouse,
        local_player: &Player,
    ) {
        if !config.misc.auto_counter_strafe {
            return;
        }

        let now = Instant::now();
        const MIN_TAP_INTERVAL: Duration = Duration::from_millis(80);
        if let Some(last_tap) = self.counter_strafe.last_tap
            && now.duration_since(last_tap) < MIN_TAP_INTERVAL
        {
            return;
        }

        let tap_duration = Duration::from_millis(config.misc.counter_strafe_tap_ms.clamp(5, 60));

        let mut tapped = false;

        let holding_w = self.input.is_key_pressed(KeyCode::W);
        let holding_a = self.input.is_key_pressed(KeyCode::A);
        let holding_s = self.input.is_key_pressed(KeyCode::S);
        let holding_d = self.input.is_key_pressed(KeyCode::D);

        if holding_w || holding_a || holding_s || holding_d {
            if holding_w {
                self.counter_strafe_press_for(mouse, MovementKey::S, now, tap_duration);
                tapped = true;
            }
            if holding_s {
                self.counter_strafe_press_for(mouse, MovementKey::W, now, tap_duration);
                tapped = true;
            }
            if holding_a {
                self.counter_strafe_press_for(mouse, MovementKey::D, now, tap_duration);
                tapped = true;
            }
            if holding_d {
                self.counter_strafe_press_for(mouse, MovementKey::A, now, tap_duration);
                tapped = true;
            }
        } else {
            let velocity = local_player.velocity(self);
            let velocity_2d = vec2(velocity.x, velocity.y);
            let speed = velocity_2d.length();
            if speed < config.misc.counter_strafe_speed_threshold {
                return;
            }

            // Convert world velocity into local forward/right speeds based on current yaw.
            let yaw = local_player.view_angles(self).y.to_radians();
            let forward = vec2(yaw.cos(), yaw.sin());
            let right = vec2(-yaw.sin(), yaw.cos());
            let forward_speed = velocity_2d.dot(forward);
            let right_speed = velocity_2d.dot(right);

            let threshold = config.misc.counter_strafe_speed_threshold;
            if right_speed > threshold {
                self.counter_strafe_press_for(mouse, MovementKey::A, now, tap_duration);
                self.counter_strafe_press_for(mouse, MovementKey::D, now, tap_duration);
                tapped = true;
            } else if right_speed < -threshold {
                self.counter_strafe_press_for(mouse, MovementKey::D, now, tap_duration);
                self.counter_strafe_press_for(mouse, MovementKey::A, now, tap_duration);
                tapped = true;
            }

            if forward_speed > threshold {
                self.counter_strafe_press_for(mouse, MovementKey::S, now, tap_duration);
                self.counter_strafe_press_for(mouse, MovementKey::W, now, tap_duration);
                tapped = true;
            } else if forward_speed < -threshold {
                self.counter_strafe_press_for(mouse, MovementKey::W, now, tap_duration);
                self.counter_strafe_press_for(mouse, MovementKey::S, now, tap_duration);
                tapped = true;
            }
        }

        if tapped {
            self.counter_strafe.last_tap = Some(now);
        }
    }

    fn counter_strafe_press_for(
        &mut self,
        mouse: &mut Mouse,
        key: MovementKey,
        now: Instant,
        duration: Duration,
    ) {
        mouse.movement_press(key);
        let release_at = now + duration;
        match key {
            MovementKey::W => self.counter_strafe.release_w = Some(release_at),
            MovementKey::A => self.counter_strafe.release_a = Some(release_at),
            MovementKey::S => self.counter_strafe.release_s = Some(release_at),
            MovementKey::D => self.counter_strafe.release_d = Some(release_at),
        }
    }

    fn counter_strafe_release_pending(&mut self, mouse: &mut Mouse) {
        let now = Instant::now();
        if let Some(release_at) = self.counter_strafe.release_w
            && now >= release_at
        {
            mouse.movement_release(MovementKey::W);
            self.counter_strafe.release_w = None;
        }
        if let Some(release_at) = self.counter_strafe.release_a
            && now >= release_at
        {
            mouse.movement_release(MovementKey::A);
            self.counter_strafe.release_a = None;
        }
        if let Some(release_at) = self.counter_strafe.release_s
            && now >= release_at
        {
            mouse.movement_release(MovementKey::S);
            self.counter_strafe.release_s = None;
        }
        if let Some(release_at) = self.counter_strafe.release_d
            && now >= release_at
        {
            mouse.movement_release(MovementKey::D);
            self.counter_strafe.release_d = None;
        }
    }
}
