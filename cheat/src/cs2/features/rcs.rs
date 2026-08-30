use std::collections::VecDeque;

use glam::Vec2;

use crate::{
    config::Config,
    cs2::{
        CS2,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::{compute_max_acceleration_component, record_acceleration, soft_clamp_acceleration},
    os::mouse::Mouse,
};

/// Per-axis limits for the acceleration clamp, matched to the old hand-tuned values.
struct AccelTuning {
    multiplier: f32,
    range: (f32, f32),
    fallback: f32,
    decay: f32,
}

const PITCH_TUNING: AccelTuning = AccelTuning {
    multiplier: 3.0,
    range: (4.0, 20.0),
    fallback: 10.0,
    decay: 0.15,
};

const YAW_TUNING: AccelTuning = AccelTuning {
    multiplier: 2.5,
    range: (1.5, 8.0),
    fallback: 5.0,
    decay: 0.30,
};

const TRACK_SCALE: Vec2 = Vec2::new(0.65, 0.55);
const ACCEL_HISTORY_MAX: usize = 12;

fn clamp_acceleration(
    history: &VecDeque<Vec2>,
    track: f32,
    component: impl Fn(&Vec2) -> f32,
    tuning: &AccelTuning,
) -> f32 {
    soft_clamp_acceleration(
        track,
        compute_max_acceleration_component(
            history,
            component,
            tuning.multiplier,
            tuning.range,
            tuning.fallback,
        ),
        tuning.decay,
    )
}

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
            accel_history: VecDeque::with_capacity(ACCEL_HISTORY_MAX),
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

        let track = raw_acceleration * TRACK_SCALE;

        let clamp = Vec2::new(
            clamp_acceleration(&self.recoil.accel_history, track.x, |v| v.x, &PITCH_TUNING),
            clamp_acceleration(&self.recoil.accel_history, track.y, |v| v.y, &YAW_TUNING),
        );

        self.recoil.velocity += clamp;

        record_acceleration(&mut self.recoil.accel_history, clamp, ACCEL_HISTORY_MAX);

        let ready = Vec2::new(
            self.recoil.velocity.x.trunc(),
            self.recoil.velocity.y.trunc(),
        );

        self.recoil.unaccounted = desired - ready;

        mouse.move_rel(ready)
    }
}
