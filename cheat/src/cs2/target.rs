use glam::Vec2;

use crate::{
    config::{Config, aim::TargetingMode},
    cs2::{CS2, entity::player::Player},
    math::angles_to_fov,
};

#[derive(Default)]
pub struct Target {
    pub player: Option<Player>,
    pub angle: Vec2,
    pub distance: f32,
    pub bone_index: u64,
    pub local_pawn_index: u64,
    pub previous_aim_punch: Vec2,
}

impl Target {
    pub fn reset(&mut self) {
        // This is per-frame target state; the pawn index is cached separately
        // and must survive target resets because the spotted fallback uses it.
        let local_pawn_index = self.local_pawn_index;
        *self = Target::default();
        self.local_pawn_index = local_pawn_index;
    }
}

impl CS2 {
    pub fn find_target(&mut self, config: &Config) {
        let Some(local_player) = Player::local_player(self) else {
            self.target.reset();
            return;
        };

        let team = local_player.team(self);
        if !team.is_playing() {
            self.target.reset();
            return;
        }

        let weapon_class = local_player.weapon_class(self);
        let shots_fired = local_player.shots_fired(self);
        let punch = local_player.aim_punch(self) * 2.0;
        let aim_punch = match (weapon_class, punch) {
            (shared::WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length() == 0.0 && shots_fired > 1 => {
                self.target.previous_aim_punch
            }
            (_, punch) => punch,
        };
        self.target.previous_aim_punch = aim_punch;

        let aimbot_config = self.aimbot_config(config);
        if aimbot_config.bones.is_empty() || self.players.is_empty() {
            self.target.reset();
            self.target.previous_aim_punch = aim_punch;
            return;
        }

        let view_angles = local_player.view_angles(self);
        let eye_position = local_player.eye_position(self);
        let ffa = self.is_ffa();
        let mut best: Option<(Player, Vec2, f32, u64, f32, f32)> = None;

        for player in &self.players {
            if !player.is_valid(self)
                || (!ffa && !aimbot_config.target_friendlies && team == player.team(self))
            {
                continue;
            }

            let velocity = player.velocity(self);
            for bone in &aimbot_config.bones {
                let bone_position = player.bone_position(self, bone.u64())
                    + velocity * aimbot_config.prediction_time.clamp(0.0, 0.25);

                if aimbot_config.visibility_check
                    && !player.bone_visible(self, &local_player, bone.u64())
                {
                    continue;
                }

                let distance = eye_position.distance(bone_position);
                let angle = self.angle_to_target(&local_player, &bone_position, &aim_punch);
                let fov = angles_to_fov(&view_angles, &angle);
                let fov_limit = aimbot_config.fov
                    * if aimbot_config.distance_adjusted_fov {
                        self.distance_scale(distance)
                    } else {
                        1.0
                    };
                if !fov.is_finite() || fov > fov_limit {
                    continue;
                }

                let should_select = match aimbot_config.targeting_mode {
                    TargetingMode::Fov => best.as_ref().is_none_or(|best| fov < best.2),
                    TargetingMode::Distance => best.as_ref().is_none_or(|best| distance < best.4),
                };

                if should_select {
                    best = Some((*player, angle, fov, bone.u64(), distance, fov_limit));
                }
            }
        }

        self.target.reset();
        self.target.previous_aim_punch = aim_punch;
        if let Some((player, angle, _fov, bone_index, distance, _fov_limit)) = best {
            self.target.player = Some(player);
            self.target.angle = angle;
            self.target.distance = distance;
            self.target.bone_index = bone_index;
        }
    }
}
