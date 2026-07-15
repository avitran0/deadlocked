use egui::{Color32, Painter, Pos2, Stroke, pos2, vec2};

use crate::{
    config::aim::KeyMode, config::text::TextPosition, cs2::entity::weapon_class::WeaponClass,
    data::Data, math::world_to_screen, ui::app::AppState,
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
        let color = self.health_color((fraction * 100.0) as i32, 255);
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
            || WeaponClass::from_string(data.weapon.as_ref()) != WeaponClass::Sniper
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
