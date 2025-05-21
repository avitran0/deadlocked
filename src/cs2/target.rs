use glam::Vec2;
use strum::IntoEnumIterator;

use crate::{constants::cs2, math::angles_to_fov};

use super::{bones::Bones, player::Player, weapon_class::WeaponClass, CS2};

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
    pub fn find_target(&mut self) {
        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        let team = local_player.team(self);
        if team != cs2::TEAM_CT && team != cs2::TEAM_T {
            self.target.reset();
            return;
        }

        let weapon_class = local_player.weapon_class(self);
        if [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
        ]
        .contains(&weapon_class)
        {
            self.target.reset();
            return;
        }

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

        let mut smallest_fov = 360.0;
        let eye_position = local_player.eye_position(self);
        if self.target.player.is_none() {
            self.target.reset();
        }
        if let Some(player) = &self.target.player {
            if !player.is_valid(self) {
                self.target.reset();
            }
        }
        for player in &self.players {
            if !ffa && team == player.team(self) {
                continue;
            }

            let head_position = player.bone_position(self, Bones::Head.u64());
            let distance = eye_position.distance(head_position);
            let angle = self.angle_to_target(&local_player, &head_position, &aim_punch);
            let fov = angles_to_fov(&view_angles, &angle);

            if fov < smallest_fov {
                smallest_fov = fov;

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
