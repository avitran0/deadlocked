use glam::vec2;

use crate::{
    config::Config,
    constants::cs2::GRENADES,
    cs2::{CS2, entity::player::Player},
    math::{angles_to_fov, vec2_clamp},
    os::mouse::Mouse,
    ui::grenades::Grenade,
};

#[derive(Default)]
pub struct GrenadeAlign {
    pub is_aligning: bool,
}

impl CS2 {
    pub fn grenade_align(&mut self, config: &Config, mouse: &mut Mouse) -> bool {
        let align_config = &config.aim.grenade_align;

        if !align_config.enabled {
            self.grenade_align.is_aligning = false;
            return false;
        }

        if !self.input.is_key_pressed(align_config.hotkey) {
            self.grenade_align.is_aligning = false;
            return false;
        }

        let Some(local_player) = Player::local_player(self) else {
            self.grenade_align.is_aligning = false;
            return false;
        };

        if !local_player.is_valid(self) {
            self.grenade_align.is_aligning = false;
            return false;
        }

        let weapon = local_player.weapon(self);
        if !GRENADES.contains(&weapon) {
            self.grenade_align.is_aligning = false;
            return false;
        }

        let current_map = self.current_map();
        let Some(grenades) = self.grenades.get(&current_map) else {
            self.grenade_align.is_aligning = false;
            return false;
        };

        let player_position = local_player.position(self);
        let view_angles = local_player.view_angles(self);

        let mut best_grenade: Option<(&Grenade, f32)> = None;

        const MAX_DISTANCE: f32 = 24.0;
        for grenade in grenades {
            if grenade.weapon != weapon {
                continue;
            }

            let dist = (player_position - grenade.position).length();
            if dist > MAX_DISTANCE {
                continue;
            }

            let fov = angles_to_fov(&view_angles, &grenade.view_angles);
            if fov > align_config.fov {
                continue;
            }

            match best_grenade {
                Some((_, best_fov)) if fov < best_fov => {
                    best_grenade = Some((grenade, fov));
                }
                None => {
                    best_grenade = Some((grenade, fov));
                }
                _ => {}
            }
        }

        let Some((target_grenade, _)) = best_grenade else {
            self.grenade_align.is_aligning = false;
            return false;
        };

        let target_angle = target_grenade.view_angles;

        let mut aim_angles = view_angles - target_angle;
        if aim_angles.y < -180.0 {
            aim_angles.y += 360.0;
        }
        vec2_clamp(&mut aim_angles);

        let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);

        let smooth = (align_config.smooth + 1.0).clamp(1.0, 50.0);
        let mouse_angles = vec2(
            aim_angles.y / sensitivity * 45.45,
            -aim_angles.x / sensitivity * 45.45,
        ) / smooth;

        mouse.move_rel(mouse_angles);

        self.grenade_align.is_aligning = true;
        true
    }
}
