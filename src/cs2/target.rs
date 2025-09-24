use glam::Vec2;
use strum::IntoEnumIterator;

use crate::{constants::cs2, math::angles_to_fov, config::{Config, TargetingMode}};

use super::{CS2, bones::Bones, player::Player, weapon_class::WeaponClass};

#[derive(Debug, Default)]
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
        *self = Target::default();
    }
}

impl CS2 {
    pub fn find_target(&mut self, config: &Config) {
        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        let team = local_player.team(self);
        if team != cs2::TEAM_CT && team != cs2::TEAM_T {
            self.target.reset();
            return;
        }

        let weapon_class = local_player.weapon_class(self);

        let view_angles = local_player.view_angles(self);
        let ffa = self.is_ffa();
        let shots_fired = local_player.shots_fired(self);
        let aim_punch = match (weapon_class, local_player.aim_punch(self) * 2.0) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length() == 0.0 && shots_fired > 1 => {
                self.target.previous_aim_punch
            }
            (_, punch) => punch,
        };
        self.target.previous_aim_punch = aim_punch;

        if self.players.is_empty() {
            self.target.reset();
            return;
        }

        let aimbot_config = self.aimbot_config(config);
        let targeting_mode = &aimbot_config.targeting_mode;
        let max_fov = aimbot_config.fov;
        
        let mut best_fov = 360.0;
        let mut best_distance = f32::MAX;
        let eye_position = local_player.eye_position(self);
        
        if self.target.player.is_none() {
            self.target.reset();
        }
        if let Some(player) = &self.target.player
            && !player.is_valid(self)
        {
            self.target.reset();
        }
        
        for player in &self.players {
            if !ffa && team == player.team(self) {
                continue;
            }

            let head_position = player.bone_position(self, Bones::Head.u64());
            let distance = eye_position.distance(head_position);
            let angle = self.angle_to_target(&local_player, &head_position, &aim_punch);
            let fov = angles_to_fov(&view_angles, &angle);
            
            let fov_limit = max_fov * self.distance_scale(distance);
            if fov > fov_limit {
                continue;
            }

            let should_select = match targeting_mode {
                TargetingMode::Fov => fov < best_fov,
                TargetingMode::Distance => distance < best_distance,
            };

            if should_select {
                best_fov = fov;
                best_distance = distance;

                self.target.player = Some(*player);
                self.target.angle = angle;
                self.target.distance = distance;
                self.target.bone_index = Bones::Head.u64();
            }
        }

        if self.target.player.is_none() {
            return;
        }

        // update target angle
        let mut smallest_fov = 360.0;
        let target = self.target.player.as_ref().unwrap();
        for bone in Bones::iter() {
            let bone_position = target.bone_position(self, bone.u64());
            let distance = eye_position.distance(bone_position);
            let angle = self.angle_to_target(&local_player, &bone_position, &aim_punch);
            let fov = angles_to_fov(&view_angles, &angle);

            if fov < smallest_fov {
                smallest_fov = fov;

                self.target.angle = angle;
                self.target.distance = distance;
                self.target.bone_index = bone.u64();
            }
        }
        /*
        let head_position = self.get_bone_position(process, self.target.pawn, Bones::Head.u64());
        let distance = eye_position.distance(head_position);
        let angle = self.get_target_angle(process, local_pawn, head_position, aim_punch);

        self.target.angle = angle;
        self.target.distance = distance;
        self.target.bone_index = Bones::Head.u64();
        */
    }
}
