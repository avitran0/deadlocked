use glam::{Vec2, vec2};

use crate::{
    config::{Config, aim::AimbotConfig},
    cs2::{
        CS2,
        bones::Bones,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::{angles_to_fov, vec2_clamp},
    os::mouse::Mouse,
};

#[derive(Debug, Default)]
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

        let Some((target_angle, _target_distance)) =
            self.best_aim_bone(config, target, &local_player)
        else {
            return false;
        };

        let view_angles = local_player.view_angles(self);

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

    fn best_aim_bone(
        &self,
        config: &AimbotConfig,
        target: &Player,
        local_player: &Player,
    ) -> Option<(Vec2, f32)> {
        self.best_aim_bone_from(config, target, local_player, &config.prioritized_bones)
            .or_else(|| self.best_aim_bone_from(config, target, local_player, &config.bones))
    }

    fn best_aim_bone_from(
        &self,
        config: &AimbotConfig,
        target: &Player,
        local_player: &Player,
        bones: &[Bones],
    ) -> Option<(Vec2, f32)> {
        let view_angles = local_player.view_angles(self);
        let eye_position = local_player.eye_position(self);
        let mut best = None;
        let mut smallest_fov = 360.0;

        for bone in bones {
            if !config.bones.contains(bone) {
                continue;
            }

            if config.visibility_check && !target.bone_visible(self, local_player, *bone) {
                continue;
            }

            let bone_pos = target.bone_position(self, bone.u64());
            let angle =
                self.angle_to_target(local_player, &bone_pos, &self.target.previous_aim_punch);
            let fov = angles_to_fov(&view_angles, &angle);
            let distance = eye_position.distance(bone_pos);
            let fov_limit = config.fov
                * if config.distance_adjusted_fov {
                    self.distance_scale(distance)
                } else {
                    1.0
                };

            if fov > fov_limit {
                continue;
            }

            if fov < smallest_fov {
                smallest_fov = fov;
                best = Some((angle, distance));
            }
        }

        best
    }
}
