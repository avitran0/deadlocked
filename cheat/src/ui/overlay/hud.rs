use egui::{Align2, Color32, Painter, Pos2, Shape, Stroke, pos2, vec2};
use shared::{Data, PlayerData, WeaponClass};

use crate::{
    config::aim::KeyMode,
    config::text::TextPosition,
    math::{world_to_camera_xy, world_to_screen},
    ui::app::AppState,
};

impl AppState {
    pub fn overlay_debug(&self, painter: &Painter, data: &Data) {
        if self.config.hud.debug {
            painter.line(
                vec![pos2(0.0, 0.0), pos2(data.window_size.x, data.window_size.y)],
                Stroke::new(self.config.hud.line_width, Color32::WHITE),
            );
            painter.line(
                vec![pos2(data.window_size.x, 0.0), pos2(0.0, data.window_size.y)],
                Stroke::new(self.config.hud.line_width, Color32::WHITE),
            );
        }
    }

    pub fn draw_bomb_timer(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.bomb_timer || !data.bomb.planted {
            return;
        }

        if let Some(pos) = world_to_screen(&data.bomb.position, data) {
            let cat = &self.config.hud.overlay_text.bomb_timer;
            let anchor = point_anchor(pos, cat.position, cat.font_size * 0.3);
            self.text_sized(
                painter,
                format!("{:.3}", data.bomb.timer),
                anchor,
                cat.align.to_align2(),
                cat.color,
                cat.font_size,
            );
            if data.bomb.being_defused {
                self.text_sized(
                    painter,
                    format!("defusing {:.3}", data.bomb.defuse_remain_time),
                    anchor + vec2(0.0, cat.font_size),
                    cat.align.to_align2(),
                    cat.color,
                    cat.font_size,
                );
            }
        }

        let fraction = (data.bomb.timer / 40.0).clamp(0.0, 1.0);
        let color = self.health_color((fraction * 100.0) as i32, 100, 255);
        painter.line(
            vec![
                pos2(0.0, data.window_size.y),
                pos2(data.window_size.x * fraction, data.window_size.y),
            ],
            Stroke::new(self.config.hud.line_width * 3.0, color),
        );
    }

    pub fn draw_fov_circle(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.fov_circle || !data.in_game {
            return;
        }

        let weapon_config = self.aimbot_config(&data.weapon);

        if !weapon_config.enabled || (weapon_config.mode == KeyMode::Toggle && !data.aimbot_active)
        {
            return;
        }

        let aim_fov = weapon_config.fov;

        if weapon_config.distance_adjusted_fov {
            self.draw_distance_scaled_fov_circle(painter, data, aim_fov, 125.0, Color32::GREEN);
            self.draw_distance_scaled_fov_circle(painter, data, aim_fov, 250.0, Color32::YELLOW);
            self.draw_distance_scaled_fov_circle(painter, data, aim_fov, 500.0, Color32::RED);
        } else {
            self.draw_simple_fov_circle(painter, data, aim_fov, Color32::WHITE);
        }
    }

    pub fn draw_keybind_list(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.keybind_list {
            return;
        }

        let cat = &self.config.hud.overlay_text.keybind_list;
        let position = screen_anchor(
            [data.window_size.x, data.window_size.y],
            cat.position,
            10.0,
            0.0,
        );
        let aimbot_color = if data.aimbot_active {
            Color32::GREEN
        } else {
            cat.color
        };
        self.text_sized(
            painter,
            format!("Aimbot: {:?}", self.config.aim.aimbot_hotkey),
            position,
            cat.align.to_align2(),
            aimbot_color,
            cat.font_size,
        );

        let triggerbot_color = if data.triggerbot_active {
            Color32::GREEN
        } else {
            cat.color
        };
        self.text_sized(
            painter,
            format!("Triggerbot: {:?}", self.config.aim.triggerbot_hotkey),
            position + vec2(0.0, cat.font_size),
            cat.align.to_align2(),
            triggerbot_color,
            cat.font_size,
        );
    }

    pub fn draw_spectator_list(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.spectator_list {
            return;
        }

        let cat = &self.config.hud.overlay_text.spectator_list;
        let position = screen_anchor(
            [data.window_size.x, data.window_size.y],
            cat.position,
            10.0,
            cat.font_size * 3.0,
        );
        self.text_sized(
            painter,
            "Spectators:",
            position,
            cat.align.to_align2(),
            cat.color,
            cat.font_size,
        );

        for (i, name) in data.spectators.iter().enumerate() {
            self.text_sized(
                painter,
                format!("> {name}"),
                position + vec2(0.0, cat.font_size * (i as f32 + 1.0)),
                cat.align.to_align2(),
                cat.color,
                cat.font_size,
            );
        }
    }

    fn get_current_fov(&self) -> f32 {
        (if self.config.misc.fov_changer {
            self.config.misc.desired_fov
        } else {
            crate::constants::cs2::DEFAULT_FOV
        }) as f32
    }

    fn calculate_fov_radius(&self, data: &Data, target_fov: f32) -> f32 {
        let current_fov = self.get_current_fov();
        let screen_width = data.window_size.x;

        let current_fov_tan = (current_fov.to_radians() / 2.0).tan();
        if current_fov_tan == 0.0 {
            return 0.0;
        }

        let target_fov_tan = (target_fov.to_radians() / 2.0).tan();
        (target_fov_tan / current_fov_tan) * (screen_width / 2.0)
    }

    fn draw_fov_circle_impl(&self, painter: &Painter, data: &Data, radius: f32, color: Color32) {
        let center = pos2(data.window_size.x / 2.0, data.window_size.y / 2.0);
        let stroke = Stroke::new(self.config.hud.line_width, color);
        painter.circle_stroke(center, radius, stroke);
    }

    fn get_distance_fov_scale(&self, distance: f32) -> f32 {
        (5.0 - (distance / 125.0)).max(1.0)
    }

    fn draw_simple_fov_circle(
        &self,
        painter: &Painter,
        data: &Data,
        target_fov: f32,
        color: Color32,
    ) {
        let radius = self.calculate_fov_radius(data, target_fov);
        self.draw_fov_circle_impl(painter, data, radius, color);
    }

    fn draw_distance_scaled_fov_circle(
        &self,
        painter: &Painter,
        data: &Data,
        base_aim_fov: f32,
        distance: f32,
        color: Color32,
    ) {
        let scale = self.get_distance_fov_scale(distance);
        let target_fov = base_aim_fov * scale;

        let radius = self.calculate_fov_radius(data, target_fov);
        self.draw_fov_circle_impl(painter, data, radius, color);
    }

    pub fn draw_sniper_crosshair(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.sniper_crosshair.enabled
            || data.weapon.weapon_class() != WeaponClass::Sniper
        {
            return;
        }

        let length = self.config.hud.sniper_crosshair.line_length;
        let gap = self.config.hud.sniper_crosshair.gap / 2.0;
        let center = data.window_size / 2.0;

        let stroke = Stroke::new(
            self.config.hud.sniper_crosshair.line_width,
            self.config.hud.sniper_crosshair.color,
        );

        painter.line_segment(
            [
                pos2(center.x + gap, center.y),
                pos2(center.x + gap + length, center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x, center.y + gap),
                pos2(center.x, center.y + gap + length),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x - gap, center.y),
                pos2(center.x - gap - length, center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x, center.y - gap),
                pos2(center.x, center.y - gap - length),
            ],
            stroke,
        );
    }

    pub fn draw_minimap(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.minimap.enabled || !data.in_game {
            return;
        }

        let cfg = self.config.hud.minimap.clone();
        let visible_color = self.config.player.box_visible_color;
        let size = cfg.size.max(80.0);
        let range = cfg.range.max(1.0);
        let pad = 12.0;
        let origin = minimap_origin(
            [data.window_size.x, data.window_size.y],
            cfg.position,
            size,
            pad,
        );
        let rect = egui::Rect::from_min_size(origin, egui::vec2(size, size));
        let center = rect.center();
        let radius = size * 0.5 - 6.0;
        let alpha = (cfg.opacity.clamp(0.15, 1.0) * 255.0) as u8;
        let icon = cfg.icon_size.clamp(3.0, 16.0);

        let bg = with_alpha(cfg.background, alpha);
        painter.circle_filled(center, radius + 3.0, Color32::from_rgba_unmultiplied(0, 0, 0, 90));
        painter.circle_filled(center, radius, bg);
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(1.6, Color32::from_rgba_unmultiplied(180, 220, 255, 55)),
        );
        painter.circle_stroke(
            center,
            radius - 3.0,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 18)),
        );

        if cfg.show_rings {
            for t in [0.33, 0.66] {
                painter.circle_stroke(
                    center,
                    radius * t,
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 22)),
                );
            }
        }

        let tick = Color32::from_rgba_unmultiplied(255, 255, 255, 28);
        painter.line_segment(
            [pos2(center.x, center.y - radius + 8.0), pos2(center.x, center.y - 6.0)],
            Stroke::new(1.0, tick),
        );
        painter.line_segment(
            [pos2(center.x - 5.0, center.y), pos2(center.x + 5.0, center.y)],
            Stroke::new(1.0, tick),
        );

        let local = data.local_player.position;
        let yaw = data.view_angles.y;
        let rotate = cfg.rotate_with_view;

        let to_map = |world: glam::Vec3| -> Option<(Pos2, bool)> {
            let cam = if rotate {
                world_to_camera_xy(local, world, yaw)
            } else {
                glam::Vec2::new(world.x - local.x, world.y - local.y)
            };
            let mut sx = cam.x / range;
            let mut sy = cam.y / range;
            let len = (sx * sx + sy * sy).sqrt();
            let mut on_edge = false;
            if len > 1.0 {
                if !cfg.clamp_to_edge {
                    return None;
                }
                sx /= len;
                sy /= len;
                on_edge = true;
            }
            Some((pos2(center.x + sx * radius, center.y - sy * radius), on_edge))
        };

        let facing = |player_yaw: f32| -> egui::Vec2 {
            let fwd = glam::Vec2::new(player_yaw.to_radians().cos(), player_yaw.to_radians().sin());
            let mapped = if rotate {
                let (right, forward) = crate::math::camera_basis_xy(yaw);
                glam::Vec2::new(fwd.dot(right), fwd.dot(forward))
            } else {
                fwd
            };
            vec2(mapped.x, -mapped.y)
        };

        let draw_player = |player: &PlayerData, color: Color32, draw_name: bool| {
            let Some((pos, on_edge)) = to_map(player.position) else {
                return;
            };
            let dir = facing(player.rotation);
            radar_arrow(painter, pos, dir, color, icon);
            if on_edge {
                painter.circle_stroke(pos, icon * 0.85, Stroke::new(1.0, color));
            }
            if draw_name && cfg.show_names && !player.name.is_empty() {
                self.text_sized(
                    painter,
                    &player.name,
                    pos2(pos.x, pos.y + icon + 1.0),
                    Align2::CENTER_TOP,
                    color,
                    (icon * 1.35).clamp(8.0, 13.0),
                );
            }
        };

        for player in &data.players {
            let color = if player.visible {
                visible_color
            } else {
                cfg.enemy_color
            };
            draw_player(player, color, true);
        }

        if cfg.show_teammates {
            for player in &data.friendlies {
                draw_player(player, cfg.teammate_color, true);
            }
        }

        if cfg.show_bomb && data.bomb.planted {
            if let Some((pos, _)) = to_map(data.bomb.position) {
                let half = icon * 0.7;
                painter.rect_filled(
                    egui::Rect::from_center_size(pos, egui::vec2(half * 2.0, half * 2.0)),
                    1.5,
                    cfg.bomb_color,
                );
            }
        }

        for entity in &data.entities {
            match entity {
                shared::EntityInfo::Smoke(info) if cfg.show_smoke => {
                    if let Some((pos, _)) = to_map(info.position) {
                        painter.circle_filled(pos, icon * 0.9, cfg.smoke_color);
                        painter.circle_stroke(
                            pos,
                            icon * 1.35,
                            Stroke::new(1.0, with_alpha(cfg.smoke_color, 140)),
                        );
                    }
                }
                shared::EntityInfo::Molotov(info) if cfg.show_molotov => {
                    if let Some((pos, _)) = to_map(info.position) {
                        painter.circle_filled(pos, icon * 0.85, cfg.molotov_color);
                    }
                }
                shared::EntityInfo::Inferno(info) if cfg.show_molotov => {
                    if let Some((pos, _)) = to_map(info.position) {
                        painter.circle_filled(pos, icon * 1.1, cfg.molotov_color);
                        painter.circle_stroke(
                            pos,
                            icon * 1.6,
                            Stroke::new(1.2, with_alpha(cfg.molotov_color, 160)),
                        );
                    }
                }
                _ => {}
            }
        }

        let local_dir = if rotate {
            vec2(0.0, -1.0)
        } else {
            facing(yaw)
        };
        radar_arrow(painter, center, local_dir, cfg.local_color, icon * 1.15);
        painter.circle_filled(center, 1.6, cfg.local_color);
    }
}

fn radar_arrow(painter: &Painter, pos: Pos2, dir: egui::Vec2, color: Color32, size: f32) {
    let len = dir.length();
    let dir = if len < 0.001 {
        vec2(0.0, -size)
    } else {
        dir / len * size
    };
    let perp = vec2(-dir.y, dir.x) * 0.48;
    let tip = pos2(pos.x + dir.x, pos.y + dir.y);
    let left = pos2(pos.x - dir.x * 0.65 + perp.x, pos.y - dir.y * 0.65 + perp.y);
    let right = pos2(pos.x - dir.x * 0.65 - perp.x, pos.y - dir.y * 0.65 - perp.y);
    painter.add(Shape::convex_polygon(
        vec![tip, left, right],
        color,
        Stroke::NONE,
    ));
}

fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    let [r, g, b, _] = color.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, alpha)
}

fn minimap_origin(size: [f32; 2], position: TextPosition, map_size: f32, pad: f32) -> Pos2 {
    let [w, h] = size;
    match position {
        TextPosition::TopLeft => pos2(pad, pad),
        TextPosition::TopCenter => pos2((w - map_size) * 0.5, pad),
        TextPosition::TopRight => pos2(w - map_size - pad, pad),
        TextPosition::CenterLeft => pos2(pad, (h - map_size) * 0.5),
        TextPosition::Center => pos2((w - map_size) * 0.5, (h - map_size) * 0.5),
        TextPosition::CenterRight => pos2(w - map_size - pad, (h - map_size) * 0.5),
        TextPosition::BottomLeft => pos2(pad, h - map_size - pad),
        TextPosition::BottomCenter => pos2((w - map_size) * 0.5, h - map_size - pad),
        TextPosition::BottomRight => pos2(w - map_size - pad, h - map_size - pad),
    }
}

pub fn point_anchor(point: Pos2, position: TextPosition, offset: f32) -> Pos2 {
    match position {
        TextPosition::TopLeft => point + vec2(-offset, -offset),
        TextPosition::TopCenter => point + vec2(0.0, -offset),
        TextPosition::TopRight => point + vec2(offset, -offset),
        TextPosition::CenterLeft => point + vec2(-offset, 0.0),
        TextPosition::Center => point,
        TextPosition::CenterRight => point + vec2(offset, 0.0),
        TextPosition::BottomLeft => point + vec2(-offset, offset),
        TextPosition::BottomCenter => point + vec2(0.0, offset),
        TextPosition::BottomRight => point + vec2(offset, offset),
    }
}

pub fn screen_anchor(size: [f32; 2], position: TextPosition, pad_x: f32, offset_y: f32) -> Pos2 {
    let [w, h] = size;
    match position {
        TextPosition::TopLeft => pos2(pad_x, offset_y),
        TextPosition::TopCenter => pos2(w / 2.0, offset_y),
        TextPosition::TopRight => pos2(w - pad_x, offset_y),
        TextPosition::CenterLeft => pos2(pad_x, h / 2.0 + offset_y),
        TextPosition::Center => pos2(w / 2.0, h / 2.0 + offset_y),
        TextPosition::CenterRight => pos2(w - pad_x, h / 2.0 + offset_y),
        TextPosition::BottomLeft => pos2(pad_x, h + offset_y),
        TextPosition::BottomCenter => pos2(w / 2.0, h + offset_y),
        TextPosition::BottomRight => pos2(w - pad_x, h + offset_y),
    }
}
