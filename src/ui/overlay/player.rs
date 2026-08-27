use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use egui::{Color32, Painter, Pos2, Stroke, pos2};
use glam::vec3;

use crate::{
    config::player::{BoxMode, DrawMode},
    config::text::TextPosition,
    cs2::bones::Bones,
    data::{Data, PlayerData, SoundType},
    math::world_to_screen,
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

        let Some((tl, br)) = self.skeleton_bounds(player, data) else {
            return;
        };
        let tr = pos2(br.x, tl.y);
        let bl = pos2(tl.x, br.y);

        if self.config.player.draw_box != DrawMode::None {
            if self.config.player.box_mode == BoxMode::Gap {
                self.draw_gap_box(painter, tl, tr, bl, br, stroke);
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
                    pos2(
                        x,
                        bl.y - (delta * player.health as f32 / player.max_health.max(1) as f32),
                    ),
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
                player.weapon.to_icon(),
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

    fn skeleton_bounds(&self, player: &PlayerData, data: &Data) -> Option<(Pos2, Pos2)> {
        let mut screen_bones: HashMap<Bones, Pos2> =
            HashMap::with_capacity(Bones::CONNECTIONS.len() * 2);
        for (a, b) in &Bones::CONNECTIONS {
            for bone in [a, b] {
                if let Some(world) = player.bones.get(bone)
                    && let Some(screen) = world_to_screen(world, data)
                {
                    screen_bones.insert(*bone, screen);
                }
            }
        }

        if screen_bones.is_empty() {
            let midpoint = (player.position + player.head) / 2.0;
            let height = (player.head.z - player.position.z + 24.0).max(1.0);
            let half = height / 2.0;
            let top = midpoint + vec3(0.0, 0.0, half);
            let bottom = midpoint - vec3(0.0, 0.0, half);
            let top = world_to_screen(&top, data)?;
            let bottom = world_to_screen(&bottom, data)?;
            let hh = (bottom.y - top.y).max(1.0);
            let hw = hh / 4.0;
            return Some((pos2(top.x - hw, top.y), pos2(bottom.x + hw, bottom.y)));
        }

        let (tl, _tr, _bl, br) = Self::calculate_box_corners(&screen_bones)?;
        Some((tl, br))
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
