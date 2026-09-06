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
use shared::{Bones, ChickenBones, ChickenInfo, Data, EntityInfo, PlayerData, Team};
use strum::IntoEnumIterator as _;

use crate::{
    config::{
        BASE_PATH,
        player::{ColorValues, DrawMode, ModelEspMode},
    },
    math::world_to_screen,
    mesh_extract,
    ui::color::health_color,
};

// how often to recheck for the agent index while it's missing
const RETRY_INTERVAL: Duration = Duration::from_secs(3);

#[derive(Default)]
pub struct PlayerMeshState {
    last_index_attempt: Option<Instant>,
    agent_index: HashMap<u16, String>,
    renderers: HashMap<String, renderer::MeshRenderer>,
    failed: HashSet<String>,
    hitbox_renderers: HashMap<Bones, renderer::MeshRenderer>,
    chicken_renderer: Option<renderer::MeshRenderer>,
    chicken_failed: bool,
    last_chicken_attempt: Option<Instant>,
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
        show_friendlies: bool,
        visible_only: bool,
        sound_alphas: &HashMap<(u64, String), f32>,
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

            // view_matrix is row-major, glam expects column-major
            let view_projection = data.view_matrix.transpose();

            let friendlies = show_friendlies
                .then(|| data.friendlies.iter())
                .into_iter()
                .flatten();
            for player in data.players.iter().chain(friendlies) {
                if visible_only && !player.visible {
                    continue;
                }
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
                // only the c4 highlight dims for not being visible
                let is_c4_highlight = colors.c4_carrier_highlight && player.has_bomb;
                let shade = if !is_c4_highlight || player.visible {
                    1.0
                } else {
                    0.4
                };
                // alpha comes from the render mode, not the color's own alpha
                let alpha = match mode {
                    ModelEspMode::Highlight => 0.5,
                    _ => 1.0,
                };
                let key = (player.steam_id, player.name.clone());
                let sound_alpha = sound_alphas.get(&key).copied().unwrap_or(1.0);
                let color = [r * shade, g * shade, b * shade, alpha * sound_alpha];
                let outline = outline_color.to_normalized_gamma_f32();
                let on_screen = bone_on_screen(player, data);
                renderer.draw(
                    gl,
                    &skin_matrices,
                    view_projection,
                    color,
                    outline,
                    mode,
                    &player.bone_visibility,
                    &on_screen,
                );
            }
        }
    }

    /// # Safety
    /// `gl` must be current on the calling thread.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn draw_hitboxes(
        &mut self,
        gl: &glow::Context,
        data: &Data,
        mode: ModelEspMode,
        color_mode: DrawMode,
        colors: &ColorValues,
        show_friendlies: bool,
        visible_only: bool,
        sound_alphas: &HashMap<(u64, String), f32>,
    ) {
        unsafe {
            if mode == ModelEspMode::Off {
                return;
            }

            let view_projection = data.view_matrix.transpose();

            let friendlies = show_friendlies
                .then(|| data.friendlies.iter())
                .into_iter()
                .flatten();
            for player in data.players.iter().chain(friendlies) {
                if visible_only && !player.visible {
                    continue;
                }
                if player.bone_transforms.is_empty() || !on_screen(player, data) {
                    continue;
                }

                let health = health_color(player.health, player.max_health, 255);
                let color = colors.resolve(color_mode, player, data, health);
                let [r, g, b, _] = color.to_normalized_gamma_f32();
                // only the c4 highlight dims for not being visible
                let is_c4_highlight = colors.c4_carrier_highlight && player.has_bomb;
                let shade = if !is_c4_highlight || player.visible {
                    1.0
                } else {
                    0.4
                };
                let alpha = match mode {
                    ModelEspMode::Highlight => 0.35,
                    ModelEspMode::Wireframe => 0.45,
                    _ => 1.0,
                };
                let key = (player.steam_id, player.name.clone());
                let sound_alpha = sound_alphas.get(&key).copied().unwrap_or(1.0);
                let rgba = [r * shade, g * shade, b * shade, alpha * sound_alpha];

                for bone in Bones::iter() {
                    let Some(&transform) = player.bone_transforms.get(&bone) else {
                        continue;
                    };
                    let renderer = self.hitbox_renderer_for(gl, bone);
                    let world =
                        Mat4::from_rotation_translation(transform.rotation, transform.position);
                    let center = bone.hitbox().world_center(transform);
                    let visible = world_to_screen(&center, data).is_some();
                    let on_screen = [if visible { 1.0 } else { 0.0 }];
                    renderer.draw(
                        gl,
                        &[world],
                        view_projection,
                        rgba,
                        [0.0, 0.0, 0.0, 0.0],
                        mode,
                        &[],
                        &on_screen,
                    );
                }
            }
        }
    }

    /// # Safety
    /// `gl` must be current on the calling thread.
    pub unsafe fn draw_chickens(
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

            let Some(renderer) = self.chicken_renderer_for(gl) else {
                return;
            };

            let view_projection = data.view_matrix.transpose();

            for entity in &data.entities {
                let EntityInfo::Chicken(chicken) = entity else {
                    continue;
                };
                if chicken.skeleton.is_empty() || world_to_screen(&chicken.position, data).is_none()
                {
                    continue;
                }

                let skin_matrices = renderer.skin_matrices(&chicken.skeleton);
                // chickens are neutral, so Color mode just uses the enemy color
                let color = match color_mode {
                    DrawMode::Health => health_color(100, 100, 255),
                    _ => colors.enemy_color,
                };
                let [r, g, b, _] = color.to_normalized_gamma_f32();
                let shade = if chicken.visible { 1.0 } else { 0.4 };
                let alpha = match mode {
                    ModelEspMode::Highlight => 0.5,
                    _ => 1.0,
                };
                let color = [r * shade, g * shade, b * shade, alpha];
                let on_screen = chicken_bone_on_screen(chicken, data);
                renderer.draw(
                    gl,
                    &skin_matrices,
                    view_projection,
                    color,
                    [0.0, 0.0, 0.0, 0.0],
                    mode,
                    &[],
                    &on_screen,
                );
            }
        }
    }

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
                // single-joint mesh, visibility_source is trivially [0] regardless
                renderer::MeshRenderer::new(gl, &asset, &HashSet::new())
                    .expect("procedurally-built capsule mesh is well-formed, gpu upload can't fail")
            })
        }
    }

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

        let named: HashSet<usize> = Bones::iter().map(|bone| bone.u64() as usize).collect();
        match unsafe { renderer::MeshRenderer::new(gl, &asset, &named) } {
            Ok(renderer) => Some(renderer),
            Err(err) => {
                utils::error!("failed to upload player mesh {stem} to gpu: {err}");
                None
            }
        }
    }

    unsafe fn chicken_renderer_for(
        &mut self,
        gl: &glow::Context,
    ) -> Option<&renderer::MeshRenderer> {
        unsafe {
            if self.chicken_renderer.is_none() && !self.chicken_failed {
                let should_retry = self
                    .last_chicken_attempt
                    .is_none_or(|last| last.elapsed() >= RETRY_INTERVAL);
                if !should_retry {
                    return None;
                }
                self.last_chicken_attempt = Some(Instant::now());

                // don't latch failure for a missing file, only a real load error
                if !BASE_PATH.join(mesh_extract::CHICKEN_MODEL_FILE).exists() {
                    return None;
                }
                self.chicken_renderer = Self::try_load_chicken(gl);
                self.chicken_failed = self.chicken_renderer.is_none();
            }
            self.chicken_renderer.as_ref()
        }
    }

    unsafe fn try_load_chicken(gl: &glow::Context) -> Option<renderer::MeshRenderer> {
        let path = BASE_PATH.join(mesh_extract::CHICKEN_MODEL_FILE);
        let bytes = std::fs::read(&path).ok()?;

        let asset = match asset::MeshAsset::load(&bytes) {
            Ok(asset) => asset,
            Err(err) => {
                utils::error!("failed to parse {}: {err}", path.display());
                return None;
            }
        };

        let named: HashSet<usize> = ChickenBones::iter().map(ChickenBones::usize).collect();
        match unsafe { renderer::MeshRenderer::new(gl, &asset, &named) } {
            Ok(renderer) => Some(renderer),
            Err(err) => {
                utils::error!("failed to upload chicken mesh to gpu: {err}");
                None
            }
        }
    }
}

/// culls a player whose position and head are both off-screen
fn on_screen(player: &PlayerData, data: &Data) -> bool {
    world_to_screen(&player.position, data).is_some()
        || world_to_screen(&player.head, data).is_some()
        || player
            .bone_transforms
            .values()
            .any(|transform| world_to_screen(&transform.position, data).is_some())
}

/// same indexing as `player.bone_visibility`, drives the edge-of-screen cull
fn bone_on_screen(player: &PlayerData, data: &Data) -> Vec<f32> {
    let mut on_screen = vec![1.0; player.bone_visibility.len()];
    for bone in Bones::iter() {
        let index = bone.u64() as usize;
        let Some(slot) = on_screen.get_mut(index) else {
            continue;
        };
        let Some(&transform) = player.bone_transforms.get(&bone) else {
            continue;
        };
        let center = bone.hitbox().world_center(transform);
        *slot = if world_to_screen(&center, data).is_some() {
            1.0
        } else {
            0.0
        };
    }
    on_screen
}

/// chickens have no hitbox capsule table, so this uses raw joint position
fn chicken_bone_on_screen(chicken: &ChickenInfo, data: &Data) -> Vec<f32> {
    let mut on_screen = vec![1.0; chicken.skeleton.len()];
    for bone in ChickenBones::iter() {
        let index = bone.usize();
        let Some(slot) = on_screen.get_mut(index) else {
            continue;
        };
        let Some(transform) = chicken.skeleton.get(index) else {
            continue;
        };
        *slot = if world_to_screen(&transform.position, data).is_some() {
            1.0
        } else {
            0.0
        };
    }
    on_screen
}
