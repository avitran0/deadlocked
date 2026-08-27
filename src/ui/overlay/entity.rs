use std::{collections::HashMap, time::Instant};

use egui::{Color32, Painter, Pos2, Stroke, vec2};

use crate::{
    config::player::DrawMode,
    cs2::entity::{
        EntityInfo, GrenadeInfo, InfernoInfo, MolotovInfo,
        chicken::{ChickenBones, ChickenInfo},
    },
    data::Data,
    math::world_to_screen,
    ui::{app::AppState, overlay::convex_hull, trail::Trail},
};

impl AppState {
    pub fn draw_entity(&self, painter: &Painter, entity: &EntityInfo, data: &Data) {
        match entity {
            EntityInfo::Weapon {
                weapon,
                position,
                ammo,
            } => {
                if !self.config.hud.dropped_weapons {
                    return;
                }
                let Some(screen) = world_to_screen(position, data) else {
                    return;
                };
                let cat = &self.config.hud.overlay_text.weapon_name;
                let anchor = super::hud::point_anchor(screen, cat.position, cat.font_size * 0.3);
                self.text_sized(
                    painter,
                    format!("{weapon}"),
                    anchor,
                    cat.align.to_align2(),
                    cat.color,
                    cat.font_size,
                );
                if ammo.0 >= 0 {
                    self.text_sized(
                        painter,
                        format!("{}/{}", ammo.0, ammo.1),
                        anchor + vec2(0.0, cat.font_size),
                        cat.align.to_align2(),
                        cat.color,
                        cat.font_size,
                    );
                }
            }
            EntityInfo::Inferno(inferno) => self.inferno(painter, data, inferno),
            EntityInfo::Molotov(molotov) => self.molotov(painter, data, molotov),
            EntityInfo::Smoke(info) => {
                self.draw_grenade(painter, data, info, self.config.hud.grenade_trails.smoke)
            }
            EntityInfo::Flashbang(info) => {
                self.draw_grenade(painter, data, info, self.config.hud.grenade_trails.flash)
            }
            EntityInfo::HeGrenade(info) => {
                self.draw_grenade(painter, data, info, self.config.hud.grenade_trails.he)
            }
            EntityInfo::Decoy(info) => {
                self.draw_grenade(painter, data, info, self.config.hud.grenade_trails.decoy)
            }
            EntityInfo::ChickenInfo(info) => self.draw_chicken(painter, data, info),
        };
    }

    fn draw_grenade(
        &self,
        painter: &Painter,
        data: &Data,
        info: &GrenadeInfo,
        trail_color: Color32,
    ) {
        if !self.config.hud.grenade_trails.enabled {
            return;
        }
        let Some(screen) = world_to_screen(&info.position, data) else {
            return;
        };
        let cat = &self.config.hud.overlay_text.grenade_name;
        let anchor = super::hud::point_anchor(screen, cat.position, cat.font_size * 0.3);
        self.text_sized(
            painter,
            &info.name,
            anchor,
            cat.align.to_align2(),
            cat.color,
            cat.font_size,
        );

        if !self.config.hud.grenade_trails.enabled {
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

        if !self.config.hud.grenade_trails.enabled || !self.config.hud.grenade_trails.inferno_poly {
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
            Color32::from_rgba_unmultiplied(255, 0, 0, 127),
            Stroke::NONE,
        );
        painter.add(shape);

        self.draw_grenade(painter, data, &inferno.grenade(), Color32::TRANSPARENT);
    }

    fn molotov(&self, painter: &Painter, data: &Data, molotov: &MolotovInfo) {
        if !self.config.hud.grenade_trails.enabled {
            return;
        }
        if molotov.is_incendiary {
            self.draw_grenade(
                painter,
                data,
                &molotov.grenade(),
                self.config.hud.grenade_trails.incendiary,
            );
        } else {
            self.draw_grenade(
                painter,
                data,
                &molotov.grenade(),
                self.config.hud.grenade_trails.molotov,
            );
        }
    }

    pub fn update_trails(&mut self) {
        let data = self.data.lock();
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

        let now = Instant::now();
        self.trails
            .retain(|_k, trail| now.duration_since(trail.last_update) < Trail::MAX_AGE);
    }

    fn draw_chicken(&self, painter: &Painter, data: &Data, chicken: &ChickenInfo) {
        if !self.config.player.chicken {
            return;
        }

        let screen_bones: HashMap<ChickenBones, Pos2> = chicken
            .bones
            .iter()
            .filter_map(|(bone, pos)| world_to_screen(pos, data).map(|s| (*bone, s)))
            .collect();

        if screen_bones.is_empty() {
            return;
        }

        // box
        if self.config.player.draw_box != DrawMode::None {
            let Some((tl, tr, bl, br)) = Self::calculate_box_corners(&screen_bones) else {
                return;
            };

            let box_color = if chicken.visible {
                self.config.player.box_visible_color
            } else {
                self.config.player.box_invisible_color
            };
            let stroke = Stroke::new(self.config.hud.line_width, box_color);
            self.draw_gap_box(painter, tl, tr, bl, br, stroke);
        }

        // skeleton
        let color = match &self.config.player.draw_skeleton {
            DrawMode::None => return,
            DrawMode::Health => self.health_color(100, 100, self.config.player.skeleton_color.a()),
            DrawMode::Color => self.config.player.skeleton_color,
        };

        let stroke = Stroke::new(self.config.hud.line_width, color);
        for (bone_a, bone_b) in &ChickenBones::CONNECTIONS {
            let (Some(a_screen), Some(b_screen)) =
                (screen_bones.get(bone_a), screen_bones.get(bone_b))
            else {
                continue;
            };

            painter.line_segment([*a_screen, *b_screen], stroke);
        }
    }
}
