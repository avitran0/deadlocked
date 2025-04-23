use glam::Vec2;

use crate::mouse::Mouse;

use super::{player::Player, weapon_class::WeaponClass, CS2};

#[derive(Debug, Default)]
pub struct Recoil {
    previous: Vec2,
    unaccounted: Vec2,
}

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

        let weapon_class = local_player.weapon_class(process, &self.offsets);
        if [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
            WeaponClass::Pistol,
            WeaponClass::Shotgun,
        ]
        .contains(&weapon_class)
        {
            return;
        }

        let shots_fired = local_player.shots_fired(process, &self.offsets);
        let aim_punch = match (weapon_class, local_player.aim_punch(process, &self.offsets)) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length() == 0.0 && shots_fired > 1 => self.recoil.previous,
            (_, punch) => punch,
        };

        if shots_fired < 1 {
            self.recoil.previous = aim_punch;
            self.recoil.unaccounted = Vec2::ZERO;
            return;
        }
        let sensitivity =
            self.get_sensitivity(process) * local_player.fov_multiplier(process, &self.offsets);

        // todo: test that this works
        let mouse_angle = Vec2::new(
            (aim_punch.y - self.recoil.previous.y) / sensitivity * 100.0,
            -(aim_punch.x - self.recoil.previous.x) / sensitivity * 100.0,
        ) + self.recoil.unaccounted;

        self.recoil.unaccounted = Vec2::ZERO;

        // only if the aimbot is not active
        self.recoil.previous = aim_punch;
        if (0.0..1.0).contains(&mouse_angle.x) {
            self.recoil.unaccounted.x = mouse_angle.x;
        }
        if (0.0..1.0).contains(&mouse_angle.y) {
            self.recoil.unaccounted.y = mouse_angle.y;
        }
        mouse.move_rel(&mouse_angle)
    }
}
