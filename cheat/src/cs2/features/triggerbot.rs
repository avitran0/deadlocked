use std::time::{Duration, Instant};

use rand::rng;
use shared::{Bones, Weapon, WeaponClass};

use crate::{
    config::Config,
    cs2::{CS2, entity::player::Player},
    math::forward_vector,
    os::mouse::Mouse,
};

#[derive(Default)]
pub struct Triggerbot {
    shot_start: Option<Instant>,
    shot_end: Option<Instant>,
    pub active: bool,
}

impl CS2 {
    pub fn triggerbot(&mut self, config: &Config) {
        let hotkey = config.aim.triggerbot_hotkey;
        let config = self.triggerbot_config(config);

        if !config.enabled {
            return;
        }

        if !Self::check_hotkey(&self.input, config.mode, hotkey, &mut self.trigger.active) {
            return;
        }

        if self.trigger.shot_start.is_some() || self.trigger.shot_end.is_some() {
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        if config.flash_check && local_player.is_flashed(self) {
            return;
        }

        let weapon_class = local_player.weapon_class(self);

        if config.scope_check
            && weapon_class == WeaponClass::Sniper
            && !local_player.is_scoped(self)
        {
            return;
        }

        if config.velocity_check {
            let scale = if config.auto_velocity_gate {
                movement_accuracy_scale(weapon_class)
            } else {
                1.0
            };
            if local_player.velocity(self).length() > config.velocity_threshold * scale {
                return;
            }
        }

        let Some(player) = local_player.crosshair_entity(self) else {
            return;
        };

        if !self.is_ffa() && player.team(self) == local_player.team(self) {
            return;
        }

        if config.head_only {
            let head_transform = player.bone_transform(self, Bones::Head.u64());
            let hitbox = Bones::Head.hitbox();

            let eye_position = local_player.eye_position(self);
            let view_angles = local_player.view_angles(self);
            let forward = forward_vector(&view_angles);

            // real ray-vs-capsule test against the head hitbox's actual
            // shape, instead of converting its radius into an angular FOV
            // threshold around a single center point (an approximation
            // that degrades for capsules viewed at sharp angles or at very
            // close range)
            let (_, distance) = hitbox.closest_approach(head_transform, eye_position, forward);
            if distance > hitbox.radius {
                return;
            }
        }

        let mean = (*config.delay.start() + *config.delay.end()) as f32 / 2.0;
        let std_dev = (*config.delay.end() - *config.delay.start()) as f32 / 2.0;

        let normal = rand_distr::Normal::new(mean, std_dev).unwrap();
        use rand_distr::Distribution as _;
        let delay = normal.sample(&mut rng()).max(0.0) as u64;

        let hold_duration = if config.auto_hold_time {
            auto_hold_duration(&local_player.weapon(self))
        } else {
            config.shot_duration
        };

        let now = Instant::now();
        let delay = Duration::from_millis(delay);
        self.trigger.shot_start = Some(now + delay);
        self.trigger.shot_end = Some(now + delay + Duration::from_millis(hold_duration));
    }

    pub fn triggerbot_shoot(&mut self, mouse: &mut Mouse) {
        let now = Instant::now();

        if let Some(shot_time) = self.trigger.shot_start
            && now >= shot_time
        {
            mouse.left_press();
            self.trigger.shot_start = None;
        }

        if let Some(shot_end) = self.trigger.shot_end
            && now >= shot_end
        {
            mouse.left_release();
            self.trigger.shot_end = None;
        }
    }
}

/// how much the configured velocity gate should stretch (or shrink) for
/// this weapon class, based on how CS2 actually handles moving accuracy:
/// smgs and pistols stay reasonably accurate on the move, rifles and
/// heavies fall off hard, and a scoped sniper needs to be almost stationary
fn movement_accuracy_scale(class: WeaponClass) -> f32 {
    match class {
        WeaponClass::Smg | WeaponClass::Pistol => 1.6,
        WeaponClass::Shotgun => 1.2,
        WeaponClass::Rifle | WeaponClass::Heavy => 0.6,
        WeaponClass::Sniper => 0.15,
        WeaponClass::Knife | WeaponClass::Grenade | WeaponClass::Utility | WeaponClass::Unknown => {
            1.0
        }
    }
}

/// the hold time that actually fires a single clean shot for this specific
/// weapon: short for anything fully automatic (a longer hold sprays more
/// than one round), longer for single-action weapons whose slower cycle
/// needs a longer press to register reliably - the revolver's cocking
/// action most of all
fn auto_hold_duration(weapon: &Weapon) -> u64 {
    use Weapon::*;
    match weapon {
        MAC10 | MP5 | MP7 | MP9 | P90 | Bizon | UMP45 => 40,
        AK47 | Aug | Famas | Galil | M4A1S | M4A4 | SG553 => 40,
        M249 | Negev => 40,
        CZ75 => 40,
        G3SG1 | SCAR20 => 40,
        Awp | SSG08 => 160,
        Revolver => 220,
        _ => 100,
    }
}
