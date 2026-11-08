use glam::Vec3;
use shared::{NadePrediction, Weapon};

use crate::{
    config::Config,
    cs2::{CS2, entity::player::Player},
};

// all values are read only approximations of the game's grenade physics
const TICK_RATE: f32 = 64.0;
const DT: f32 = 1.0 / TICK_RATE;
const GRAVITY: f32 = 800.0 * 0.4;
const ELASTICITY: f32 = 0.45;
const MAX_THROW_SPEED: f32 = 750.0;
const MAX_STEPS: usize = (TICK_RATE * 5.0) as usize;
const REST_SPEED: f32 = 20.0;
const FLOOR_NORMAL_Z: f32 = 0.7;
const SURFACE_OFFSET: f32 = 0.5;

impl CS2 {
    pub fn nade_prediction(&self, config: &Config, local_player: &Player) -> NadePrediction {
        let mut prediction = NadePrediction::default();
        if !config.hud.nade_prediction.enabled {
            return prediction;
        }

        let weapon = local_player.weapon(self);
        if !matches!(
            weapon,
            Weapon::Flashbang
                | Weapon::HE
                | Weapon::Smoke
                | Weapon::Molotov
                | Weapon::Incendiary
                | Weapon::Decoy
        ) {
            return prediction;
        }
        let Some(bvh) = self.bvh.as_ref() else {
            return prediction;
        };

        // while the pin is not pulled the game has no strength yet, so show a full throw
        let mut strength = 1.0;
        if let Some(entity) = local_player.weapon_entity(self) {
            let pin_pulled: u8 = self
                .process
                .read(entity.address() + self.offsets.grenade.pin_pulled);
            if pin_pulled != 0 {
                let value: f32 = self
                    .process
                    .read(entity.address() + self.offsets.grenade.throw_strength);
                if value.is_finite() {
                    strength = value.clamp(0.0, 1.0);
                }
            }
        }

        let view_angles = local_player.view_angles(self);
        let mut pitch = view_angles.x;
        let yaw = view_angles.y;
        if pitch < -90.0 {
            pitch += 360.0;
        } else if pitch > 90.0 {
            pitch -= 360.0;
        }
        // thrown grenades fly slightly upwards compared to the view
        pitch -= (90.0 - pitch.abs()) * 10.0 / 90.0;

        let (pitch_rad, yaw_rad) = (pitch.to_radians(), yaw.to_radians());
        let forward = Vec3::new(
            pitch_rad.cos() * yaw_rad.cos(),
            pitch_rad.cos() * yaw_rad.sin(),
            -pitch_rad.sin(),
        );

        let mut start = local_player.eye_position(self);
        start.z += strength * 12.0 - 12.0;

        // keep the spawn point out of walls right in front of the player
        let spawn_target = start + forward * 22.0;
        if let Some((distance, _)) = bvh.trace_segment(start, spawn_target) {
            start += forward * (distance - 6.0).max(0.0);
        } else {
            start = spawn_target - forward * 6.0;
        }

        let throw_speed = ((90.0 - pitch) * 6.0).clamp(15.0, MAX_THROW_SPEED);
        let throw_speed = throw_speed * (strength * 0.7 + 0.3);
        let mut velocity = forward * throw_speed + local_player.velocity(self) * 1.25;

        let fuse = match weapon {
            Weapon::Flashbang | Weapon::HE => 1.5,
            Weapon::Smoke => 3.0,
            Weapon::Molotov | Weapon::Incendiary => 2.0,
            _ => 5.0,
        };

        let mut position = start;
        prediction.path.push(position);
        let mut elapsed = 0.0;
        for _ in 0..MAX_STEPS {
            velocity.z -= GRAVITY * DT;
            let next = position + velocity * DT;
            elapsed += DT;

            if let Some((distance, normal)) = bvh.trace_segment(position, next) {
                let step = next - position;
                let length = step.length();
                position += step.normalize_or_zero() * distance.min(length);
                position += normal * SURFACE_OFFSET;
                prediction.path.push(position);

                let is_floor = normal.z > FLOOR_NORMAL_Z;
                if matches!(weapon, Weapon::Molotov | Weapon::Incendiary) && is_floor {
                    prediction.end = Some(position);
                    break;
                }

                prediction.bounces.push(position);
                velocity -= normal * (2.0 * velocity.dot(normal));
                velocity *= ELASTICITY;
                if is_floor && velocity.length() < REST_SPEED {
                    prediction.end = Some(position);
                    break;
                }
            } else {
                position = next;
                prediction.path.push(position);
            }

            if elapsed >= fuse {
                prediction.end = Some(position);
                break;
            }
        }

        if prediction.end.is_none() {
            prediction.end = prediction.path.last().copied();
        }
        prediction.weapon = weapon;
        prediction
    }
}
