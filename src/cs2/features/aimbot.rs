use crate::{
    config::{AimbotConfig, Config, KeyMode},
    cs2::{
        CS2,
        bones::Bones,
        entity::{player::Player, weapon_class::WeaponClass},
        features::humanized_aim::{AimHumanizer, HumanizedAimConfig},
    },
    math::{angles_to_fov, vec2_clamp},
    os::mouse::Mouse,
};

#[derive(Debug, Default)]
pub struct Aimbot {
    pub active: bool,
    humanizer: AimHumanizer,
    target_pawn: Option<u64>,
    bone_index: Option<u64>,
}

impl Aimbot {
    fn reset_path(&mut self) {
        self.humanizer.reset();
        self.target_pawn = None;
        self.bone_index = None;
    }
}

impl CS2 {
    pub fn aimbot(&mut self, config: &Config, mouse: &mut Mouse) -> bool {
        let hotkey = config.aim.aimbot_hotkey;
        let config = self.aimbot_config(config).clone();

        if !config.enabled {
            return false;
        }

        match config.mode {
            KeyMode::Hold => {
                if !self.input.is_key_pressed(hotkey) {
                    self.aim.reset_path();
                    return false;
                }
            }
            KeyMode::Toggle => {
                if self.input.key_just_pressed(hotkey) {
                    self.aim.active = !self.aim.active;
                }
                if !self.aim.active {
                    self.aim.reset_path();
                    return false;
                }
            }
        }

        self.aim_at_target(&config, mouse, true)
    }

    pub fn magnetized_triggerbot_aim(&mut self, config: &Config, mouse: &mut Mouse) -> bool {
        let config = self.aimbot_config(config).clone();
        self.aim_at_target(&config, mouse, false)
    }

    pub fn magnetized_triggerbot_aim_at(
        &mut self,
        config: &Config,
        mouse: &mut Mouse,
        target: Player,
    ) -> bool {
        let Some(local_player) = Player::local_player(self) else {
            return false;
        };

        let previous_player = self.target.player;
        let previous_angle = self.target.angle;
        let previous_distance = self.target.distance;
        let previous_bone_index = self.target.bone_index;

        self.target.player = Some(target);
        let distance_bone = if self.target.bone_index == 0 {
            Bones::Head.u64()
        } else {
            self.target.bone_index
        };
        self.target.distance = local_player
            .eye_position(self)
            .distance(target.bone_position(self, distance_bone));

        let moved = self.magnetized_triggerbot_aim(config, mouse);

        self.target.player = previous_player;
        self.target.angle = previous_angle;
        self.target.distance = previous_distance;
        self.target.bone_index = previous_bone_index;

        moved
    }

    fn aim_at_target(
        &mut self,
        config: &AimbotConfig,
        mouse: &mut Mouse,
        check_start_bullet: bool,
    ) -> bool {
        let Some(target) = self.target.player else {
            self.aim.reset_path();
            return false;
        };

        if !target.is_valid(self) {
            self.aim.reset_path();
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
            self.aim.reset_path();
            return false;
        }

        if config.flash_check && local_player.is_flashed(self) {
            self.aim.reset_path();
            return false;
        }

        if config.visibility_check && !target.visible(self, &local_player) {
            self.aim.reset_path();
            return false;
        }

        if check_start_bullet && local_player.shots_fired(self) < config.start_bullet {
            self.aim.reset_path();
            return false;
        }

        let target_changed = self.aim.target_pawn != Some(target.pawn);
        if target_changed {
            self.aim.target_pawn = Some(target.pawn);
            self.aim.bone_index = None;
            self.aim.humanizer.reset();
        }

        let target_angle = {
            let mut smallest_fov = 360.0;
            let mut smallest_angle = glam::Vec2::ZERO;

            let stable_bone = self
                .aim
                .bone_index
                .filter(|bone_index| config.bones.iter().any(|bone| bone.u64() == *bone_index));
            let selected_bones: Vec<u64> = stable_bone
                .map(|bone_index| vec![bone_index])
                .unwrap_or_else(|| config.bones.iter().map(|bone| bone.u64()).collect());

            for bone_index in selected_bones {
                let bone_pos = target.bone_position(self, bone_index);
                let angle =
                    self.angle_to_target(&local_player, &bone_pos, &self.target.previous_aim_punch);
                let fov = angles_to_fov(&local_player.view_angles(self), &angle);
                if fov < smallest_fov {
                    smallest_fov = fov;
                    smallest_angle = angle;
                    self.aim.bone_index = Some(bone_index);
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

        let mouse_angles = glam::vec2(
            aim_angles.y / sensitivity * 45.45,
            -aim_angles.x / sensitivity * 45.45,
        );

        mouse.move_rel(self.aim.humanizer.apply(
            mouse_angles,
            HumanizedAimConfig {
                smooth: config.smooth,
                inertia: config.inertia,
                curve: config.curve,
                humanization: config.humanization,
                settle_radius: 2.0,
            },
        ));

        true
    }
}
