use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use egui::{Color32, Painter, Pos2, Stroke, pos2};
use shared::{Bones, Data, PlayerData, SoundType};

use crate::{
    config::player::{BoxMode, DrawMode},
    config::text::TextPosition,
    math::{CYLINDER_SAMPLES, world_to_camera_xy, world_to_screen},
    ui::app::AppState,
};

impl AppState {
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

        let health_color = self.health_color(
            player.health,
            player.max_health,
            self.config.player.box_visible_color.a(),
        );
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

        let Some((tl, tr, bl, br)) = self.projected_world_bounds(player, data) else {
            return;
        };

        if self.config.player.draw_box != DrawMode::None {
            if self.config.player.box_mode == BoxMode::Gap {
                self.draw_projected_gap_box(painter, tl, tr, bl, br, stroke);
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

        let edge = tl - bl;
        let edge_length = edge.length();
        if edge_length > f32::EPSILON {
            let direction = edge / edge_length;
            let outward = egui::vec2(direction.y, -direction.x);
            let mut offset = outward * line_width * 2.0;
            let draw_bar = |offset: egui::Vec2, fraction: f32, color: Color32| {
                let start = bl + offset;
                painter.line_segment(
                    [start, start + edge * fraction.clamp(0.0, 1.0)],
                    Stroke::new(line_width, Self::alpha(color, alpha)),
                );
            };
            if self.config.player.health_bar {
                draw_bar(
                    offset,
                    player.health.max(0) as f32 / player.max_health.max(1) as f32,
                    health_color,
                );
                offset += outward * line_width * 2.0;
            }
            if self.config.player.armor_bar && player.armor > 0 {
                draw_bar(offset, player.armor as f32 / 100.0, Color32::BLUE);
            }
        }

        let pad = 4.0 * esp_scale;
        let mut offset = 0.0;

        if self.config.player.player_name {
            let cat = &self.config.hud.overlay_text.player_name;
            let fs = cat.font_size * esp_scale;
            let anchor = self.box_anchor(tl, tr, bl, br, cat.position, pad, offset);
            self.text_sized(
                painter,
                &player.name,
                anchor,
                cat.align.to_align2(),
                Self::alpha(cat.color, alpha),
                fs,
            );
            offset += fs;
        }

        if self.config.player.tags {
            let cat = &self.config.hud.overlay_text.player_tags;
            let fs = cat.font_size * esp_scale;
            let anchor = self.box_anchor(tl, tr, bl, br, cat.position, pad, offset);
            if player.has_defuser {
                self.text_sized(
                    painter,
                    "\u{e00f}",
                    anchor,
                    cat.align.to_align2(),
                    Self::alpha(cat.color, alpha),
                    fs,
                );
                offset += fs;
            }
            if player.has_helmet {
                let anchor = self.box_anchor(tl, tr, bl, br, cat.position, pad, offset);
                self.text_sized(
                    painter,
                    "\u{e017}",
                    anchor,
                    cat.align.to_align2(),
                    Self::alpha(cat.color, alpha),
                    fs,
                );
                offset += fs;
            }
            if player.has_bomb {
                let anchor = self.box_anchor(tl, tr, bl, br, cat.position, pad, offset);
                self.text_sized(
                    painter,
                    "\u{e01e}",
                    anchor,
                    cat.align.to_align2(),
                    Self::alpha(cat.color, alpha),
                    fs,
                );
            }
        }

        if self.config.player.weapon_icon {
            let icon_cat = &self.config.hud.overlay_text.weapon_icon;
            let ammo_cat = &self.config.hud.overlay_text.ammo_text;
            let ifs = icon_cat.font_size * esp_scale;
            let afs = ammo_cat.font_size * esp_scale;
            let icon_anchor = self.box_anchor(tl, tr, bl, br, icon_cat.position, 0.0, 0.0);
            self.text_sized(
                painter,
                player.weapon.to_icon().to_string(),
                icon_anchor,
                icon_cat.align.to_align2(),
                Self::alpha(icon_cat.color, alpha),
                ifs,
            );
            if player.ammo.0 >= 0 {
                let ammo_anchor = self.box_anchor(tl, tr, bl, br, ammo_cat.position, 0.0, afs);
                self.text_sized(
                    painter,
                    format!("{}/{}", player.ammo.0, player.ammo.1),
                    ammo_anchor,
                    ammo_cat.align.to_align2(),
                    Self::alpha(ammo_cat.color, alpha),
                    afs,
                );
            }
        }
    }

    pub fn calculate_box_corners<K>(
        screen_bones: &HashMap<K, Pos2>,
    ) -> Option<(Pos2, Pos2, Pos2, Pos2)> {
        let screen_positions: Vec<&Pos2> = screen_bones.values().collect();

        if screen_positions.len() < 2 {
            return None;
        }

        let min_x = screen_positions
            .iter()
            .map(|p| p.x)
            .reduce(f32::min)
            .unwrap();
        let max_x = screen_positions
            .iter()
            .map(|p| p.x)
            .reduce(f32::max)
            .unwrap();
        let min_y = screen_positions
            .iter()
            .map(|p| p.y)
            .reduce(f32::min)
            .unwrap();
        let max_y = screen_positions
            .iter()
            .map(|p| p.y)
            .reduce(f32::max)
            .unwrap();

        let margin_x = (max_x - min_x) * 0.1;
        let margin_y = (max_y - min_y) * 0.1;

        Some((
            pos2(min_x - margin_x, min_y - margin_y),
            pos2(max_x + margin_x, min_y - margin_y),
            pos2(min_x - margin_x, max_y + margin_y),
            pos2(max_x + margin_x, max_y + margin_y),
        ))
    }

    fn draw_projected_gap_box(
        &self,
        painter: &Painter,
        tl: Pos2,
        tr: Pos2,
        bl: Pos2,
        br: Pos2,
        stroke: Stroke,
    ) {
        let width = (tr - tl).length();
        let height = (bl - tl).length();
        let gap = width.max(height) / 8.0;
        let corner = width.max(height) / 4.0 - 2.0;
        let mark = |point: Pos2, first: Pos2, second: Pos2| {
            painter.line(
                vec![
                    point + (first - point).normalized() * gap,
                    point,
                    point + (second - point).normalized() * corner,
                ],
                stroke,
            );
        };
        mark(tl, tr, bl);
        mark(tr, tl, br);
        mark(bl, br, tl);
        mark(br, bl, tr);
    }

    pub fn draw_gap_box(
        &self,
        painter: &Painter,
        tl: Pos2,
        tr: Pos2,
        bl: Pos2,
        br: Pos2,
        stroke: Stroke,
    ) {
        let gap_size = (tr.x - tl.x) / 8.0; // eighth of width
        let corner_length = (tr.x - tl.x) / 4.0 - 2.0; // quarter width minus small offset

        painter.line(
            vec![
                pos2(tl.x + gap_size, tl.y),
                tl,
                pos2(tl.x, tl.y + corner_length),
            ],
            stroke,
        );

        painter.line(
            vec![
                pos2(tr.x - gap_size, tr.y),
                tr,
                pos2(tr.x, tr.y + corner_length),
            ],
            stroke,
        );

        painter.line(
            vec![
                pos2(bl.x + gap_size, bl.y),
                bl,
                pos2(bl.x, bl.y - corner_length),
            ],
            stroke,
        );

        painter.line(
            vec![
                pos2(br.x - gap_size, br.y),
                br,
                pos2(br.x, br.y - corner_length),
            ],
            stroke,
        );
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
            DrawMode::Health => self.health_color(
                player.health,
                player.max_health,
                self.config.player.skeleton_color.a(),
            ),
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

    #[allow(clippy::too_many_arguments)]
    fn box_anchor(
        &self,
        tl: Pos2,
        tr: Pos2,
        bl: Pos2,
        br: Pos2,
        position: TextPosition,
        pad: f32,
        offset: f32,
    ) -> Pos2 {
        let top = pos2((tl.x + tr.x) / 2.0, tl.y);
        let bottom = pos2((bl.x + br.x) / 2.0, bl.y);
        let center = pos2((tl.x + br.x) / 2.0, (tl.y + bl.y) / 2.0);
        let center_left = pos2(tl.x, (tl.y + bl.y) / 2.0);
        let center_right = pos2(tr.x, (tr.y + br.y) / 2.0);
        match position {
            TextPosition::TopLeft => pos2(tl.x + pad, tl.y + offset),
            TextPosition::TopCenter => pos2(top.x, tl.y + offset),
            TextPosition::TopRight => pos2(tr.x + pad, tr.y + offset),
            TextPosition::CenterLeft => pos2(center_left.x + pad, center_left.y + offset),
            TextPosition::Center => pos2(center.x, center.y + offset),
            TextPosition::CenterRight => pos2(center_right.x + pad, center_right.y + offset),
            TextPosition::BottomLeft => pos2(bl.x + pad, bl.y + offset),
            TextPosition::BottomCenter => pos2(bottom.x, bl.y + offset),
            TextPosition::BottomRight => pos2(br.x + pad, bl.y + offset),
        }
    }

    fn projected_world_bounds(
        &self,
        player: &PlayerData,
        data: &Data,
    ) -> Option<(Pos2, Pos2, Pos2, Pos2)> {
        let min = player.collision_mins;
        let max = player.collision_maxs;
        if !min.is_finite() || !max.is_finite() || min.cmpgt(max).any() || min == max {
            return None;
        }
        const SAMPLES: usize = CYLINDER_SAMPLES;
        let center = (min + max) * 0.5;
        let radius = 0.5 * (max.x - min.x).abs().max((max.y - min.y).abs());
        let mut points = [Pos2::ZERO; CYLINDER_SAMPLES * 2];
        for layer in 0..2 {
            let z = if layer == 0 { min.z } else { max.z };
            for i in 0..SAMPLES {
                let angle = std::f32::consts::TAU * i as f32 / SAMPLES as f32;
                let local = glam::Vec3::new(
                    center.x + radius * angle.cos(),
                    center.y + radius * angle.sin(),
                    z,
                );
                points[layer * SAMPLES + i] =
                    world_to_screen(&player.collision_transform.transform_point3(local), data)?;
            }
        }
        let left = (0..SAMPLES).min_by(|a, b| {
            points[*a]
                .x
                .min(points[*a + SAMPLES].x)
                .partial_cmp(&points[*b].x.min(points[*b + SAMPLES].x))
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
        let right = (0..SAMPLES).max_by(|a, b| {
            points[*a]
                .x
                .max(points[*a + SAMPLES].x)
                .partial_cmp(&points[*b].x.max(points[*b + SAMPLES].x))
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
        Some((
            points[left + SAMPLES],
            points[right + SAMPLES],
            points[left],
            points[right],
        ))
    }

    pub fn draw_behind_players(&self, painter: &Painter, data: &Data) {
        if !self.config.player.behind_players.enabled || !data.in_game {
            return;
        }

        let cfg = self.config.player.behind_players.clone();
        let w = data.window_size.x;
        let h = data.window_size.y;
        if w < 32.0 || h < 32.0 {
            return;
        }

        let pad = 28.0;
        let y = h - pad;
        let marker_size = cfg.size.max(10.0);

        let draw_marker = |player: &PlayerData| {
            let target = if player.head.length_squared() > 1.0 {
                player.head
            } else {
                player.position
            };
            if world_to_screen(&target, data).is_some() {
                return;
            }

            let cam = world_to_camera_xy(
                data.local_player.position,
                player.position,
                data.view_angles.y,
            );
            let abs_x = cam.x.abs();
            let abs_y = cam.y.abs().max(1.0);

            // Behind → bottom edge. Left/right (including off-screen sides) → side edges.
            let pos = if cam.y < 0.0 && abs_y >= abs_x {
                let x = (w * 0.5
                    + (cam.x / -cam.y).clamp(-1.0, 1.0) * (w * 0.5 - pad))
                    .clamp(pad, w - pad);
                pos2(x, y)
            } else if cam.x < 0.0 {
                let sy = (h * 0.5
                    - (cam.y / abs_x).clamp(-1.0, 1.0) * (h * 0.35))
                    .clamp(pad, h - pad);
                pos2(pad, sy)
            } else {
                let sy = (h * 0.5
                    - (cam.y / abs_x.max(1.0)).clamp(-1.0, 1.0) * (h * 0.35))
                    .clamp(pad, h - pad);
                pos2(w - pad, sy)
            };

            let color = if player.visible {
                cfg.visible_color
            } else {
                cfg.color
            };

            let tip = if pos.x <= pad + 1.0 {
                pos2(pos.x - 8.0, pos.y)
            } else if pos.x >= w - pad - 1.0 {
                pos2(pos.x + 8.0, pos.y)
            } else {
                pos2(pos.x, pos.y + 8.0)
            };
            let left = if tip.y > pos.y {
                pos2(pos.x - 7.0, pos.y - 4.0)
            } else if tip.x < pos.x {
                pos2(pos.x + 4.0, pos.y - 7.0)
            } else {
                pos2(pos.x - 4.0, pos.y - 7.0)
            };
            let right = if tip.y > pos.y {
                pos2(pos.x + 7.0, pos.y - 4.0)
            } else if tip.x < pos.x {
                pos2(pos.x + 4.0, pos.y + 7.0)
            } else {
                pos2(pos.x - 4.0, pos.y + 7.0)
            };
            painter.add(egui::Shape::convex_polygon(
                vec![tip, left, right],
                color,
                Stroke::NONE,
            ));
            self.text_sized(
                painter,
                "!",
                pos2(pos.x, pos.y - marker_size * 0.35),
                egui::Align2::CENTER_BOTTOM,
                color,
                marker_size,
            );
        };

        for player in &data.players {
            draw_marker(player);
        }
        if self.config.player.show_friendlies {
            for player in &data.friendlies {
                draw_marker(player);
            }
        }
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
