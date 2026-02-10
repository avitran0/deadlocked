use std::time::Instant;

use egui::{Align2, Color32, FontId, Painter, Stroke, pos2};
use glam::vec3;

use crate::{
    config::{BoxMode, DrawMode},
    cs2::bones::Bones,
    data::{Data, PlayerData},
    math::world_to_screen,
    ui::app::App,
};

impl App {
    pub fn draw_player(&self, painter: &Painter, player: &PlayerData, data: &Data) {
        if self.config.player.visiblity_check && !player.visible {
            return;
        }

        self.player_box(painter, player, data);
        self.skeleton(painter, player, data);
    }

    fn player_box(&self, painter: &Painter, player: &PlayerData, data: &Data) {
        use crate::config::DrawMode;

        let health_color =
            self.health_color(player.health, self.config.player.box_visible_color.a());
        let color = match &self.config.player.draw_box {
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
        let stroke = Stroke::new(self.config.hud.line_width, color);
        let icon_font = FontId::monospace(self.config.hud.icon_size);

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
            let x = bl.x - self.config.hud.line_width * 2.0;
            let delta = bl.y - tl.y;
            painter.line(
                vec![
                    pos2(x, bl.y),
                    pos2(x, bl.y - (delta * player.health as f32 / 100.0)),
                ],
                Stroke::new(self.config.hud.line_width, health_color),
            );
        }

        if self.config.player.armor_bar && player.armor > 0 {
            let x = bl.x
                - self.config.hud.line_width
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
                Stroke::new(self.config.hud.line_width, Color32::BLUE),
            );
        }

        let mut offset = 0.0;
        let font_size = self.config.hud.font_size;
        let text_color = self.config.hud.text_color;
        if self.config.player.player_name {
            self.text(
                painter,
                &player.name,
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                None,
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
        }
    }

    fn skeleton(&self, painter: &Painter, player: &PlayerData, data: &Data) {
        let color = match &self.config.player.draw_skeleton {
            DrawMode::None => return,
            DrawMode::Health => {
                self.health_color(player.health, self.config.player.skeleton_color.a())
            }
            DrawMode::Color => self.config.player.skeleton_color,
        };
        let stroke = Stroke::new(self.config.hud.line_width, color);

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

        self.player_sounds
            .retain(|_, (time, _)| time.elapsed() < self.config.player.sound.fadeout_duration);
    }
}
