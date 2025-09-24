use std::time::{Duration, Instant};

use glam::Vec2;
use rand::{rng, Rng};

use crate::{
    config::{Config, TriggerbotMode},
    cs2::{CS2, bones::Bones, player::Player, weapon_class::WeaponClass},
    math::angles_to_fov,
    mouse::Mouse,
};
use strum::IntoEnumIterator;

#[derive(Debug, Default)]
pub struct Triggerbot {
    next_shot: Option<Instant>,
    previous_button_state: bool,
    pub(crate) active: bool,
    additional_shots_remaining: u32,
    additional_shot_delay: Option<Instant>,
}

impl CS2 {
    pub fn triggerbot(&mut self, config: &Config) {
        let hotkey = config.aim.triggerbot_hotkey;
        let triggerbot_config = self.triggerbot_config(config);

        if !triggerbot_config.enabled || self.trigger.next_shot.is_some() {
            return;
        }

        let button_state = self.is_button_down(&hotkey);
        if triggerbot_config.mode == TriggerbotMode::Hold && !button_state {
            return;
        } else {
            if button_state && !self.trigger.previous_button_state {
                self.trigger.active = !self.trigger.active;
            }
            self.trigger.previous_button_state = button_state;
            if !self.trigger.active {
                return;
            }
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        if triggerbot_config.flash_check && local_player.is_flashed(self) {
            return;
        }

        if triggerbot_config.scope_check
            && local_player.weapon_class(self) == WeaponClass::Sniper
            && !local_player.is_scoped(self)
        {
            return;
        }

        if triggerbot_config.velocity_check && local_player.velocity(self).length() > triggerbot_config.velocity_threshold
        {
            return;
        }

        let player = if let Some(crosshair_player) = local_player.crosshair_entity(self) {
            crosshair_player
        } else if !triggerbot_config.smoke_check {
            let view_angles = local_player.view_angles(self);
            let mut best_player: Option<Player> = None;
            let mut best_fov = 1.5;
            
            for player in &self.players {
                if !self.is_ffa() && player.team(self) == local_player.team(self) {
                    continue;
                }
                
                if !player.is_valid(self) {
                    continue;
                }
                
                for bone in Bones::iter() {
                    let bone_pos = player.bone_position(self, bone.u64());
                    let angle = self.angle_to_target(&local_player, &bone_pos, &Vec2::ZERO);
                    let fov = angles_to_fov(&view_angles, &angle);
                    
                    if fov < best_fov {
                        best_fov = fov;
                        best_player = Some(*player);
                    }
                }
            }
            
            if let Some(player) = best_player {
                player
            } else {
                return;
            }
        } else {
            return;
        };

        if !player.visible(self, &local_player) {
            return;
        }

        if !self.is_ffa() && player.team(self) == local_player.team(self) {
            return;
        }

        if triggerbot_config.head_only {
            let head = player.bone_position(self, Bones::Head.u64());

            let target_angle = self.angle_to_target(&local_player, &head, &Vec2::ZERO);
            let view_angles = local_player.view_angles(self);
            let fov = angles_to_fov(&view_angles, &target_angle);

            let head_radius_fov =
                3.5 / (local_player.position(self) - player.position(self)).length() * 100.0;

            if fov > head_radius_fov {
                return;
            }
        }

        let mean = (*triggerbot_config.delay.start() + *triggerbot_config.delay.end()) as f32 / 2.0;
        let std_dev = (*triggerbot_config.delay.end() - *triggerbot_config.delay.start()) as f32 / 2.0;

        let normal = rand_distr::Normal::new(mean, std_dev).unwrap();
        use rand_distr::Distribution as _;
        let delay = normal.sample(&mut rng()).max(0.0) as u64;

        self.trigger.next_shot = Some(Instant::now() + Duration::from_millis(delay));
        self.trigger.additional_shots_remaining = triggerbot_config.additional_shots;
    }

    pub fn triggerbot_shoot(&mut self, mouse: &mut Mouse) {
        if let Some(shot_time) = self.trigger.next_shot
            && Instant::now() >= shot_time
        {
            mouse.left_press();
            mouse.left_release();
            self.trigger.next_shot = None;
            
            if self.trigger.additional_shots_remaining > 0 {
                let additional_delay = 50 + (rand::rng().random::<u64>() % 100);
                self.trigger.additional_shot_delay = Some(Instant::now() + Duration::from_millis(additional_delay));
            }
        }
        
        if let Some(additional_shot_time) = self.trigger.additional_shot_delay
            && Instant::now() >= additional_shot_time
            && self.trigger.additional_shots_remaining > 0
        {
            mouse.left_press();
            mouse.left_release();
            self.trigger.additional_shots_remaining -= 1;
            
            if self.trigger.additional_shots_remaining > 0 {
                let next_delay = 40 + (rand::rng().random::<u64>() % 80);
                self.trigger.additional_shot_delay = Some(Instant::now() + Duration::from_millis(next_delay));
            } else {
                self.trigger.additional_shot_delay = None;
            }
        }
    }
}
