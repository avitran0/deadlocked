use egui::{Align2, Color32, Painter, Pos2, Shape, Stroke, Ui, pos2};
use glam::{Vec3, vec3};

use crate::{
    config::AimbotConfig,
    cs2::entity::weapon::Weapon,
    data::Data,
    math::world_to_screen,
    ui::{app::App, grenades::Grenade},
};

mod entity;
mod hud;
mod player;

impl App {
    fn aimbot_config(&self, weapon: &Weapon) -> &AimbotConfig {
        if let Some(weapon_config) = self.config.aim.weapons.get(weapon)
            && weapon_config.aimbot.enable_override
        {
            return &weapon_config.aimbot;
        }
        &self.config.aim.global.aimbot
    }

    pub fn overlay(&mut self, ui: &mut Ui) {
        ui.ctx().set_pixels_per_point(1.0);
        let painter = ui.layer_painter(egui::LayerId::background());

        self.update_trails();
        self.update_player_sounds();
        let (window_position, window_size) = {
            let data = self.data.lock();
            (data.window_position, data.window_size)
        };
        self.update_window(window_position, window_size);
        let data = &self.data.lock();
        self.overlay_debug(&painter, data);

        for player in &data.players {
            if data.esp_active {
                self.draw_player(&painter, player, data);
            }
        }

        if self.config.player.show_friendlies && data.is_custom_mode {
            for player in &data.friendlies {
                if data.esp_active {
                    self.draw_player(&painter, player, data);
                }
            }
        }

        if self.config.hud.dropped_weapons || self.config.hud.grenade_trails {
            for entity in &data.entities {
                self.draw_entity(&painter, entity, data);
            }
        }

        self.draw_bomb_timer(&painter, data);
        self.draw_fov_circle(&painter, data);
        self.draw_sniper_crosshair(&painter, data);
        self.draw_keybind_list(&painter, data);

        if data.aimbot_active {
            self.text(
                &painter,
                "aimbot active",
                pos2(
                    data.window_size.x / 2.0 + 8.0,
                    data.window_size.y / 2.0 + 8.0,
                ),
                Align2::LEFT_TOP,
                None,
            );
        }

        if data.triggerbot_active {
            self.text(
                &painter,
                "trigger active",
                pos2(
                    data.window_size.x / 2.0 + 8.0,
                    data.window_size.y / 2.0 + 8.0 + self.config.hud.font_size,
                ),
                Align2::LEFT_TOP,
                None,
            );
        }

        self.grenade_manager(data, &painter);
    }

    fn update_window(&mut self, window_position: glam::Vec2, window_size: glam::Vec2) {
        let Some(window) = &self.overlay else {
            return;
        };

        let new_pos = (window_position.x as i32, window_position.y as i32);
        if self.overlay_window_pos != Some(new_pos) {
            window
                .window()
                .set_outer_position(winit::dpi::PhysicalPosition::new(new_pos.0, new_pos.1));
            self.overlay_window_pos = Some(new_pos);
        }

        let new_size = (window_size.x.max(1.0) as u32, window_size.y.max(1.0) as u32);
        if self.overlay_window_size != Some(new_size) {
            let _ = window
                .window()
                .request_inner_size(winit::dpi::PhysicalSize::new(new_size.0, new_size.1));
            self.overlay_window_size = Some(new_size);
        }
    }

    fn grenade_manager(&self, data: &Data, painter: &Painter) {
        let position = data.local_player.position;
        let map = &data.map_name;

        let Some(grenades) = self.grenades.get(map) else {
            return;
        };

        let player_weapon = &data.local_player.weapon;
        for grenade in grenades {
            if *player_weapon != grenade.weapon {
                continue;
            }
            let distance = (position - grenade.position).length();
            if distance > 500.0 {
                continue;
            }
            self.grenade_circle(data, grenade, painter);

            if distance > 24.0 {
                continue;
            }
            self.grenade_indicator(data, grenade, painter);
        }
    }

    fn grenade_circle(&self, data: &Data, grenade: &Grenade, painter: &Painter) {
        let Some(center_screen) = world_to_screen(&grenade.position, data) else {
            return;
        };
        let center = &grenade.position;

        // player hitbox width and length
        const WIDTH: f32 = 24.0;
        const HALF_WIDTH: f32 = WIDTH / 2.0;

        const V1: Vec3 = vec3(WIDTH, HALF_WIDTH, 0.0);
        const V2: Vec3 = vec3(HALF_WIDTH, WIDTH, 0.0);
        const V3: Vec3 = vec3(-HALF_WIDTH, WIDTH, 0.0);
        const V4: Vec3 = vec3(-WIDTH, HALF_WIDTH, 0.0);

        let points: Vec<Pos2> = [
            center + V1,
            center + V2,
            center + V3,
            center + V4,
            center - V1,
            center - V2,
            center - V3,
            center - V4,
        ]
        .iter()
        .filter_map(|p| world_to_screen(p, data))
        .collect();

        let shape = Shape::convex_polygon(
            points,
            Color32::from_rgba_unmultiplied(0, 255, 0, 127),
            Stroke::NONE,
        );
        painter.add(shape);

        painter.circle_filled(
            center_screen,
            WIDTH / 8.0,
            Color32::from_rgba_unmultiplied(255, 0, 0, 127),
        );
    }

    fn grenade_indicator(&self, data: &Data, grenade: &Grenade, painter: &Painter) {
        let position = grenade.position + (data.local_player.head - data.local_player.position);
        let view_angles = grenade.view_angles;

        let pitch = view_angles.x.to_radians();
        let yaw = view_angles.y.to_radians();

        let forward = vec3(
            pitch.cos() * yaw.cos(),
            pitch.cos() * yaw.sin(),
            -pitch.sin(),
        )
        .normalize();

        const CROSS_DISTANCE: f32 = 1000.0;
        let center = position + forward * CROSS_DISTANCE;

        const WORLD_UP: Vec3 = vec3(0.0, 0.0, 1.0);
        const CROSS_SIZE: f32 = 24.0;

        let right = forward.cross(WORLD_UP).normalize();
        let up = right.cross(forward).normalize();

        let v1 = center + right * CROSS_SIZE + up * CROSS_SIZE;
        let v2 = center + right * CROSS_SIZE - up * CROSS_SIZE;
        let v3 = center - right * CROSS_SIZE + up * CROSS_SIZE;
        let v4 = center - right * CROSS_SIZE - up * CROSS_SIZE;

        let stroke = Stroke::new(self.config.hud.line_width, self.config.hud.text_color);
        let stroke_bg = Stroke::new(self.config.hud.line_width * 2.0, Color32::BLACK);

        let Some(v1) = world_to_screen(&v1, data) else {
            return;
        };
        let Some(v2) = world_to_screen(&v2, data) else {
            return;
        };
        let Some(v3) = world_to_screen(&v3, data) else {
            return;
        };
        let Some(v4) = world_to_screen(&v4, data) else {
            return;
        };

        painter.line_segment([v1, v4], stroke_bg);
        painter.line_segment([v2, v3], stroke_bg);

        painter.line_segment([v1, v4], stroke);
        painter.line_segment([v2, v3], stroke);

        let text_center = center - up * CROSS_SIZE;
        if let Some(text_center) = world_to_screen(&text_center, data) {
            self.text(
                painter,
                &grenade.name,
                text_center,
                Align2::CENTER_TOP,
                None,
            );
            let mut offset = self.config.hud.font_size;
            self.text(
                painter,
                format!("{}", grenade.weapon,),
                text_center + egui::vec2(0.0, offset),
                Align2::CENTER_TOP,
                None,
            );
            offset += self.config.hud.font_size;
            let text = match (
                grenade.modifiers.duck,
                grenade.modifiers.jump,
                grenade.modifiers.run,
            ) {
                (false, false, false) => "",
                (true, false, false) => "Duck",
                (true, true, false) => "Duck/Jump",
                (true, true, true) => "Duck/Jump/Run",
                (true, false, true) => "Duck/Run",
                (false, true, false) => "Jump",
                (false, true, true) => "Jump/Run",
                (false, false, true) => "Run",
            };
            if !text.is_empty() {
                self.text(
                    painter,
                    text,
                    text_center + egui::vec2(0.0, offset),
                    Align2::CENTER_TOP,
                    None,
                );
                offset += self.config.hud.font_size;
            }
            if !grenade.description.is_empty() {
                self.text(
                    painter,
                    &grenade.description,
                    text_center + egui::vec2(0.0, offset),
                    Align2::CENTER_TOP,
                    None,
                );
            }
        }
    }

    fn health_color(&self, health: i32, alpha: u8) -> Color32 {
        let health = health.clamp(0, 100);

        let (r, g) = if health <= 50 {
            let factor = health as f32 / 50.0;
            (255, (255.0 * factor) as u8)
        } else {
            let factor = 1.0 - (health - 50) as f32 / 50.0;
            ((255.0 * factor) as u8, 255)
        };

        Color32::from_rgba_unmultiplied(r, g, 0, alpha)
    }

    fn text(
        &self,
        painter: &Painter,
        text: impl AsRef<str>,
        position: Pos2,
        align: Align2,
        color: Option<Color32>,
    ) {
        use egui::FontId;

        let font = FontId::proportional(self.config.hud.font_size);
        let color = match color {
            Some(color) => color,
            None => self.config.hud.text_color,
        };
        if self.config.hud.text_outline {
            for (pos, color) in outline(position, color) {
                painter.text(pos, align, text.as_ref(), font.clone(), color);
            }
        } else {
            painter.text(position, align, text.as_ref(), font, color);
        }
    }
}

const OUTLINE_WIDTH: f32 = 1.0;
fn outline(pos: Pos2, color: Color32) -> [(Pos2, Color32); 5] {
    let outline_color = Color32::from_rgba_unmultiplied(0, 0, 0, color.a());
    [
        (
            pos2(pos.x - OUTLINE_WIDTH, pos.y - OUTLINE_WIDTH),
            outline_color,
        ),
        (
            pos2(pos.x + OUTLINE_WIDTH, pos.y - OUTLINE_WIDTH),
            outline_color,
        ),
        (
            pos2(pos.x - OUTLINE_WIDTH, pos.y + OUTLINE_WIDTH),
            outline_color,
        ),
        (
            pos2(pos.x + OUTLINE_WIDTH, pos.y + OUTLINE_WIDTH),
            outline_color,
        ),
        (pos, color),
    ]
}

fn convex_hull(points: &[Vec3]) -> Vec<Vec3> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut sorted_points = points.to_vec();
    sorted_points.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    });

    let mut deduped: Vec<Vec3> = Vec::with_capacity(sorted_points.len());
    for point in sorted_points {
        let is_duplicate = deduped
            .last()
            .is_some_and(|last| point.x == last.x && point.y == last.y);
        if !is_duplicate {
            deduped.push(point);
        }
    }

    if deduped.len() <= 2 {
        return deduped;
    }

    let mut lower = Vec::new();
    for point in &deduped {
        while lower.len() >= 2
            && cross(&lower[lower.len() - 2], &lower[lower.len() - 1], point) <= 0.0
        {
            lower.pop();
        }
        lower.push(*point);
    }

    let mut upper = Vec::new();
    for point in deduped.iter().rev() {
        while upper.len() >= 2
            && cross(&upper[upper.len() - 2], &upper[upper.len() - 1], point) <= 0.0
        {
            upper.pop();
        }
        upper.push(*point);
    }

    upper.pop();
    lower.pop();

    lower.append(&mut upper);
    lower
}

fn cross(o: &Vec3, a: &Vec3, b: &Vec3) -> f32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}
