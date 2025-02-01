use glam::Vec2;
use strum::IntoEnumIterator;

use crate::math::angles_to_fov;

use super::{bones::Bones, constants::Constants, player::Player, weapon_class::WeaponClass, CS2};

#[derive(Debug, Default)]
pub struct Target {
    pub player: Option<Player>,
    pub angle: Vec2,
    pub distance: f32,
    pub bone_index: u64,
    pub local_pawn_index: u64,
}

impl Target {
    pub fn reset(&mut self) {
        *self = Target::default();
    }
}

impl CS2 {
    pub fn find_target(&mut self) {
        let process = match &self.process {
            Some(process) => process,
            None => {
                self.is_valid = false;
                return;
            }
        };

        let local_player = match Player::local_player(process, &self.offsets) {
            Some(player) => player,
            None => return,
        };

        let team = local_player.team(process, &self.offsets);
        if team != Constants::TEAM_CT && team != Constants::TEAM_T {
            self.target.reset();
            return;
        }

        let weapon_class = local_player.weapon_class(process, &self.offsets);
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

        let view_angles = local_player.view_angles(process, &self.offsets);
        let ffa = self.is_ffa(process);
        let shots_fired = local_player.shots_fired(process, &self.offsets);
        let aim_punch = match (
            weapon_class,
            local_player.aim_punch(process, &self.offsets) * 2.0,
        ) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length() == 0.0 && shots_fired > 1 => self.previous_aim_punch,
            (_, punch) => punch,
        };

        let mut players = Vec::with_capacity(64);
        for i in 0..=64 {
            let player = match Player::index(process, &self.offsets, i) {
                Some(player) => player,
                None => continue,
            };

            if !player.is_valid(process, &self.offsets) {
                continue;
            }

            if player == local_player {
                self.target.local_pawn_index = i - 1;
            } else {
                players.push(player);
            }
        }

        if players.is_empty() {
            self.target.reset();
            return;
        }

        let mut smallest_fov = 360.0;
        let eye_position = local_player.eye_position(process, &self.offsets);
        if self.target.player.is_none() {
            self.target.reset();
        }
        if let Some(player) = &self.target.player {
            if !player.is_valid(process, &self.offsets) {
                self.target.reset();
            }
        }
        for player in players {
            if !ffa && team == player.team(process, &self.offsets) {
                continue;
            }

            let head_position = player.bone_position(process, &self.offsets, Bones::Head.u64());
            let distance = eye_position.distance(head_position);
            let angle = self.get_target_angle(process, &local_player, &head_position, &aim_punch);
            let fov = angles_to_fov(&view_angles, &angle);

            if fov < smallest_fov {
                smallest_fov = fov;

                self.target.player = Some(player);
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
            let bone_position = target.bone_position(process, &self.offsets, bone.u64());
            let distance = eye_position.distance(bone_position);
            let angle = self.get_target_angle(process, &local_player, &bone_position, &aim_punch);
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
