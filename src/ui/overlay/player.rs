use std::time::{Duration, Instant};

use egui::{Align2, Color32, FontId, Painter, Stroke, pos2};
use glam::vec3;

use crate::{
    config::{BoxMode, DrawMode},
    cs2::bones::Bones,
    data::{Data, PlayerData, SoundType},
    math::world_to_screen,
    ui::app::App,
};

impl App {
    pub fn draw_player(&self, painter: &Painter, player: &PlayerData, data: &Data) {
        if self.config.player.visible_only && !player.visible {
            return;
        }

        let sound = self.player_sounds.get(&player.steam_id);
        let sound_alpha = if self.config.player.sound.enabled {
            self.player_sound_alpha(player, sound, data)
        } else {
            None
        };

        self.player_box(painter, player, data, sound_alpha);
        self.skeleton(painter, player, data, sound_alpha);
    }

    fn player_sound_alpha(
        &self,
        player: &PlayerData,
        sound: Option<&(Instant, SoundType)>,
        data: &Data,
    ) -> Option<f32> {
        if self.config.player.sound.show_visible && player.visible {
            return Some(1.0);
        }

        let Some((time, sound)) = sound else {
            return Some(0.0);
        };

        let local_player = &data.local_player;
        let max_distance = match sound {
            SoundType::Footstep => self.config.player.sound.footstep_diameter,
            SoundType::Gunshot => self.config.player.sound.gunshot_diameter,
            SoundType::Weapon => self.config.player.sound.weapon_diameter,
        };
        if local_player.position.distance(player.position) > max_distance {
            return Some(0.0);
        }

        if time.elapsed() > self.total_sound_duration() {
            return Some(0.0);
        }

        Some(
            1.0 - ((time.elapsed().as_secs_f32() - self.config.player.sound.fadeout_start)
                / self.config.player.sound.fadeout_duration),
        )
    }

    fn total_sound_duration(&self) -> Duration {
        Duration::from_secs_f32(
            self.config.player.sound.fadeout_start + self.config.player.sound.fadeout_duration,
        )
    }

    fn alpha(color: Color32, alpha: f32) -> Color32 {
        Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            (alpha.clamp(0.0, 1.0) * 255.0) as u8,
        )
    }

    fn player_box(&self, painter: &Painter, player: &PlayerData, data: &Data, alpha: Option<f32>) {
        use crate::config::DrawMode;

        let alpha = match alpha {
            Some(alpha) => alpha.clamp(0.0, 1.0),
            None => 1.0,
        };
        let distance = data
            .local_player
            .position
            .distance(player.position)
            .max(1.0);

        let esp_scale = (500.0 / distance).clamp(0.4, 1.0);
        let line_width = self.config.hud.line_width * esp_scale;

        let health_color =
            self.health_color(player.health, self.config.player.box_visible_color.a());
        let mut color = match &self.config.player.draw_box {
            DrawMode::None => health_color,
            DrawMode::Health => health_color,
            DrawMode::Color => {
                if player.visible {
                    self.config.player.box_visible_color
                } else {
                    self.config.player.box_invisible_color
                }
            }
        };

        color = Self::alpha(color, alpha);

        let stroke = Stroke::new(line_width, color);
        let icon_font = FontId::monospace(self.config.hud.icon_size * esp_scale);

        let midpoint = (player.position + player.head) / 2.0;
        let height = player.head.z - player.position.z + 24.0;
        let half_height = height / 2.0;
        let top = midpoint + vec3(0.0, 0.0, half_height);
        let bottom = midpoint - vec3(0.0, 0.0, half_height);

        let Some(top) = world_to_screen(&top, data) else {
            return;
        };
        let Some(bottom) = world_to_screen(&bottom, data) else {
            return;
        };
        let half_height = bottom.y - top.y;
        let width = half_height / 2.0;
        let half_width = width / 2.0;
        // quarter width
        let qw = half_width - 2.0;
        // eigth width
        let ew = qw / 2.0;

        let tl = pos2(top.x - half_width, top.y);
        let tr = pos2(top.x + half_width, top.y);
        let bl = pos2(bottom.x - half_width, bottom.y);
        let br = pos2(bottom.x + half_width, bottom.y);

        if self.config.player.draw_box != DrawMode::None {
            if self.config.player.box_mode == BoxMode::Gap {
                painter.line(
                    vec![pos2(tl.x + ew, tl.y), tl, pos2(tl.x, tl.y + qw)],
                    stroke,
                );
                painter.line(
                    vec![pos2(tr.x - ew, tl.y), tr, pos2(tr.x, tr.y + qw)],
                    stroke,
                );
                painter.line(
                    vec![pos2(bl.x + ew, bl.y), bl, pos2(bl.x, bl.y - qw)],
                    stroke,
                );
                painter.line(
                    vec![pos2(br.x - ew, bl.y), br, pos2(br.x, br.y - qw)],
                    stroke,
                );
            } else {
                painter.rect(
                    egui::Rect::from_min_max(tl, br),
                    0,
                    Color32::TRANSPARENT,
                    stroke,
                    egui::StrokeKind::Middle,
                );
            }
        }

        // health bar
        if self.config.player.health_bar {
            let x = bl.x - line_width * 2.0;
            let delta = bl.y - tl.y;
            painter.line(
                vec![
                    pos2(x, bl.y),
                    pos2(x, bl.y - (delta * player.health as f32 / 100.0)),
                ],
                Stroke::new(line_width, Self::alpha(health_color, alpha)),
            );
        }

        if self.config.player.armor_bar && player.armor > 0 {
            let x = bl.x
                - line_width
                    * if self.config.player.health_bar {
                        4.0
                    } else {
                        2.0
                    };
            let delta = bl.y - tl.y;
            painter.line(
                vec![
                    pos2(x, bl.y),
                    pos2(x, bl.y - (delta * player.armor as f32 / 100.0)),
                ],
                Stroke::new(line_width, Self::alpha(Color32::BLUE, alpha)),
            );
        }

        let mut offset = 0.0;
        let font_size = self.config.hud.font_size * esp_scale;
        let text_color = Self::alpha(self.config.hud.text_color, alpha);
        if self.config.player.player_name {
            self.text_sized(
                painter,
                &player.name,
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                Some(text_color),
                font_size,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_defuser {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "\u{e00f}",
                icon_font.clone(),
                text_color,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_helmet {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "\u{e017}",
                icon_font.clone(),
                text_color,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_bomb {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "\u{e01e}",
                icon_font.clone(),
                text_color,
            );
        }

        if self.config.player.weapon_icon {
            painter.text(
                pos2(bl.x + half_width, bl.y),
                Align2::CENTER_TOP,
                player.weapon.to_icon(),
                icon_font.clone(),
                text_color,
            );
            if player.ammo.0 >= 0 {
                self.text_sized(
                    painter,
                    format!("{}/{}", player.ammo.0, player.ammo.1),
                    pos2(bl.x + half_width, bl.y + font_size),
                    Align2::CENTER_TOP,
                    Some(text_color),
                    font_size,
                );
            }
        }
    }

    fn skeleton(&self, painter: &Painter, player: &PlayerData, data: &Data, alpha: Option<f32>) {
        let distance = data
            .local_player
            .position
            .distance(player.position)
            .max(1.0);
        let esp_scale = (500.0 / distance).clamp(0.25, 1.0);

        let mut color = match &self.config.player.draw_skeleton {
            DrawMode::None => return,
            DrawMode::Health => {
                self.health_color(player.health, self.config.player.skeleton_color.a())
            }
            DrawMode::Color => self.config.player.skeleton_color,
        };
        if let Some(alpha) = alpha {
            color = Self::alpha(color, alpha);
        }
        let stroke = Stroke::new(self.config.hud.line_width * esp_scale, color);

        for (a, b) in &Bones::CONNECTIONS {
            let Some(a) = player.bones.get(a) else {
                continue;
            };
            let Some(b) = player.bones.get(b) else {
                continue;
            };

            let Some(a) = world_to_screen(a, data) else {
                continue;
            };
            let Some(b) = world_to_screen(b, data) else {
                continue;
            };

            painter.line(vec![a, b], stroke);
        }

        // head circle
        if !self.config.player.head_circle {
            return;
        }
        let Some(neck) = player.bones.get(&Bones::Neck) else {
            return;
        };
        let Some(spine) = player.bones.get(&Bones::Spine3) else {
            return;
        };

        let Some(neck) = world_to_screen(neck, data) else {
            return;
        };
        let Some(spine) = world_to_screen(spine, data) else {
            return;
        };

        let height = spine.y - neck.y;
        let pos = pos2(neck.x - (spine.x - neck.x) / 2.0, neck.y - height / 2.0);
        painter.circle_stroke(pos, height / 2.0, stroke);
    }

    pub fn update_player_sounds(&mut self) {
        let data = self.data.lock();

        for player in &data.players {
            let Some(sound) = &player.sound else {
                continue;
            };

            self.player_sounds
                .insert(player.steam_id, (Instant::now(), *sound));
        }

        let total_duration = self.total_sound_duration();
        self.player_sounds
            .retain(|_, (time, _)| time.elapsed() < total_duration);
    }
}
