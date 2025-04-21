use glam::Vec2;

use crate::mouse::Mouse;

use super::{player::Player, weapon_class::WeaponClass, CS2};

impl CS2 {
    pub fn rcs(&mut self, mouse: &mut Mouse) {
        let process = match &self.process {
            Some(process) => process,
            None => return,
        };

        let local_player = match Player::local_player(process, &self.offsets) {
            Some(player) => player,
            None => return,
        };

        if self.is_bomb_planted(process) {
            dbg!(self.get_bomb_site(process));
            dbg!(self.get_bomb_blow_time(process));
        }

        let weapon_class = local_player.weapon_class(process, &self.offsets);
        if [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
            WeaponClass::Pistol,
        ]
        .contains(&weapon_class)
        {
            return;
        }

        let shots_fired = local_player.shots_fired(process, &self.offsets);
        let aim_punch = match (weapon_class, local_player.aim_punch(process, &self.offsets)) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length() == 0.0 && shots_fired > 1 => self.previous_aim_punch,
            (_, punch) => punch,
        };

        if shots_fired < 1 {
            self.previous_aim_punch = aim_punch;
            self.unaccounted_aim_punch = Vec2::ZERO;
            return;
        }
        let sensitivity =
            self.get_sensitivity(process) * local_player.fov_multiplier(process, &self.offsets);
        let xy = (aim_punch - self.previous_aim_punch) * -1.0;

        let mouse_angle = Vec2::new(
            ((xy.y * 2.0) / sensitivity) / -0.022,
            ((xy.x * 2.0) / sensitivity) / 0.022,
        ) + self.unaccounted_aim_punch;
        self.unaccounted_aim_punch = Vec2::ZERO;

        // only if the aimbot is not active
        self.previous_aim_punch = aim_punch;
        if (0.0..1.0).contains(&mouse_angle.x) {
            self.unaccounted_aim_punch.x = mouse_angle.x;
        }
        if (0.0..1.0).contains(&mouse_angle.y) {
            self.unaccounted_aim_punch.y = mouse_angle.y;
        }
        mouse.move_rel(&mouse_angle)
    }
}
