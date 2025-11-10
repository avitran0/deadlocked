use std::time::{Duration, Instant};

use egui::{Align2, Color32, Context, FontId, Painter, Pos2, Shape, Stroke, pos2};
use glam::{Vec3, vec3};

use crate::{
    bvh::{Aabb, Triangle},
    config::{AimbotConfig, BoxMode, DrawMode},
    cs2::{
        bones::Bones,
        entity::{
            EntityInfo, GrenadeInfo, inferno::InfernoInfo, molotov::MolotovInfo, smoke::SmokeInfo,
            weapon::Weapon, weapon_class::WeaponClass,
        },
    },
    data::{Data, PlayerData},
    math::world_to_screen,
    ui::{app::App, color::Colors, grenades::Grenade, trail::Trail},
};

impl App {
    fn aimbot_config(&self, weapon: &Weapon) -> &AimbotConfig {
        if let Some(weapon_config) = self.config.aim.weapons.get(weapon)
            && weapon_config.aimbot.enable_override
        {
            return &weapon_config.aimbot;
        }
        &self.config.aim.global.aimbot
    }

    pub fn overlay(&mut self, ctx: &Context) {
        ctx.set_pixels_per_point(1.0);
        let painter = ctx.layer_painter(egui::LayerId::background());

        self.add_trails();
        let data = &self.data.lock().unwrap();
        if let Some(window) = &self.overlay_window {
            window
                .window()
                .set_outer_position(winit::dpi::PhysicalPosition::new(
                    data.window_position.x,
                    data.window_position.y,
                ));
            let _ = window
                .window()
                .request_inner_size(winit::dpi::PhysicalSize::new(
                    data.window_size.x.max(1.0),
                    data.window_size.y.max(1.0),
                ));
        }

        if self.config.hud.debug {
            painter.line(
                vec![pos2(0.0, 0.0), pos2(data.window_size.x, data.window_size.y)],
                Stroke::new(self.config.hud.line_width, self.apply_alpha(Color32::WHITE)),
            );
            painter.line(
                vec![pos2(data.window_size.x, 0.0), pos2(0.0, data.window_size.y)],
                Stroke::new(self.config.hud.line_width, self.apply_alpha(Color32::WHITE)),
            );
        }

        if data.wallhack_active {
            for player in &data.players {
                self.player_box(&painter, player, data);
                self.skeleton(&painter, player, data);
            }

            if self.config.player.show_friendlies && data.is_custom_mode {
                for player in &data.friendlies {
                    self.player_box(&painter, player, data);
                    self.skeleton(&painter, player, data);
                }
            }
        }

        let now = Instant::now();
        let max_age = Duration::from_secs(1);
        self.trails
            .retain(|_k, trail| now - trail.last_update < max_age);

        if self.config.hud.dropped_weapons || self.config.hud.grenade_trails {
            for entity in &data.entities {
                self.entity(&painter, entity, data);
            }
        }

        if self.config.hud.bomb_timer && data.bomb.planted {
            if let Some(pos) = world_to_screen(&data.bomb.position, data) {
                self.text(
                    &painter,
                    format!("{:.3}", data.bomb.timer),
                    pos,
                    Align2::CENTER_CENTER,
                    None,
                );
                if data.bomb.being_defused {
                    self.text(
                        &painter,
                        format!("defusing {:.3}", data.bomb.defuse_remain_time),
                        pos2(pos.x, pos.y + self.config.hud.font_size),
                        Align2::CENTER_CENTER,
                        None,
                    );
                }
            }

            let fraction = (data.bomb.timer / 40.0).clamp(0.0, 1.0);
            let color = self.apply_alpha(self.health_color((fraction * 100.0) as i32));
            painter.line(
                vec![
                    pos2(0.0, data.window_size.y),
                    pos2(data.window_size.x * fraction, data.window_size.y),
                ],
                Stroke::new(self.config.hud.line_width * 3.0, color),
            );
        }

        // fov circle
        if self.config.hud.fov_circle && data.in_game {
            let weapon_config = self.aimbot_config(&data.weapon);
            let aim_fov = weapon_config.fov;

            if weapon_config.distance_adjusted_fov {
                self.draw_distance_scaled_fov_circle(
                    &painter,
                    data,
                    aim_fov,
                    125.0,
                    self.apply_alpha(Color32::GREEN),
                );
                self.draw_distance_scaled_fov_circle(
                    &painter,
                    data,
                    aim_fov,
                    250.0,
                    self.apply_alpha(Color32::YELLOW),
                );
                self.draw_distance_scaled_fov_circle(
                    &painter,
                    data,
                    aim_fov,
                    500.0,
                    self.apply_alpha(Color32::RED),
                );
            } else {
                self.draw_simple_fov_circle(
                    &painter,
                    data,
                    aim_fov,
                    self.apply_alpha(Colors::TEXT),
                );
            }
        }

        if self.config.hud.sniper_crosshair
            && WeaponClass::from_string(data.weapon.as_ref()) == WeaponClass::Sniper
        {
            painter.line(
                vec![
                    pos2(data.window_size.x / 2.0, data.window_size.y / 2.0 - 50.0),
                    pos2(data.window_size.x / 2.0, data.window_size.y / 2.0 + 50.0),
                ],
                Stroke::new(
                    self.config.hud.line_width,
                    self.apply_alpha(self.config.hud.crosshair_color),
                ),
            );
            painter.line(
                vec![
                    pos2(data.window_size.x / 2.0 - 50.0, data.window_size.y / 2.0),
                    pos2(data.window_size.x / 2.0 + 50.0, data.window_size.y / 2.0),
                ],
                Stroke::new(
                    self.config.hud.line_width,
                    self.apply_alpha(self.config.hud.crosshair_color),
                ),
            );
        }

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

        if self.config.hud.spectators {
            let mut spectators_watching_me = Vec::new();
            for (spectator_name, target_id) in &data.spectator_names {
                if *target_id == data.local_player.steam_id {
                    spectators_watching_me.push(spectator_name.clone());
                }
            }

            if !spectators_watching_me.is_empty() {
                let mut offset = 10.0;

                self.text(
                    &painter,
                    format!("{} watching you", spectators_watching_me.len()),
                    pos2(data.window_size.x - 300.0, offset),
                    Align2::LEFT_TOP,
                    Some(Color32::from_rgb(255, 100, 100)),
                );
                offset += self.config.hud.font_size + 5.0;

                for spectator_name in spectators_watching_me {
                    self.text(
                        &painter,
                        format!("• {}", spectator_name),
                        pos2(data.window_size.x - 290.0, offset),
                        Align2::LEFT_TOP,
                        Some(Color32::WHITE),
                    );
                    offset += self.config.hud.font_size;
                }
            }
        }

        self.grenade_manager(data, &painter);
    }

    #[allow(unused)]
    fn triangle(&self, triangle: &Triangle, data: &Data, painter: &Painter) {
        let Some(v1) = world_to_screen(&triangle.v0, data) else {
            return;
        };
        let Some(v2) = world_to_screen(&triangle.v1, data) else {
            return;
        };
        let Some(v3) = world_to_screen(&triangle.v2, data) else {
            return;
        };
        painter.line(vec![v1, v2, v3], Stroke::new(1.0, Color32::WHITE));
    }

    #[allow(unused)]
    fn aabb_box(&self, aabb: &Aabb, data: &Data, painter: &Painter) {
        let min = aabb.min();
        let max = aabb.max();

        let corners = [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(max.x, max.y, max.z),
        ];

        let screen_points: Vec<Option<Pos2>> =
            corners.iter().map(|p| world_to_screen(p, data)).collect();

        let edges = [
            (0, 1),
            (1, 3),
            (3, 2),
            (2, 0),
            (4, 5),
            (5, 7),
            (7, 6),
            (6, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];

        for (i, j) in edges.iter() {
            if let (Some(p0), Some(p1)) = (screen_points[*i], screen_points[*j]) {
                painter.line_segment([p0, p1], Stroke::new(1.0, Color32::WHITE));
            }
        }
    }

    fn player_box(&self, painter: &Painter, player: &PlayerData, data: &Data) {
        use crate::config::DrawMode;

        let health_color = self.apply_alpha(self.health_color(player.health));
        let color = match &self.config.player.draw_box {
            DrawMode::None => health_color,
            DrawMode::Health => health_color,
            DrawMode::Color => {
                if player.visible {
                    self.apply_alpha(self.config.player.box_visible_color)
                } else {
                    self.apply_alpha(self.config.player.box_invisible_color)
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
                Stroke::new(self.config.hud.line_width, self.apply_alpha(Color32::BLUE)),
            );
        }

        let mut offset = 0.0;
        let font_size = self.config.hud.font_size;
        let text_color = self.apply_alpha(self.config.hud.text_color);
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
                "r",
                icon_font.clone(),
                text_color,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_helmet {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "q",
                icon_font.clone(),
                text_color,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_bomb {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "o",
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
            DrawMode::Health => self.apply_alpha(self.health_color(player.health)),
            DrawMode::Color => self.apply_alpha(self.config.player.skeleton_color),
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

    #[cfg(feature = "visuals")]
    fn entity(&self, painter: &Painter, entity: &EntityInfo, data: &Data) {
        match entity {
            EntityInfo::Weapon { weapon, position } => {
                use egui::FontId;

                if !self.config.hud.dropped_weapons {
                    return;
                }
                let Some(position) = world_to_screen(position, data) else {
                    return;
                };
                painter.text(
                    position,
                    Align2::CENTER_CENTER,
                    format!("{weapon}"),
                    FontId::proportional(self.config.hud.font_size),
                    self.config.hud.text_color,
                );
            }
            EntityInfo::Inferno(inferno) => self.inferno(painter, data, inferno),
            EntityInfo::Smoke(smoke) => self.smoke(painter, data, smoke),
            EntityInfo::Molotov(molotov) => self.molotov(painter, data, molotov),
            EntityInfo::Flashbang(info) => {
                self.draw_grenade(painter, data, info, self.config.hud.flash_trail_color)
            }
            EntityInfo::HeGrenade(info) => {
                self.draw_grenade(painter, data, info, self.config.hud.he_trail_color)
            }
            EntityInfo::Decoy(info) => {
                self.draw_grenade(painter, data, info, self.config.hud.decoy_trail_color)
            }
        };
    }

    fn draw_grenade(
        &self,
        painter: &Painter,
        data: &Data,
        info: &GrenadeInfo,
        trail_color: Color32,
    ) {
        if !self.config.hud.grenade_trails {
            return;
        }
        let Some(position) = world_to_screen(&info.position, data) else {
            return;
        };
        self.text(painter, info.name, position, Align2::CENTER_CENTER, None);

        if !self.config.hud.grenade_trails {
            return;
        }

        let stroke = Stroke::new(self.config.hud.line_width, trail_color);
        let Some(trail) = self.trails.get(&info.entity) else {
            return;
        };
        for window in trail.trail.windows(2) {
            if let [v1, v2] = window {
                use crate::math::world_to_screen;

                let Some(v1) = world_to_screen(v1, data) else {
                    continue;
                };
                let Some(v2) = world_to_screen(v2, data) else {
                    continue;
                };
                painter.line_segment([v1, v2], stroke);
            }
        }
    }

    fn inferno(&self, painter: &Painter, data: &Data, inferno: &InfernoInfo) {
        use egui::Shape;

        if !self.config.hud.grenade_trails {
            return;
        }
        let hull: Vec<Pos2> = convex_hull(&inferno.hull)
            .iter()
            .filter_map(|p| {
                use crate::math::world_to_screen;

                let p = p + (p - inferno.position).clamp_length(60.0, 60.0);
                world_to_screen(&p, data)
            })
            .collect();
        if hull.len() < 3 {
            return;
        }

        let shape = Shape::convex_polygon(
            hull,
            Color32::from_rgba_premultiplied(255, 0, 0, 127),
            Stroke::NONE,
        );
        painter.add(shape);

        self.draw_grenade(painter, data, &inferno.grenade(), Color32::TRANSPARENT);
    }

    fn smoke(&self, painter: &Painter, data: &Data, smoke: &SmokeInfo) {
        if !self.config.hud.grenade_trails {
            return;
        }
        self.draw_grenade(
            painter,
            data,
            &smoke.grenade(),
            self.config.hud.smoke_trail_color,
        );
    }

    fn molotov(&self, painter: &Painter, data: &Data, molotov: &MolotovInfo) {
        if !self.config.hud.grenade_trails {
            return;
        }
        if molotov.is_incendiary {
            self.draw_grenade(
                painter,
                data,
                &molotov.grenade(),
                self.config.hud.incendiary_trail_color,
            );
        } else {
            self.draw_grenade(
                painter,
                data,
                &molotov.grenade(),
                self.config.hud.molotov_trail_color,
            );
        }
    }

    fn add_trails(&mut self) {
        let data = self.data.lock().unwrap();
        for entity in &data.entities {
            let (entity, position) = match entity {
                EntityInfo::Inferno(info) => (info.entity, info.position),
                EntityInfo::Smoke(info) => (info.entity, info.position),
                EntityInfo::Molotov(info) => (info.entity, info.position),
                EntityInfo::Flashbang(info) | EntityInfo::HeGrenade(info) => {
                    (info.entity, info.position)
                }
                _ => continue,
            };
            if let Some(trail) = self.trails.get_mut(&entity) {
                if (position - trail.trail.last().unwrap()).length() < 1.0 {
                    continue;
                }
                trail.trail.push(position);
                trail.last_update = Instant::now();
            } else {
                self.trails.insert(
                    entity,
                    Trail {
                        trail: vec![position],
                        last_update: Instant::now(),
                    },
                );
            }
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
            Color32::from_rgba_premultiplied(0, 255, 0, 127),
            Stroke::NONE,
        );
        painter.add(shape);

        painter.circle_filled(
            center_screen,
            WIDTH / 8.0,
            Color32::from_rgba_premultiplied(255, 0, 0, 127),
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

    fn health_color(&self, health: i32) -> Color32 {
        let health = health.clamp(0, 100);

        let (r, g) = if health <= 50 {
            let factor = health as f32 / 50.0;
            (255, (255.0 * factor) as u8)
        } else {
            let factor = 1.0 - (health - 50) as f32 / 50.0;
            ((255.0 * factor) as u8, 255)
        };

        Color32::from_rgb(r, g, 0)
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
            None => self.apply_alpha(self.config.hud.text_color),
        };
        if self.config.hud.text_outline {
            for (pos, color) in outline(position, color) {
                painter.text(pos, align, text.as_ref(), font.clone(), color);
            }
        } else {
            painter.text(position, align, text.as_ref(), font, color);
        }
    }

    fn apply_alpha(&self, color: Color32) -> Color32 {
        let [r, g, b, _] = color.to_array();
        let alpha = (255.0 * self.config.player.alpha) as u8;
        Color32::from_rgba_unmultiplied(r, g, b, alpha)
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

    fn draw_fov_circle(&self, painter: &Painter, data: &Data, radius: f32, color: Color32) {
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
        self.draw_fov_circle(painter, data, radius, color);
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
        self.draw_fov_circle(painter, data, radius, color);
    }
}

const OUTLINE_WIDTH: f32 = 1.0;
fn outline(pos: Pos2, color: Color32) -> [(Pos2, Color32); 5] {
    [
        (
            pos2(pos.x - OUTLINE_WIDTH, pos.y - OUTLINE_WIDTH),
            Color32::BLACK,
        ),
        (
            pos2(pos.x + OUTLINE_WIDTH, pos.y - OUTLINE_WIDTH),
            Color32::BLACK,
        ),
        (
            pos2(pos.x - OUTLINE_WIDTH, pos.y + OUTLINE_WIDTH),
            Color32::BLACK,
        ),
        (
            pos2(pos.x + OUTLINE_WIDTH, pos.y + OUTLINE_WIDTH),
            Color32::BLACK,
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

    let mut deduped: Vec<Vec3> = Vec::new();
    for point in sorted_points {
        if deduped.is_empty()
            || point.x != deduped.last().unwrap().x
            || point.y != deduped.last().unwrap().y
        {
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
