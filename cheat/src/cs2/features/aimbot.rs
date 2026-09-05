use glam::{Vec2, Vec3, vec2};
use shared::{BoneTransform, HitboxCapsule, WeaponClass};

use crate::{
    config::Config,
    cs2::{CS2, entity::player::Player},
    math::{angles_to_fov, vec2_clamp},
    os::mouse::Mouse,
    parser::bvh::Bvh,
};

// how many steps to probe inward from a visible edge of a partially
// covered hitbox toward its (occluded) center; higher = finds a deeper,
// safer aim point at the cost of a few extra line-of-sight checks
const EXPOSED_POINT_STEPS: usize = 4;

/// picks the best point to aim at on one hitbox capsule given real,
/// possibly-partial cover: the capsule's own center when it has line of
/// sight, otherwise the deepest point reachable by stepping in from
/// whichever end of the capsule *is* visible toward that (occluded)
/// center - "aim a little past the exposed edge" instead of either the
/// covered center or the bare tip of the capsule. returns the point and
/// whether anything on the capsule was visible at all.
fn exposed_aim_point(
    bvh: Option<&Bvh>,
    eye_position: Vec3,
    hitbox: HitboxCapsule,
    transform: BoneTransform,
) -> (Vec3, bool) {
    let center = hitbox.world_center(transform);
    let Some(bvh) = bvh else {
        return (center, true);
    };
    if bvh.has_line_of_sight(eye_position, center) {
        return (center, true);
    }

    let (point0, point1) = hitbox.world_points(transform);
    let visible_edge = if bvh.has_line_of_sight(eye_position, point0) {
        Some(point0)
    } else if bvh.has_line_of_sight(eye_position, point1) {
        Some(point1)
    } else {
        None
    };

    let Some(edge) = visible_edge else {
        // nothing on this capsule has line of sight; fall back to the
        // center so callers still get a sane point, flagged as not visible
        return (center, false);
    };

    let mut best = edge;
    for step in 1..=EXPOSED_POINT_STEPS {
        let t = step as f32 / EXPOSED_POINT_STEPS as f32;
        let probe = edge.lerp(center, t);
        if bvh.has_line_of_sight(eye_position, probe) {
            best = probe;
        } else {
            break;
        }
    }
    (best, true)
}

#[derive(Default)]
pub struct Aimbot {
    pub active: bool,
    inertia: Vec2,
}

impl CS2 {
    pub fn aimbot(&mut self, config: &Config, mouse: &mut Mouse) -> bool {
        let hotkey = config.aim.aimbot_hotkey;
        let config = self.aimbot_config(config);

        if !config.enabled {
            return false;
        }

        if !Self::check_hotkey(&self.input, config.mode, hotkey, &mut self.aim.active) {
            return false;
        }

        let Some(target) = &self.target.player else {
            return false;
        };

        if !target.is_valid(self) {
            return false;
        }

        let Some(local_player) = Player::local_player(self) else {
            return false;
        };

        let weapon_class = local_player.weapon_class(self);
        let disallowed_weapons = [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
        ];
        if disallowed_weapons.contains(&weapon_class) {
            return false;
        }

        if config.flash_check && local_player.is_flashed(self) {
            return false;
        }

        if config.visibility_check && !target.visible(self, &local_player) {
            return false;
        }

        if local_player.shots_fired(self) < config.start_bullet {
            return false;
        }

        let target_angle = {
            let mut smallest_fov = 360.0;
            let mut smallest_angle = glam::Vec2::ZERO;
            let mut found_visible_bone = false;
            let target_velocity = target.velocity(self);
            let prediction_time = config.prediction_time.clamp(0.0, 0.25);
            let eye_position = local_player.eye_position(self);

            for bone in &config.bones {
                let mut transform = target.bone_transform(self, bone.u64());
                transform.position += target_velocity * prediction_time;

                // some of the configured aim bones can be behind cover
                // while others on the same target are fully exposed (an
                // arm or head peeking out); rather than snapping straight
                // to the (possibly covered) hitbox center or the bare tip
                // of it, find the deepest point on the exposed side that
                // still has line of sight. once a visible bone is found,
                // stop considering fully-occluded ones even if they'd
                // otherwise be angularly closer to the crosshair
                let (bone_pos, visible) =
                    exposed_aim_point(self.bvh.as_ref(), eye_position, bone.hitbox(), transform);
                if !visible && found_visible_bone {
                    continue;
                }
                if visible && !found_visible_bone {
                    found_visible_bone = true;
                    smallest_fov = 360.0;
                }

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
            > (config.fov
                * if config.distance_adjusted_fov {
                    self.distance_scale(self.target.distance)
                } else {
                    1.0
                })
        {
            return false;
        }

        let mut aim_angles = view_angles - target_angle;
        if aim_angles.y < -180.0 {
            aim_angles.y += 360.0
        }
        vec2_clamp(&mut aim_angles);

        let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);

        let mouse_angles = vec2(
            aim_angles.y / sensitivity * 45.45,
            -aim_angles.x / sensitivity * 45.45,
        ) / (config.smooth + 1.0).clamp(1.0, 20.0);

        let alpha = 1.0 - config.inertia.clamp(0.0, 1.0) * 0.5;
        self.aim.inertia += (mouse_angles - self.aim.inertia) * alpha;
        mouse.move_rel(self.aim.inertia);

        self.recoil.previous = local_player.aim_punch(self);

        true
    }
}
