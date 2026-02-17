use glam::Vec2;

use crate::{
    config::Config,
    cs2::{
        CS2,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    os::mouse::Mouse,
};

#[derive(Debug, Default)]
pub struct Recoil {
    previous: Vec2,
    unaccounted: Vec2,
}

impl CS2 {
    pub fn rcs(&mut self, config: &Config, mouse: &mut Mouse) {
        let config = self.rcs_config(config);

        if !config.enabled {
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        let weapon_class = local_player.weapon_class(self);
        let disallowed_weapons = [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
            WeaponClass::Pistol,
            WeaponClass::Shotgun,
        ];
        if disallowed_weapons.contains(&weapon_class) {
            return;
        }

        let shots_fired = local_player.shots_fired(self);
        let aim_punch = match (weapon_class, local_player.aim_punch(self)) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length() == 0.0 && shots_fired > 1 => self.recoil.previous,
            (_, punch) => punch,
        };

        if shots_fired < 1 {
            self.recoil.previous = aim_punch;
            self.recoil.unaccounted = Vec2::ZERO;
            return;
        }
        let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);
        let yaw_pitch_factor = 45.454545;

        // We multiply by 2.0 because aim_punch represents half of the actual recoil displacement
        let punch_delta = (aim_punch - self.recoil.previous) * 2.0;

        let mut mouse_angle = Vec2::new(
            punch_delta.y / sensitivity * yaw_pitch_factor,
            -punch_delta.x / sensitivity * yaw_pitch_factor,
        ) + self.recoil.unaccounted;
        
        // RCS smoothing should be very subtle to keep up with the fast recoil changes
        let rcs_smooth = (config.smooth + 1.0).clamp(1.0, 5.0);
        let current_move = mouse_angle / rcs_smooth;

        self.recoil.previous = aim_punch;
        self.recoil.unaccounted = mouse_angle - current_move;

        // Only move if we aren't currently snapping with the aimbot to avoid conflict
        if !self.aim.active || !self.is_button_down(&config.aimbot_hotkey) {
             mouse.move_rel(&current_move);
        }
    }
}
