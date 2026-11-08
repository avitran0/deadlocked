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
const MAX_THROW_SPEED: f32 = 750.0;
const MAX_STEPS: usize = (TICK_RATE * 5.0) as usize;
const FLOOR_NORMAL_Z: f32 = 0.7;
const MAX_COLLISIONS_PER_TICK: usize = 4;

impl CS2 {
    pub fn nade_prediction(&self, config: &Config, local_player: &Player) -> NadePrediction {
        let mut prediction = NadePrediction::default();
        if !config.hud.nade_prediction.enabled {
            return prediction;
        }

        let physics = &config.hud.nade_prediction;
        let radius = physics.radius.clamp(0.0, 4.0);
        let rest_speed_sq = physics.rest_speed * physics.rest_speed;

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

        // the game traces a small hull 22 units forward and spawns the grenade 6 units before the hit
        let spawn_distance = bvh
            .sweep_sphere(start, start + forward * 22.0, radius)
            .map_or(22.0, |(distance, _)| distance);
        start += forward * (spawn_distance - 6.0);

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
        'ticks: for _ in 0..MAX_STEPS {
            let new_z = velocity.z - GRAVITY * DT;
            let step_z = if physics.average_gravity {
                (velocity.z + new_z) * 0.5
            } else {
                new_z
            };
            let mut step = Vec3::new(velocity.x * DT, velocity.y * DT, step_z * DT);
            velocity.z = new_z;
            elapsed += DT;

            for _ in 0..MAX_COLLISIONS_PER_TICK {
                let length = step.length();
                if length <= 1e-4 {
                    break;
                }
                let Some((distance, normal)) =
                    bvh.sweep_sphere(position, position + step, radius)
                else {
                    position += step;
                    break;
                };

                let fraction = (distance / length).clamp(0.0, 1.0);
                position += step * fraction;
                prediction.path.push(position);

                let is_floor = normal.z > FLOOR_NORMAL_Z;
                if matches!(weapon, Weapon::Molotov | Weapon::Incendiary) && is_floor {
                    prediction.end = Some(position);
                    break 'ticks;
                }

                prediction.bounces.push(position);
                // reflect (overbounce 2.0), never keep moving into the surface, then lose energy
                velocity -= normal * (2.0 * velocity.dot(normal));
                let into_surface = velocity.dot(normal);
                if into_surface < 0.0 {
                    velocity -= normal * into_surface;
                }
                velocity *= physics.elasticity;

                let slow = velocity.length_squared() < rest_speed_sq;
                if slow {
                    if is_floor {
                        prediction.end = Some(position);
                        break 'ticks;
                    }
                    // a slow grenade on a wall loses all speed and drops
                    velocity = Vec3::ZERO;
                    break;
                }

                // only bounces off the floor continue with the rest of the tick
                if is_floor && physics.bounce_remainder {
                    step = velocity * ((1.0 - fraction) * DT);
                } else {
                    break;
                }
            }
            prediction.path.push(position);

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
