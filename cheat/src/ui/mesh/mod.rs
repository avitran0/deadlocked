pub mod asset;
pub mod capsule;
pub mod renderer;

use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use egui::Color32;
use egui_glow::glow;
use glam::Mat4;
use shared::{Bones, Data, PlayerData, Team};
use strum::IntoEnumIterator as _;

use crate::{
    config::BASE_PATH,
    config::player::{ColorValues, DrawMode, MeshPartVisibilityMode, ModelEspMode},
    math::world_to_screen,
    mesh_extract,
    ui::color::health_color,
};

// how often to recheck for the agent index while it's missing, so running
// the in-app extractor (see mesh_extract) gets picked up without restarting
const RETRY_INTERVAL: Duration = Duration::from_secs(3);

/// highlights each enemy with their own equipped agent's mesh, falling back
/// to a team-correct default (mesh_extract::FALLBACK_AGENT_T/_CT) for any
/// agent whose mesh isn't available (not yet extracted, extraction failed
/// for it, or the player is using their team's plain default skin with no
/// specific agent purchased). producing the meshes happens via the
/// "Extract Player Models" button (see mesh_extract), which reads the
/// user's own CS2 install; this feature does nothing until that has run at
/// least once.
#[derive(Default)]
pub struct PlayerMeshState {
    last_index_attempt: Option<Instant>,
    agent_index: HashMap<u16, String>,
    renderers: HashMap<String, renderer::MeshRenderer>,
    failed: HashSet<String>,
    hitbox_renderers: HashMap<Bones, renderer::MeshRenderer>,
}

impl PlayerMeshState {
    /// # Safety
    /// `gl` must be current on the calling thread.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn draw(
        &mut self,
        gl: &glow::Context,
        data: &Data,
        mode: ModelEspMode,
        color_mode: DrawMode,
        colors: &ColorValues,
        outline_color: Color32,
        part_visibility: MeshPartVisibilityMode,
    ) {
        unsafe {
            if mode == ModelEspMode::Off {
                return;
            }

            if self.agent_index.is_empty() {
                let should_retry = self
                    .last_index_attempt
                    .is_none_or(|last| last.elapsed() >= RETRY_INTERVAL);
                if !should_retry {
                    return;
                }
                self.last_index_attempt = Some(Instant::now());
                self.agent_index = mesh_extract::load_agent_index();
                if self.agent_index.is_empty() {
                    return;
                }
            }

            // data.view_matrix is read raw from CS2's memory, which stores
            // it row-major; glam::Mat4 assumes column-major, so a plain GLSL
            // `matrix * vector` multiply needs the transpose first (this is
            // the same reason math::world_to_screen() does its own manual
            // per-axis dot product instead of a normal matrix multiply)
            let view_projection = data.view_matrix.transpose();

            for player in &data.players {
                if player.skeleton.is_empty() || !on_screen(player, data) {
                    continue;
                }

                let fallback = match player.team {
                    Team::T => mesh_extract::FALLBACK_AGENT_T,
                    _ => mesh_extract::FALLBACK_AGENT_CT,
                };
                let stem = self
                    .agent_index
                    .get(&player.agent_def_index)
                    .cloned()
                    .unwrap_or_else(|| fallback.to_string());

                let Some(renderer) = self.renderer_for(gl, &stem, fallback) else {
                    continue;
                };

                let skin_matrices = renderer.skin_matrices(&player.skeleton);
                let health = health_color(player.health, player.max_health, 255);
                let color = colors.resolve(color_mode, player, data, health);
                let [r, g, b, _] = color.to_normalized_gamma_f32();
                // shade the mesh darker while the player isn't actually
                // visible (behind a wall, etc.), same visual cue the
                // minimap and box ESP already use for hidden targets
                let shade = if player.visible { 1.0 } else { 0.4 };
                // the resolved color's own alpha isn't used here: alpha
                // comes from the render mode instead, so Highlight stays a
                // translucent see-through tint while Wireframe/Solid are
                // fully opaque regardless of which color mode is active
                let alpha = match mode {
                    ModelEspMode::Highlight => 0.5,
                    _ => 1.0,
                };
                let color = [r * shade, g * shade, b * shade, alpha];
                let outline = outline_color.to_normalized_gamma_f32();
                renderer.draw(
                    gl,
                    &skin_matrices,
                    view_projection,
                    color,
                    outline,
                    mode,
                    &player.bone_visibility,
                    part_visibility,
                );
            }
        }
    }

    /// draws every real hitbox capsule as an actual solid capsule mesh (the
    /// exact geometry CS2's own server checks bullets against, extracted
    /// from the compiled model), not just its centerline - for
    /// verifying/trusting what the aimbot and triggerbot are actually aiming
    /// at
    /// # Safety
    /// `gl` must be current on the calling thread.
    pub unsafe fn draw_hitboxes(
        &mut self,
        gl: &glow::Context,
        data: &Data,
        mode: ModelEspMode,
        color_mode: DrawMode,
        colors: &ColorValues,
    ) {
        unsafe {
            if mode == ModelEspMode::Off {
                return;
            }

            let view_projection = data.view_matrix.transpose();

            for player in &data.players {
                if player.bone_transforms.is_empty() || !on_screen(player, data) {
                    continue;
                }

                let health = health_color(player.health, player.max_health, 255);
                let color = colors.resolve(color_mode, player, data, health);
                let [r, g, b, _] = color.to_normalized_gamma_f32();
                let shade = if player.visible { 1.0 } else { 0.4 };
                let alpha = match mode {
                    ModelEspMode::Highlight => 0.35,
                    _ => 0.45,
                };
                let rgba = [r * shade, g * shade, b * shade, alpha];

                for bone in Bones::iter() {
                    let Some(&transform) = player.bone_transforms.get(&bone) else {
                        continue;
                    };
                    let renderer = self.hitbox_renderer_for(gl, bone);
                    let world =
                        Mat4::from_rotation_translation(transform.rotation, transform.position);
                    renderer.draw(
                        gl,
                        &[world],
                        view_projection,
                        rgba,
                        [0.0, 0.0, 0.0, 0.0],
                        mode,
                        &[],
                        MeshPartVisibilityMode::Off,
                    );
                }
            }
        }
    }

    /// lazily builds and caches the capsule mesh renderer for one `Bones`
    /// variant (only 19 unique shapes total, shared across every player)
    unsafe fn hitbox_renderer_for(
        &mut self,
        gl: &glow::Context,
        bone: Bones,
    ) -> &renderer::MeshRenderer {
        unsafe {
            self.hitbox_renderers.entry(bone).or_insert_with(|| {
                let hitbox = bone.hitbox();
                let asset =
                    capsule::build_capsule_mesh(hitbox.point0, hitbox.point1, hitbox.radius);
                renderer::MeshRenderer::new(gl, &asset)
                    .expect("procedurally-built capsule mesh is well-formed, gpu upload can't fail")
            })
        }
    }

    /// lazily loads and caches the mesh for one agent stem; falls back to
    /// the given team-correct fallback stem if this specific stem failed to
    /// load, and gives up entirely (returns None) if even the fallback
    /// isn't available
    unsafe fn renderer_for(
        &mut self,
        gl: &glow::Context,
        stem: &str,
        fallback: &str,
    ) -> Option<&renderer::MeshRenderer> {
        unsafe {
            if !self.renderers.contains_key(stem) && !self.failed.contains(stem) {
                match Self::try_load(gl, stem) {
                    Some(renderer) => {
                        self.renderers.insert(stem.to_string(), renderer);
                    }
                    None => {
                        self.failed.insert(stem.to_string());
                    }
                }
            }

            let key = if self.renderers.contains_key(stem) {
                stem
            } else {
                fallback
            };

            if !self.renderers.contains_key(key) && !self.failed.contains(key) {
                match Self::try_load(gl, key) {
                    Some(renderer) => {
                        self.renderers.insert(key.to_string(), renderer);
                    }
                    None => {
                        self.failed.insert(key.to_string());
                    }
                }
            }

            self.renderers.get(key)
        }
    }

    unsafe fn try_load(gl: &glow::Context, stem: &str) -> Option<renderer::MeshRenderer> {
        let path = BASE_PATH.join(format!("player_model_{stem}.dlms"));
        let bytes = std::fs::read(&path).ok()?;

        let asset = match asset::MeshAsset::load(&bytes) {
            Ok(asset) => asset,
            Err(err) => {
                utils::error!("failed to parse {}: {err}", path.display());
                return None;
            }
        };

        match unsafe { renderer::MeshRenderer::new(gl, &asset) } {
            Ok(renderer) => Some(renderer),
            Err(err) => {
                utils::error!("failed to upload player mesh {stem} to gpu: {err}");
                None
            }
        }
    }
}

/// cheap culling: skips uploading skin matrices and issuing a draw call for
/// a player whose position and head are both off the visible window,
/// checking both since a standing player's feet vs head can straddle the
/// screen edge
fn on_screen(player: &PlayerData, data: &Data) -> bool {
    world_to_screen(&player.position, data).is_some()
        || world_to_screen(&player.head, data).is_some()
}
