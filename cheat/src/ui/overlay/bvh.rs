use std::sync::{Arc, OnceLock};

use bytemuck::cast_slice;
use egui::{PaintCallback, Painter, Rect, pos2};
use egui_glow::{
    CallbackFn,
    glow::{self, HasContext as _},
};
use glam::Vec3;
use utils::Mutex;

use shared::Data;

use crate::{
    parser::{bvh::Bvh, load_map},
    ui::{app::AppState, overlay::opengl},
};

const VERTEX_STRIDE_BYTES: i32 = 24;
const WIREFRAME_LINE_WIDTH: f32 = 2.0;
const BVH_RETRY_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

pub struct BvhRenderParams<'a> {
    pub viewport: (i32, i32, i32, i32),
    pub view: &'a [f32; 16],
    pub view_pos: Vec3,
    pub depth_threshold: f32,
    pub visible_only: bool,
}

pub struct BvhRenderer {
    pub map: String,
    glow: Arc<glow::Context>,
    program: Arc<BvhProgram>,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    surface_vertex_count: i32,
    line_vertex_count: i32,
    origin: Vec3,
    range: f32,
}

struct BvhProgram {
    program: glow::Program,
    uniforms: BvhUniforms,
}

struct BvhUniforms {
    view: glow::UniformLocation,
    view_pos: glow::UniformLocation,
    depth_threshold: glow::UniformLocation,
}

static BVH_PROGRAM: OnceLock<Arc<BvhProgram>> = OnceLock::new();
pub static BVH_RENDERER: OnceLock<Mutex<Option<BvhRenderer>>> = OnceLock::new();

fn get_or_build_program(glow: &glow::Context) -> Result<Arc<BvhProgram>, String> {
    if let Some(program) = BVH_PROGRAM.get() {
        return Ok(program.clone());
    }

    let program = build_program(glow)?;
    let uniforms = BvhUniforms {
        view: opengl::uniform_location(glow, program, "u_view")?,
        view_pos: opengl::uniform_location(glow, program, "u_view_pos")?,
        depth_threshold: opengl::uniform_location(glow, program, "u_depth_threshold")?,
    };
    let program = Arc::new(BvhProgram { program, uniforms });
    Ok(BVH_PROGRAM.get_or_init(|| program).clone())
}

impl BvhRenderer {
    pub fn new(
        glow: Arc<glow::Context>,
        map: String,
        bvh: &Bvh,
        origin: Vec3,
        range: f32,
    ) -> Result<Self, String> {
        let (vertices, surface_vertex_count, line_vertex_count) =
            build_vertices(bvh, origin, range);
        let program = get_or_build_program(glow.as_ref())?;

        let vao = unsafe { glow.create_vertex_array() }.map_err(|error| error.to_string())?;
        let vbo = unsafe { glow.create_buffer() }.map_err(|error| error.to_string())?;
        unsafe {
            glow.bind_vertex_array(Some(vao));
            glow.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            glow.enable_vertex_attrib_array(0);
            glow.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, VERTEX_STRIDE_BYTES, 0);
            glow.enable_vertex_attrib_array(1);
            glow.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, VERTEX_STRIDE_BYTES, 12);
            glow.buffer_data_u8_slice(glow::ARRAY_BUFFER, cast_slice(&vertices), glow::STATIC_DRAW);
            glow.bind_buffer(glow::ARRAY_BUFFER, None);
            glow.bind_vertex_array(None);
        }

        Ok(Self {
            map,
            glow,
            program,
            vao,
            vbo,
            surface_vertex_count,
            line_vertex_count,
            origin,
            range,
        })
    }

    pub fn map_matches(&self, map: &str, origin: Vec3, range: f32) -> bool {
        self.map == map
            && self.range == range
            && self.origin.distance_squared(origin) <= (range * 0.25).powi(2)
    }

    pub fn render(&self, glow: &glow::Context, params: BvhRenderParams<'_>) {
        if self.line_vertex_count == 0 {
            return;
        }

        unsafe {
            let BvhRenderParams {
                viewport,
                view,
                view_pos,
                depth_threshold,
                visible_only,
            } = params;

            let (left, bottom, width, height) = viewport;
            glow.viewport(left, bottom, width.max(1), height.max(1));
            glow.use_program(Some(self.program.program));
            glow.uniform_matrix_4_f32_slice(Some(&self.program.uniforms.view), true, view);
            glow.uniform_3_f32(
                Some(&self.program.uniforms.view_pos),
                view_pos.x,
                view_pos.y,
                view_pos.z,
            );
            glow.uniform_1_f32(
                Some(&self.program.uniforms.depth_threshold),
                depth_threshold,
            );

            glow.bind_vertex_array(Some(self.vao));
            glow.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            glow.disable(glow::CULL_FACE);

            if visible_only {
                glow.clear(glow::DEPTH_BUFFER_BIT);
                glow.enable(glow::DEPTH_TEST);
                glow.depth_func(glow::LEQUAL);
                glow.depth_mask(true);
                glow.color_mask(false, false, false, false);
                glow.draw_arrays(glow::TRIANGLES, 0, self.surface_vertex_count);
                glow.color_mask(true, true, true, true);
                glow.depth_mask(false);
            } else {
                glow.disable(glow::DEPTH_TEST);
                glow.depth_mask(false);
            }

            glow.line_width(WIREFRAME_LINE_WIDTH);
            glow.draw_arrays(
                glow::LINES,
                self.surface_vertex_count,
                self.line_vertex_count,
            );

            glow.line_width(1.0);
            glow.color_mask(true, true, true, true);
            glow.depth_mask(true);
            glow.disable(glow::DEPTH_TEST);
            glow.bind_buffer(glow::ARRAY_BUFFER, None);
            glow.bind_vertex_array(None);
            glow.use_program(None);
        }
    }
}

impl Drop for BvhRenderer {
    fn drop(&mut self) {
        unsafe {
            self.glow.delete_buffer(self.vbo);
            self.glow.delete_vertex_array(self.vao);
        }
    }
}

impl AppState {
    pub fn handle_bvh(&mut self, data: &Data) {
        if !self.config.hud.bvh_debug.enabled
            || !data.in_game
            || !data.bvh_available
            || data.map_name.is_empty()
        {
            self.bvh = None;
            self.bvh_map.clear();
            self.bvh_build_date.clear();
            self.last_bvh_load = None;
            return;
        }

        if self.bvh_map != data.map_name || self.bvh_build_date != data.map_build_date {
            self.bvh = None;
            self.bvh_map.clone_from(&data.map_name);
            self.bvh_build_date.clone_from(&data.map_build_date);
            self.last_bvh_load = None;
        }

        if self.bvh.is_some()
            || self
                .last_bvh_load
                .is_some_and(|last| last.elapsed() < BVH_RETRY_INTERVAL)
        {
            return;
        }

        self.last_bvh_load = Some(std::time::Instant::now());
        self.bvh = load_map(&self.bvh_map, &self.bvh_build_date);
        if self.bvh.is_none() {
            utils::warn!(
                "failed to load BVH cache for visualizer on {}",
                self.bvh_map
            );
        }
    }

    pub fn draw_bvh_visualizer(&self, painter: &Painter, data: &Data, glow: &Arc<glow::Context>) {
        if !self.config.hud.bvh_debug.enabled || !data.in_game {
            return;
        }
        let Some(bvh) = &self.bvh else {
            return;
        };
        if bvh.all_triangles().is_empty() {
            return;
        }

        let map = data.map_name.clone();
        let view = data.view_matrix.to_cols_array();
        let origin = data.local_player.head;
        let range = self.config.hud.bvh_debug.range;
        let visible_only = self.config.hud.bvh_debug.visible_only;
        let renderer_cache = BVH_RENDERER.get_or_init(|| Mutex::new(None));
        {
            let mut renderer = renderer_cache.lock();
            let needs_rebuild = renderer
                .as_ref()
                .is_none_or(|renderer| !renderer.map_matches(&map, origin, range));
            if needs_rebuild {
                match BvhRenderer::new(glow.clone(), map, bvh, origin, range) {
                    Ok(new_renderer) => *renderer = Some(new_renderer),
                    Err(error) => {
                        utils::error!("failed to create BVH visualizer: {error}");
                        *renderer = None;
                        return;
                    }
                }
            }
        }

        let callback = CallbackFn::new(move |info, painter| {
            let renderer_guard = renderer_cache.lock();
            let Some(renderer) = renderer_guard.as_ref() else {
                return;
            };
            let viewport = info.viewport_in_pixels();
            renderer.render(
                painter.gl(),
                BvhRenderParams {
                    viewport: (
                        viewport.left_px,
                        viewport.from_bottom_px,
                        viewport.width_px,
                        viewport.height_px,
                    ),
                    view: &view,
                    view_pos: origin,
                    depth_threshold: range,
                    visible_only,
                },
            );
        });
        painter.add(PaintCallback {
            rect: Rect::from_min_size(
                pos2(0.0, 0.0),
                egui::vec2(data.window_size.x, data.window_size.y),
            ),
            callback: Arc::new(callback),
        });
    }
}

fn build_vertices(bvh: &Bvh, origin: Vec3, range: f32) -> (Vec<f32>, i32, i32) {
    let mut surface_vertices = Vec::new();
    let mut line_vertices = Vec::new();
    let max_distance = range + 128.0;
    let max_penetration = bvh
        .surface_materials()
        .iter()
        .map(|material| material.penetration_modifier)
        .filter(|value| value.is_finite() && *value > 0.0)
        .fold(0.0_f32, f32::max);
    let max_damage = bvh
        .surface_materials()
        .iter()
        .map(|material| material.damage_modifier)
        .filter(|value| value.is_finite() && *value > 0.0)
        .fold(0.0_f32, f32::max);

    for triangle in bvh.all_triangles() {
        if triangle
            .v0
            .distance(origin)
            .min(triangle.v1.distance(origin))
            .min(triangle.v2.distance(origin))
            > max_distance
        {
            continue;
        }
        let score = bvh
            .material(triangle)
            .map(|material| {
                material_score(
                    material.penetration_modifier,
                    material.damage_modifier,
                    max_penetration,
                    max_damage,
                )
            })
            .unwrap_or(0.0);
        let color = penetrability_color(score);
        push_filled_triangle(
            &mut surface_vertices,
            triangle.v0,
            triangle.v1,
            triangle.v2,
            color,
        );
        push_triangle(
            &mut line_vertices,
            triangle.v0,
            triangle.v1,
            triangle.v2,
            color,
        );
    }

    let surface_vertex_count = (surface_vertices.len() / 6) as i32;
    let line_vertex_count = (line_vertices.len() / 6) as i32;
    surface_vertices.extend(line_vertices);
    (surface_vertices, surface_vertex_count, line_vertex_count)
}

fn material_score(
    penetration_modifier: f32,
    damage_modifier: f32,
    max_penetration: f32,
    max_damage: f32,
) -> f32 {
    if !penetration_modifier.is_finite()
        || !damage_modifier.is_finite()
        || penetration_modifier <= 0.0
        || damage_modifier <= 0.0
        || max_penetration <= 0.0
        || max_damage <= 0.0
    {
        return 0.0;
    }

    let penetration = (penetration_modifier / max_penetration).clamp(0.0, 1.0);
    let damage = (damage_modifier / max_damage).clamp(0.0, 1.0);
    (penetration * damage).sqrt()
}

fn penetrability_color(score: f32) -> [f32; 3] {
    let score = score.clamp(0.0, 1.0);
    if score <= 0.5 {
        [1.0, score * 2.0, 0.0]
    } else {
        [(1.0 - score) * 2.0, 1.0, 0.0]
    }
}

fn build_program(glow: &glow::Context) -> Result<glow::Program, String> {
    opengl::build_program(
        glow,
        include_str!("shaders/bvh.vert"),
        include_str!("shaders/bvh.frag"),
    )
}

fn push_filled_triangle(vertices: &mut Vec<f32>, a: Vec3, b: Vec3, c: Vec3, color: [f32; 3]) {
    push_vertex(vertices, a, color);
    push_vertex(vertices, b, color);
    push_vertex(vertices, c, color);
}

fn push_triangle(vertices: &mut Vec<f32>, a: Vec3, b: Vec3, c: Vec3, color: [f32; 3]) {
    push_vertex(vertices, a, color);
    push_vertex(vertices, b, color);
    push_vertex(vertices, b, color);
    push_vertex(vertices, c, color);
    push_vertex(vertices, c, color);
    push_vertex(vertices, a, color);
}

fn push_vertex(vertices: &mut Vec<f32>, position: Vec3, color: [f32; 3]) {
    vertices.extend_from_slice(&[
        position.x, position.y, position.z, color[0], color[1], color[2],
    ]);
}
