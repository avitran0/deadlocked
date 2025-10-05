use crate::{config::Config, mouse::Mouse};

use super::{player::Player, CS2};

#[derive(Debug)]
pub struct Bhop {
    jump_count: u32,
    prev_view_angle: Option<f32>,
    keys_released: bool,
}

impl Default for Bhop {
    fn default() -> Self {
        Self {
            jump_count: 0,
            prev_view_angle: None,
            keys_released: true,
        }
    }
}

impl CS2 {
    pub fn bhop(&mut self, config: &Config, mouse: &mut Mouse) {
        if config.misc.bhop.enabled {
            let Some(local_player) = Player::local_player(self) else {
                return;
            };

            if self.is_button_down(&config.misc.bhop.hotkey) {
                if local_player.is_on_ground(self) {
                    if self.bhop_state.jump_count >= 5 {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        self.bhop_state.jump_count = 0;
                    }

                    let scroll_value = rand::random::<i32>().abs() % 2 + 1;
                    mouse.scroll_wheel(scroll_value);
                    
                    self.bhop_state.jump_count += 1;
                    
                    std::thread::sleep(std::time::Duration::from_millis(25));
                }
            }
        }

        if config.misc.bhop.strafe_helper {
            let Some(local_player) = Player::local_player(self) else {
                if !self.bhop_state.keys_released {
                    mouse.release_a();
                    mouse.release_d();
                    self.bhop_state.keys_released = true;
                }
                return;
            };

            if self.is_button_down(&config.misc.bhop.strafe_hotkey) && !local_player.is_on_ground(self) {
                let current_view_angle = local_player.view_angles(self).y;

                if let Some(prev_angle) = self.bhop_state.prev_view_angle {
                    let angle_diff = current_view_angle - prev_angle;
                    
                    if angle_diff.abs() > 0.05 {
                        if angle_diff > 0.0 {
                            mouse.press_a();
                            mouse.release_d();
                            self.bhop_state.keys_released = false;
                        } else {
                            mouse.press_d();    
                            mouse.release_a();
                            self.bhop_state.keys_released = false;
                        }
                    }
                }

                self.bhop_state.prev_view_angle = Some(current_view_angle);
            } else {
                if !self.bhop_state.keys_released {
                    mouse.release_a();
                    mouse.release_d();
                    self.bhop_state.keys_released = true;
                }
                self.bhop_state.prev_view_angle = None;
            }
        } else {
            if !self.bhop_state.keys_released {
                mouse.release_a();
                mouse.release_d();
                self.bhop_state.keys_released = true;
            }
        }
    }
}
