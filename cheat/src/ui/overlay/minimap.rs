use std::collections::HashMap;
use std::sync::Mutex;

use egui::{
    Color32, ColorImage, Context, Painter, Pos2, Rect, Stroke, TextureHandle, TextureOptions, pos2,
    vec2,
};
use epaint::{Mesh, Vertex};
use glam::Vec3;
use shared::{Data, EntityInfo, PlayerData, WeaponClass};

use crate::ui::{app::AppState, color::Colors, overlay::hud::screen_anchor};

struct MapCalibration {
    pos_x: f32,
    pos_y: f32,
    scale: f32,
}

impl MapCalibration {
    /// mirrors the web radar's worldToRadar()
    fn world_to_percent(&self, world: Vec3) -> (f32, f32) {
        (
            (world.x - self.pos_x) / self.scale / 1024.0 * 100.0,
            (self.pos_y - world.y) / self.scale / 1024.0 * 100.0,
        )
    }
}

fn map_calibration(name: &str) -> Option<MapCalibration> {
    let (pos_x, pos_y, scale) = match name {
        "ar_baggage" | "ar_baggage_lower" => (-1316.0, 1288.0, 2.539062),
        "ar_shoots" | "ar_shoots_night" => (-1368.0, 1952.0, 2.6875),
        "cs_italy" => (-2647.0, 2592.0, 4.6),
        "cs_office" => (-1838.0, 1858.0, 4.1),
        "de_ancient" | "de_ancient_night" | "de_ancient_v1" | "de_ancient_v2" => {
            (-2953.0, 2164.0, 5.0)
        }
        "de_anubis" => (-2796.0, 3328.0, 5.22),
        "de_cache" => (-2000.0, 3250.0, 5.5),
        "de_dust" => (-2850.0, 4073.0, 6.0),
        "de_dust2" => (-2476.0, 3239.0, 4.4),
        "de_inferno" | "de_inferno_s2" => (-2087.0, 3870.0, 4.9),
        "de_mirage" => (-3230.0, 1713.0, 5.0),
        "de_nuke" | "de_nuke_lower" => (-3453.0, 2887.0, 7.0),
        "de_overpass" | "de_overpass_2v2" => (-4831.0, 1781.0, 5.2),
        "de_train" | "de_train_lower" => (-2308.0, 2078.0, 4.082077),
        "de_vertigo" | "de_vertigo_lower" => (-3168.0, 1762.0, 4.0),
        "cs_shelter" => (-3448.7712, 3805.3228, 3.311857),
        "de_boulder" => (-3273.4917, 2930.2207, 2.8491137),
        "de_debris" => (-3015.2393, 2919.2393, 1.9445695),
        "de_eldorado" => (-3548.6814, 2571.1816, 2.0656068),
        "de_fachwerk" => (-2311.0767, 2874.6204, 2.5537856),
        "de_poseidon" => (-1046.3943, 1166.3942, 3.0124886),
        _ => return None,
    };
    Some(MapCalibration {
        pos_x,
        pos_y,
        scale,
    })
}

fn map_image_bytes(name: &str) -> Option<&'static [u8]> {
    Some(match name {
        "ar_baggage" => include_bytes!("../../../assets/maps/ar_baggage.png"),
        "ar_baggage_lower" => include_bytes!("../../../assets/maps/ar_baggage_lower.png"),
        "ar_shoots" => include_bytes!("../../../assets/maps/ar_shoots.png"),
        "ar_shoots_night" => include_bytes!("../../../assets/maps/ar_shoots_night.png"),
        "cs_italy" => include_bytes!("../../../assets/maps/cs_italy.png"),
        "cs_office" => include_bytes!("../../../assets/maps/cs_office.png"),
        "de_ancient" => include_bytes!("../../../assets/maps/de_ancient.png"),
        "de_ancient_night" => include_bytes!("../../../assets/maps/de_ancient_night.png"),
        "de_ancient_v1" => include_bytes!("../../../assets/maps/de_ancient_v1.png"),
        "de_anubis" => include_bytes!("../../../assets/maps/de_anubis.png"),
        "de_cache" => include_bytes!("../../../assets/maps/de_cache.png"),
        "de_dust2" => include_bytes!("../../../assets/maps/de_dust2.png"),
        "de_inferno" => include_bytes!("../../../assets/maps/de_inferno.png"),
        "de_mirage" => include_bytes!("../../../assets/maps/de_mirage.png"),
        "de_nuke" => include_bytes!("../../../assets/maps/de_nuke.png"),
        "de_nuke_lower" => include_bytes!("../../../assets/maps/de_nuke_lower.png"),
        "de_overpass" => include_bytes!("../../../assets/maps/de_overpass.png"),
        "de_train" => include_bytes!("../../../assets/maps/de_train.png"),
        "de_train_lower" => include_bytes!("../../../assets/maps/de_train_lower.png"),
        "de_vertigo" => include_bytes!("../../../assets/maps/de_vertigo.png"),
        "de_vertigo_lower" => include_bytes!("../../../assets/maps/de_vertigo_lower.png"),
        "de_ancient_v2" => include_bytes!("../../../assets/maps/de_ancient.png"),
        "de_inferno_s2" => include_bytes!("../../../assets/maps/de_inferno.png"),
        "de_overpass_2v2" => include_bytes!("../../../assets/maps/de_overpass.png"),
        _ => return None,
    })
}

#[derive(Default)]
pub struct MinimapState {
    textures: Mutex<HashMap<String, TextureHandle>>,
    ghosts: Mutex<HashMap<u64, GhostEntry>>,
}

/// last known state of a player who has dropped off the tracked list; called
/// "last seen" rather than "death" since a dormant/out-of-range player looks
/// identical to a dead one here
struct GhostEntry {
    position: Vec3,
    color: Color32,
    missing_since: Option<f64>,
}

impl MinimapState {
    fn get_or_load(&self, ctx: &Context, name: &str) -> Option<TextureHandle> {
        let mut cache = self.textures.lock().unwrap();
        if let Some(handle) = cache.get(name) {
            return Some(handle.clone());
        }

        let bytes = map_image_bytes(name)?;
        let image = image::load_from_memory(bytes).ok()?.to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, image.as_raw());
        let handle = ctx.load_texture(name.to_owned(), color_image, TextureOptions::LINEAR);
        cache.insert(name.to_owned(), handle.clone());
        Some(handle)
    }
}

struct Projector {
    calibration: MapCalibration,
    ref_percent: (f32, f32),
    rotation_rad: f32,
    zoom: f32,
    box_size: f32,
    center: Pos2,
}

impl Projector {
    fn project(&self, world: Vec3) -> Pos2 {
        let (px, py) = self.calibration.world_to_percent(world);
        let ex = (px - self.ref_percent.0) / 100.0 * self.box_size;
        let ey = (py - self.ref_percent.1) / 100.0 * self.box_size;
        let (sin, cos) = self.rotation_rad.sin_cos();
        let rx = ex * cos - ey * sin;
        let ry = ex * sin + ey * cos;
        self.center + vec2(rx * self.zoom, ry * self.zoom)
    }
}

impl AppState {
    pub fn draw_minimap(&self, ctx: &Context, painter: &Painter, data: &Data) {
        let config = &self.config.hud.minimap;

        if data.in_game && config.hide_native_minimap {
            // measured against a 2560x1440 screenshot, only accurate at that res
            painter.circle_filled(pos2(200.0, 195.0), 190.0, Colors::BACKDROP);
        }

        if !config.enabled {
            return;
        }

        let box_size = config.size;
        let anchor = screen_anchor(
            [data.window_size.x, data.window_size.y],
            config.position,
            box_size / 2.0 + 4.0,
            box_size / 2.0 + 4.0,
        );
        let rect = Rect::from_center_size(anchor, vec2(box_size, box_size));

        if !data.in_game {
            painter.rect_filled(rect, 4.0, Color32::from_black_alpha(160));
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(1.5, Colors::SUBTEXT),
                egui::StrokeKind::Outside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "not in a match",
                egui::FontId::proportional(13.0),
                Colors::SUBTEXT,
            );
            return;
        }

        let Some(calibration) = map_calibration(&data.map_name) else {
            painter.rect_filled(rect, 4.0, Color32::from_black_alpha(160));
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(1.5, Colors::RED),
                egui::StrokeKind::Outside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("no minimap data for\n\"{}\"", data.map_name),
                egui::FontId::proportional(13.0),
                Colors::TEXT,
            );
            return;
        };

        painter.rect_filled(rect, 4.0, Color32::from_black_alpha(160));

        let centering = config.center_on_self && data.local_player.steam_id != 0;
        let ref_percent = if centering {
            calibration.world_to_percent(data.local_player.position)
        } else {
            (50.0, 50.0)
        };
        let rotation_deg = if centering {
            data.local_player.rotation - 90.0
        } else {
            0.0
        };
        let rotation_rad = rotation_deg.to_radians();

        if let Some(texture) = self.minimap.get_or_load(ctx, &data.map_name) {
            draw_rotated_map(
                painter,
                texture.id(),
                rect.center(),
                box_size * config.zoom,
                rotation_rad,
                ref_percent,
            );
        }

        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.5, Colors::HIGHLIGHT),
            egui::StrokeKind::Outside,
        );

        let projector = Projector {
            calibration,
            ref_percent,
            rotation_rad,
            zoom: config.zoom,
            box_size,
            center: rect.center(),
        };

        let painter = painter.with_clip_rect(rect);
        let painter = &painter;

        let trails = &self.config.hud.grenade_trails;
        for entity in &data.entities {
            if let EntityInfo::Weapon(weapon) = entity
                && weapon.weapon == shared::Weapon::C4
            {
                let point = projector.project(weapon.position);
                if rect.contains(point) {
                    draw_dropped_c4(painter, point, ctx);
                }
                continue;
            }

            let (position, color, is_weapon) = match entity {
                EntityInfo::Weapon(weapon) => (
                    weapon.position,
                    weapon_class_color(weapon.weapon.weapon_class()),
                    true,
                ),
                EntityInfo::Inferno(inferno) => (inferno.position, trails.incendiary, false),
                EntityInfo::Molotov(molotov) => (
                    molotov.position,
                    if molotov.is_incendiary {
                        trails.incendiary
                    } else {
                        trails.molotov
                    },
                    false,
                ),
                EntityInfo::Smoke(grenade) => (grenade.position, trails.smoke, false),
                EntityInfo::Flashbang(grenade) => (grenade.position, trails.flash, false),
                EntityInfo::HeGrenade(grenade) => (grenade.position, trails.he, false),
                EntityInfo::Decoy(grenade) => (grenade.position, trails.decoy, false),
                EntityInfo::Chicken(_) => continue,
            };

            let point = projector.project(position);
            if !rect.contains(point) {
                continue;
            }
            if is_weapon {
                draw_diamond(painter, point, 4.0, color);
            } else {
                painter.circle_filled(point, 3.0, color);
                painter.circle_stroke(point, 3.0, Stroke::new(1.0, Colors::BACKDROP));
            }
        }

        self.draw_ghosts(painter, &projector, rect, data, ctx);

        for player in &data.players {
            draw_marker(
                painter,
                &projector,
                rect,
                player,
                Colors::RED,
                config,
                ctx,
                false,
            );
        }
        for player in &data.friendlies {
            let is_local = player.steam_id == data.local_player.steam_id;
            draw_marker(
                painter,
                &projector,
                rect,
                player,
                Colors::BLUE,
                config,
                ctx,
                is_local,
            );
        }
        if !data
            .friendlies
            .iter()
            .any(|p| p.steam_id == data.local_player.steam_id)
            && data.local_player.steam_id != 0
        {
            draw_marker(
                painter,
                &projector,
                rect,
                &data.local_player,
                Colors::BLUE,
                config,
                ctx,
                true,
            );
        }

        if data.bomb.planted {
            draw_bomb_marker(painter, &projector, ctx, &data.bomb);
        }
    }

    fn draw_ghosts(
        &self,
        painter: &Painter,
        projector: &Projector,
        bounds: Rect,
        data: &Data,
        ctx: &Context,
    ) {
        const GHOST_DURATION: f64 = 2.5;
        let now = ctx.input(|i| i.time);
        let mut present_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();

        let mut ghosts = self.minimap.ghosts.lock().unwrap();

        for (list, color) in [
            (&data.players, Colors::RED),
            (&data.friendlies, Colors::BLUE),
        ] {
            for player in list.iter() {
                if player.steam_id == 0 {
                    continue;
                }
                present_ids.insert(player.steam_id);
                ghosts.insert(
                    player.steam_id,
                    GhostEntry {
                        position: player.position,
                        color,
                        missing_since: None,
                    },
                );
            }
        }

        for (steam_id, entry) in ghosts.iter_mut() {
            if present_ids.contains(steam_id) {
                continue;
            }
            entry.missing_since.get_or_insert(now);
        }

        ghosts.retain(|_, entry| entry.missing_since.is_none_or(|t| now - t < GHOST_DURATION));

        for (steam_id, entry) in ghosts.iter() {
            if present_ids.contains(steam_id) {
                continue;
            }
            let Some(missing_since) = entry.missing_since else {
                continue;
            };
            let elapsed = now - missing_since;
            let alpha = (((1.0 - elapsed / GHOST_DURATION) * 140.0).max(0.0)) as u8;
            let point = projector.project(entry.position);
            if !bounds.contains(point) {
                continue;
            }
            let ring_color = Color32::from_rgba_unmultiplied(
                entry.color.r(),
                entry.color.g(),
                entry.color.b(),
                alpha,
            );
            painter.circle_stroke(point, 7.0, Stroke::new(1.5, ring_color));
            painter.text(
                point,
                egui::Align2::CENTER_CENTER,
                "?",
                egui::FontId::proportional(10.0),
                ring_color,
            );
        }
    }
}

fn weapon_class_color(class: WeaponClass) -> Color32 {
    match class {
        WeaponClass::Pistol => Colors::SUBTEXT,
        WeaponClass::Smg => Colors::TEAL,
        WeaponClass::Heavy => Colors::PURPLE,
        WeaponClass::Shotgun => Colors::ORANGE,
        WeaponClass::Rifle => Colors::YELLOW,
        WeaponClass::Sniper => Colors::RED,
        WeaponClass::Grenade => Colors::GREEN,
        WeaponClass::Utility => Colors::BLUE,
        WeaponClass::Knife | WeaponClass::Unknown => Colors::TEXT,
    }
}

fn draw_diamond(painter: &Painter, point: Pos2, radius: f32, color: Color32) {
    let points = vec![
        pos2(point.x, point.y - radius),
        pos2(point.x + radius, point.y),
        pos2(point.x, point.y + radius),
        pos2(point.x - radius, point.y),
    ];
    painter.add(egui::Shape::convex_polygon(
        points,
        color,
        Stroke::new(1.0, Colors::BACKDROP),
    ));
}

#[allow(clippy::too_many_arguments)]
fn draw_marker(
    painter: &Painter,
    projector: &Projector,
    bounds: Rect,
    player: &PlayerData,
    color: Color32,
    config: &crate::config::hud::MinimapConfig,
    ctx: &Context,
    is_local: bool,
) {
    let point = projector.project(player.position);
    let marker_size = config.marker_size;

    if !bounds.contains(point) {
        draw_edge_indicator(painter, bounds, point, color);
        return;
    }

    if is_local {
        let time = ctx.input(|i| i.time);
        let period = 1.1_f64;
        let pulse = (time % period) / period;
        painter.circle_stroke(
            point,
            marker_size + 6.0 + (pulse as f32) * 12.0,
            Stroke::new(
                2.0,
                Color32::from_rgba_unmultiplied(
                    Colors::YELLOW.r(),
                    Colors::YELLOW.g(),
                    Colors::YELLOW.b(),
                    ((1.0 - pulse) * 255.0) as u8,
                ),
            ),
        );
    }

    if player.has_bomb {
        painter.circle_stroke(point, marker_size + 3.0, Stroke::new(2.5, Colors::GOLD));
    }

    // composed with the view rotation so markers turn together with the map
    let heading_rad = (-player.rotation + 90.0).to_radians() + projector.rotation_rad;
    let (sin, cos) = heading_rad.sin_cos();
    let rotate = |x: f32, y: f32| pos2(point.x + x * cos - y * sin, point.y + x * sin + y * cos);

    let tip = rotate(0.0, -marker_size);
    let left = rotate(-marker_size * 0.75, marker_size * 0.6);
    let right = rotate(marker_size * 0.75, marker_size * 0.6);

    if player.visible {
        painter.add(egui::Shape::convex_polygon(
            vec![tip, right, left],
            color,
            Stroke::new(1.0, Colors::BACKDROP),
        ));
    } else {
        let dim = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 100);
        painter.add(egui::Shape::convex_polygon(
            vec![tip, right, left],
            Color32::TRANSPARENT,
            Stroke::new(1.5, dim),
        ));
    }
}

fn draw_edge_indicator(painter: &Painter, bounds: Rect, target: Pos2, color: Color32) {
    let center = bounds.center();
    let dx = target.x - center.x;
    let dy = target.y - center.y;
    if dx == 0.0 && dy == 0.0 {
        return;
    }

    let inset = 14.0;
    let half_w = bounds.width() / 2.0 - inset;
    let half_h = bounds.height() / 2.0 - inset;
    let scale = (half_w / dx.abs().max(1e-6)).min(half_h / dy.abs().max(1e-6));
    let point = pos2(center.x + dx * scale, center.y + dy * scale);

    let angle = dy.atan2(dx) + std::f32::consts::FRAC_PI_2;
    let (sin, cos) = angle.sin_cos();
    let rotate = |x: f32, y: f32| pos2(point.x + x * cos - y * sin, point.y + x * sin + y * cos);

    painter.add(egui::Shape::convex_polygon(
        vec![rotate(0.0, -7.0), rotate(6.0, 6.0), rotate(-6.0, 6.0)],
        color,
        Stroke::new(1.0, Colors::BACKDROP),
    ));
}

fn draw_c4_marker(
    painter: &Painter,
    point: Pos2,
    ctx: &Context,
    color: Color32,
    period: f64,
    label: &str,
) {
    let time = ctx.input(|i| i.time);
    let pulse = ((time % period) / period) as f32;

    painter.circle_filled(
        point,
        8.0 + pulse * 18.0,
        Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            ((1.0 - pulse) * 130.0) as u8,
        ),
    );
    painter.circle_filled(point, 8.0, color);
    painter.circle_stroke(point, 8.0, Stroke::new(2.0, Colors::BACKDROP));

    painter.text(
        point + vec2(0.0, -18.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(13.0),
        color,
    );
}

fn draw_dropped_c4(painter: &Painter, point: Pos2, ctx: &Context) {
    draw_c4_marker(painter, point, ctx, Colors::GOLD, 1.4, "C4");
}

fn draw_bomb_marker(
    painter: &Painter,
    projector: &Projector,
    ctx: &Context,
    bomb: &shared::BombData,
) {
    let point = projector.project(bomb.position);
    let defusing = bomb.being_defused;
    let color = if defusing { Colors::GREEN } else { Colors::RED };
    let period = if defusing { 0.6 } else { 1.1 };
    let label = if defusing {
        format!("defusing {:.1}s", bomb.defuse_remain_time)
    } else {
        format!("{:.1}", bomb.timer)
    };

    draw_c4_marker(painter, point, ctx, color, period, &label);
}

fn draw_rotated_map(
    painter: &Painter,
    texture_id: egui::TextureId,
    center: Pos2,
    image_size: f32,
    rotation_rad: f32,
    ref_percent: (f32, f32),
) {
    // image covers percent-space (0,0)-(100,100); offset so ref_percent lands on `center`
    let top_left_x = -(ref_percent.0 / 100.0) * image_size;
    let top_left_y = -(ref_percent.1 / 100.0) * image_size;

    let corners_local = [
        (top_left_x, top_left_y),
        (top_left_x + image_size, top_left_y),
        (top_left_x + image_size, top_left_y + image_size),
        (top_left_x, top_left_y + image_size),
    ];
    let uvs = [
        pos2(0.0, 0.0),
        pos2(1.0, 0.0),
        pos2(1.0, 1.0),
        pos2(0.0, 1.0),
    ];

    let (sin, cos) = rotation_rad.sin_cos();
    let mut mesh = Mesh::with_texture(texture_id);
    for (i, (x, y)) in corners_local.iter().enumerate() {
        let rx = x * cos - y * sin;
        let ry = x * sin + y * cos;
        mesh.vertices.push(Vertex {
            pos: center + vec2(rx, ry),
            uv: uvs[i],
            color: Color32::WHITE,
        });
    }
    mesh.indices = vec![0, 1, 2, 0, 2, 3];
    painter.add(egui::Shape::mesh(mesh));
}
