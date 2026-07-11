use std::collections::VecDeque;

use glam::Vec2;
use rand::{RngExt, rng};

use crate::{
    config::Config,
    cs2::{
        CS2,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::{compute_max_acceleration_component, record_acceleration, soft_clamp_acceleration},
    os::mouse::Mouse,
};

#[derive(Debug)]
pub struct Recoil {
    pub previous: Vec2,
    unaccounted: Vec2,
    velocity: Vec2,
    accel_history: VecDeque<Vec2>,
}

impl Default for Recoil {
    fn default() -> Self {
        Self {
            previous: Vec2::ZERO,
            unaccounted: Vec2::ZERO,
            velocity: Vec2::ZERO,
            accel_history: VecDeque::with_capacity(12),
        }
    }
}

impl Recoil {
    fn reset_smoothing(&mut self) {
        self.velocity = Vec2::ZERO;
        self.accel_history.clear();
    }
}

impl CS2 {
    pub fn rcs(&mut self, config: &Config, mouse: &mut Mouse) {
        let config = self.rcs_config(config);

        if !config.enabled {
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        let weapon_class = local_player.weapon_class(self);
        let disallowed_weapons = [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
            WeaponClass::Pistol,
            WeaponClass::Shotgun,
        ];
        if disallowed_weapons.contains(&weapon_class) {
            return;
        }

        let shots_fired = local_player.shots_fired(self);
        let aim_punch = match (weapon_class, local_player.aim_punch(self)) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length() == 0.0 && shots_fired > 1 => self.recoil.previous,
            (_, punch) => punch,
        };

        if shots_fired < 1 {
            self.recoil.previous = aim_punch;
            self.recoil.unaccounted = Vec2::ZERO;
            self.recoil.reset_smoothing();
            return;
        }
        let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);

        let mouse_angle = Vec2::new(
            (aim_punch.y - self.recoil.previous.y) / sensitivity * 100.0,
            -(aim_punch.x - self.recoil.previous.x) / sensitivity * 100.0,
        );

        let desired =
            mouse_angle * config.strength.clamp(Vec2::ZERO, Vec2::ONE) + self.recoil.unaccounted;

        self.recoil.previous = aim_punch;

        let raw_acceleration = desired - self.recoil.velocity;

        let track = Vec2::new(
            raw_acceleration.x * rng().random_range(0.55..0.75),
            raw_acceleration.y * rng().random_range(0.45..0.65),
        );

        let clamp = Vec2::new(
            soft_clamp_acceleration(
                track.x,
                compute_max_acceleration_component(
                    &self.recoil.accel_history,
                    |v| v.x,
                    3.0,
                    (4.0, 20.0),
                    10.0,
                ),
                0.15,
            ),
            soft_clamp_acceleration(
                track.y,
                compute_max_acceleration_component(
                    &self.recoil.accel_history,
                    |v| v.y,
                    2.5,
                    (1.5, 8.0),
                    5.0,
                ),
                0.30,
            ),
        );

        self.recoil.velocity += clamp;

        record_acceleration(&mut self.recoil.accel_history, clamp, 12);

        let ready = Vec2::new(
            self.recoil.velocity.x.trunc(),
            self.recoil.velocity.y.trunc(),
        );

        self.recoil.unaccounted = desired - ready;

        mouse.move_rel(ready)
    }
}
