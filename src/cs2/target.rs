use glam::Vec2;
use strum::IntoEnumIterator;

use crate::{
    config::{Config, TargetingMode},
    constants::cs2,
    cs2::{
        CS2,
        bones::Bones,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::angles_to_fov,
};

// ------------------------------------------------------------------
pub struct Target {
    pub player: Option<Player>,
    pub angle: Vec2,
    pub distance: f32,
    pub bone_index: u64,
    pub local_pawn_index: u64,
    pub previous_aim_punch: Vec2,
}

impl Default for Target {
    fn default() -> Self {
        Self {
            player: None,
            angle: Vec2::ZERO,
            distance: 0.0,
            bone_index: 0,
            local_pawn_index: 0,
            previous_aim_punch: Vec2::ZERO,
        }
    }
}

impl Target {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

// ------------------------------------------------------------------
impl CS2 {
    pub fn find_target(&mut self, config: &Config) {
        /* -----------------------------------------------------------
           1.  Early-outs
        ----------------------------------------------------------- */
        let local_player = match Player::local_player(self) {
            Some(p) => p,
            None => return self.target.reset(),
        };

        let local_team = local_player.team(self);
        if !matches!(local_team, cs2::TEAM_CT | cs2::TEAM_T) {
            return self.target.reset();
        }

        /* -----------------------------------------------------------
           2.  Weapon-specific recoil
        ----------------------------------------------------------- */
        let weapon_class = local_player.weapon_class(self);
        let shots_fired = local_player.shots_fired(self);
        let aim_punch = match (weapon_class, local_player.aim_punch(self) * 2.0) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length_squared() == 0.0 && shots_fired > 1 => {
                self.target.previous_aim_punch
            }
            (_, punch) => punch,
        };
        self.target.previous_aim_punch = aim_punch;

        /* -----------------------------------------------------------
           3.  Config shortcuts
        ----------------------------------------------------------- */
        let aim_cfg = self.aimbot_config(config);
        let view_angles = local_player.view_angles(self);
        let eye_pos = local_player.eye_position(self);
        let max_fov = aim_cfg.fov;
        let ffa = self.is_ffa();
        let is_custom = self.is_custom_game_mode();
        let friendly_ok = aim_cfg.target_friendlies;

        /* -----------------------------------------------------------
           4.  Helper: decide which candidate is better
        ----------------------------------------------------------- */
        #[inline(always)]
        fn is_better(mode: &TargetingMode, curr: &Candidate, best: &Candidate) -> bool {
            match mode {
                TargetingMode::Fov => curr.fov < best.fov,
                TargetingMode::Distance => curr.distance < best.distance,
            }
        }

        /* -----------------------------------------------------------
           5.  Main search
        ----------------------------------------------------------- */
        let mut best: Option<Candidate> = None;

        for &player in &self.players {
            // --- skip invalid / teammates ---------------------------------
            if !player.is_valid(self) {
                continue;
            }
            let player_team = player.team(self);
            if !ffa && !friendly_ok && !is_custom && player_team == local_team {
                continue;
            }

            // --- use HEAD as a cheap first-pass filter ---------------------
            let head_pos = player.bone_position(self, Bones::Head.u64());
            let head_dist = eye_pos.distance(head_pos);
            let head_angle = self.angle_to_target(&local_player, &head_pos, &aim_punch);
            let head_fov = angles_to_fov(&view_angles, &head_angle);

            let fov_limit = max_fov * self.distance_scale(head_dist);
            if head_fov > fov_limit {
                continue;
            }

            // --- now find the best bone on this player ---------------------
            let mut best_bone: Option<Candidate> = None;

            for bone in Bones::iter() {
                let bone_pos = player.bone_position(self, bone.u64());
                let dist = eye_pos.distance(bone_pos);
                let angle = self.angle_to_target(&local_player, &bone_pos, &aim_punch);
                let fov = angles_to_fov(&view_angles, &angle);

                let cand = Candidate {
                    player,
                    angle,
                    distance: dist,
                    bone_index: bone.u64(),
                    fov,
                };

                if best_bone.as_ref().map_or(true, |b| is_better(&aim_cfg.targeting_mode, &cand, b)) {
                    best_bone = Some(cand);
                }
            }

            // --- compare best bone of this player to global best -----------
            if let Some(bone_cand) = best_bone
                && best.as_ref().map_or(true, |b| is_better(&aim_cfg.targeting_mode, &bone_cand, b))
            {
                best = Some(bone_cand);
            }
        }

        /* -----------------------------------------------------------
           6.  Commit
        ----------------------------------------------------------- */
        match best {
            Some(c) => {
                self.target.player = Some(c.player);
                self.target.angle = c.angle;
                self.target.distance = c.distance;
                self.target.bone_index = c.bone_index;
            }
            None => self.target.reset(),
        }
    }
}

// ------------------------------------------------------------------
#[derive(Clone, Copy)]
struct Candidate {
    player: Player,
    angle: Vec2,
    distance: f32,
    bone_index: u64,
    fov: f32,
}
