use std::{collections::HashMap, sync::Arc};

use bytemuck::cast_slice;
use egui_glow::glow::{self, HasContext as _};
use glam::{Mat4, Vec3};

use shared::BoneTransform;

use crate::ui::overlay::{models::MODELS, opengl};

#[derive(Clone, Copy)]
pub enum ModelRenderMode {
    Filled,
    Wireframe,
}

pub struct ModelRenderParams<'a> {
    pub model_name: &'a str,
    pub skeleton: &'a [BoneTransform],
    pub viewport: (i32, i32, i32, i32),
    pub view: &'a [f32; 16],
    pub model: &'a [f32; 16],
    pub visible_color: [f32; 4],
    pub invisible_color: [f32; 4],
    pub mode: ModelRenderMode,
}

struct PrimitiveMesh {
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    joint_vbo: glow::Buffer,
    weight_vbo: glow::Buffer,
    ebo: glow::Buffer,
    index_count: i32,
    joint_names: Vec<String>,
    inverse_bind_matrices: Vec<Mat4>,
}

struct ModelMesh {
    primitives: Vec<PrimitiveMesh>,
}

pub struct ModelRenderer {
    glow: Arc<glow::Context>,
    program: glow::Program,
    meshes: HashMap<String, ModelMesh>,
    uniforms: ModelUniforms,
}

struct ModelUniforms {
    view: glow::UniformLocation,
    model: glow::UniformLocation,
    visible_color: glow::UniformLocation,
    invisible_color: glow::UniformLocation,
    bone_visibility: glow::UniformLocation,
    bones: glow::UniformLocation,
}

impl ModelRenderer {
    pub fn new(glow: Arc<glow::Context>) -> Result<Self, String> {
        let program = build_program(glow.as_ref())?;
        let mut meshes = HashMap::with_capacity(MODELS.len());

        for info in MODELS {
            let model = load_model(glow.as_ref(), info.data)?;
            if meshes.insert(info.name.to_owned(), model).is_some() {
                return Err(format!("duplicate model name: {}", info.name));
            }
        }

        if meshes.is_empty() {
            return Err("model registry is empty".to_string());
        }
        let uniforms = ModelUniforms {
            view: opengl::uniform_location(glow.as_ref(), program, "u_view")?,
            model: opengl::uniform_location(glow.as_ref(), program, "u_model")?,
            visible_color: opengl::uniform_location(glow.as_ref(), program, "u_visible_color")?,
            invisible_color: opengl::uniform_location(glow.as_ref(), program, "u_invisible_color")?,
            bone_visibility: opengl::uniform_location(
                glow.as_ref(),
                program,
                "u_bone_visibility[0]",
            )?,
            bones: opengl::uniform_location(glow.as_ref(), program, "u_bones[0]")?,
        };

        Ok(Self {
            glow,
            program,
            meshes,
            uniforms,
        })
    }

    pub fn render(&self, glow: &glow::Context, params: ModelRenderParams<'_>) {
        let model_key = params
            .model_name
            .rsplit('/')
            .next()
            .unwrap_or(params.model_name)
            .trim_end_matches(".vmdl_c")
            .trim_end_matches(".vmdl");
        let Some(model) = self
            .meshes
            .get(params.model_name)
            .or_else(|| self.meshes.get(model_key))
        else {
            return;
        };

        unsafe {
            let (left, bottom, width, height) = params.viewport;
            glow.viewport(left, bottom, width.max(1), height.max(1));
            glow.use_program(Some(self.program));
            // game view matrix is row-major
            glow.uniform_matrix_4_f32_slice(Some(&self.uniforms.view), true, params.view);
            glow.uniform_matrix_4_f32_slice(Some(&self.uniforms.model), false, params.model);
            glow.uniform_4_f32(
                Some(&self.uniforms.visible_color),
                params.visible_color[0],
                params.visible_color[1],
                params.visible_color[2],
                params.visible_color[3],
            );
            glow.uniform_4_f32(
                Some(&self.uniforms.invisible_color),
                params.invisible_color[0],
                params.invisible_color[1],
                params.invisible_color[2],
                params.invisible_color[3],
            );
            let visibility: Vec<f32> = params.skeleton.iter().map(|bone| bone.visibility).collect();
            glow.uniform_1_f32_slice(Some(&self.uniforms.bone_visibility), &visibility);
            glow.enable(glow::BLEND);
            glow.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            glow.disable(glow::CULL_FACE);
            glow.polygon_mode(
                glow::FRONT_AND_BACK,
                match params.mode {
                    ModelRenderMode::Filled => glow::FILL,
                    ModelRenderMode::Wireframe => glow::LINE,
                },
            );

            for primitive in &model.primitives {
                let palette = skin_palette(primitive, params.skeleton);
                glow.uniform_matrix_4_f32_slice(Some(&self.uniforms.bones), false, &palette);
                glow.bind_vertex_array(Some(primitive.vao));
                glow.draw_elements(
                    glow::TRIANGLES,
                    primitive.index_count,
                    glow::UNSIGNED_INT,
                    0,
                );
            }

            glow.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
            glow.disable(glow::BLEND);
            glow.bind_vertex_array(None);
            glow.use_program(None);
        }
    }
}

impl Drop for ModelRenderer {
    fn drop(&mut self) {
        unsafe {
            for model in self.meshes.values() {
                for primitive in &model.primitives {
                    self.glow.delete_buffer(primitive.vbo);
                    self.glow.delete_buffer(primitive.joint_vbo);
                    self.glow.delete_buffer(primitive.weight_vbo);
                    self.glow.delete_buffer(primitive.ebo);
                    self.glow.delete_vertex_array(primitive.vao);
                }
            }
            self.glow.delete_program(self.program);
        }
    }
}

fn load_model(glow: &glow::Context, data: &[u8]) -> Result<ModelMesh, String> {
    let (document, buffers, _) = gltf::import_slice(data).map_err(|e| e.to_string())?;
    let mut mesh_skins = HashMap::new();

    for node in document.nodes() {
        let (Some(mesh), Some(skin)) = (node.mesh(), node.skin()) else {
            continue;
        };

        let reader = skin.reader(|buffer| buffers.get(buffer.index()).map(|b| b.0.as_slice()));
        let inverse_bind_matrices: Vec<Mat4> = reader
            .read_inverse_bind_matrices()
            .ok_or_else(|| format!("skin {} has no inverse-bind matrices", skin.index()))?
            .map(|matrix| Mat4::from_cols_array_2d(&matrix))
            .collect();
        let joint_names = skin
            .joints()
            .map(|joint| joint.name().unwrap_or_default().to_owned())
            .collect::<Vec<_>>();

        if inverse_bind_matrices.len() != joint_names.len() {
            return Err(format!(
                "skin {} has {} joints but {} inverse-bind matrices",
                skin.index(),
                joint_names.len(),
                inverse_bind_matrices.len()
            ));
        }

        mesh_skins.insert(mesh.index(), (joint_names, inverse_bind_matrices));
    }

    let mut primitives = Vec::new();
    for mesh in document.meshes() {
        let Some((joint_names, inverse_bind_matrices)) = mesh_skins.get(&mesh.index()) else {
            return Err(format!("mesh {} has no skin", mesh.index()));
        };

        for primitive in mesh.primitives() {
            let reader =
                primitive.reader(|buffer| buffers.get(buffer.index()).map(|b| b.0.as_slice()));
            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .ok_or_else(|| "model primitive has no positions".to_string())?
                .collect();
            let joints: Vec<[u16; 4]> = reader
                .read_joints(0)
                .ok_or_else(|| "skinned primitive has no JOINTS_0".to_string())?
                .into_u16()
                .collect();
            let weights: Vec<[f32; 4]> = reader
                .read_weights(0)
                .ok_or_else(|| "skinned primitive has no WEIGHTS_0".to_string())?
                .into_f32()
                .collect();
            let indices: Vec<u32> = reader
                .read_indices()
                .map(|indices| indices.into_u32().collect())
                .unwrap_or_else(|| (0..positions.len() as u32).collect());

            if positions.is_empty() || indices.is_empty() {
                continue;
            }
            if positions.len() != joints.len() || positions.len() != weights.len() {
                return Err("position, joint, and weight counts do not match".to_string());
            }
            if joints
                .iter()
                .flatten()
                .any(|joint| usize::from(*joint) >= joint_names.len())
            {
                return Err("primitive references a joint outside its skin".to_string());
            }

            let vao = unsafe { glow.create_vertex_array() }.map_err(|e| e.to_string())?;
            let vbo = unsafe { glow.create_buffer() }.map_err(|e| e.to_string())?;
            let joint_vbo = unsafe { glow.create_buffer() }.map_err(|e| e.to_string())?;
            let weight_vbo = unsafe { glow.create_buffer() }.map_err(|e| e.to_string())?;
            let ebo = unsafe { glow.create_buffer() }.map_err(|e| e.to_string())?;

            unsafe {
                glow.bind_vertex_array(Some(vao));

                glow.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                glow.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    cast_slice(&positions),
                    glow::STATIC_DRAW,
                );
                glow.enable_vertex_attrib_array(0);
                glow.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 12, 0);

                glow.bind_buffer(glow::ARRAY_BUFFER, Some(joint_vbo));
                glow.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    cast_slice(&joints),
                    glow::STATIC_DRAW,
                );
                glow.enable_vertex_attrib_array(1);
                glow.vertex_attrib_pointer_i32(1, 4, glow::UNSIGNED_SHORT, 8, 0);

                glow.bind_buffer(glow::ARRAY_BUFFER, Some(weight_vbo));
                glow.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    cast_slice(&weights),
                    glow::STATIC_DRAW,
                );
                glow.enable_vertex_attrib_array(2);
                glow.vertex_attrib_pointer_f32(2, 4, glow::FLOAT, false, 16, 0);

                glow.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
                glow.buffer_data_u8_slice(
                    glow::ELEMENT_ARRAY_BUFFER,
                    cast_slice(&indices),
                    glow::STATIC_DRAW,
                );
                glow.bind_vertex_array(None);
                glow.bind_buffer(glow::ARRAY_BUFFER, None);
                glow.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            }

            primitives.push(PrimitiveMesh {
                vao,
                vbo,
                joint_vbo,
                weight_vbo,
                ebo,
                index_count: indices.len() as i32,
                joint_names: joint_names.clone(),
                inverse_bind_matrices: inverse_bind_matrices.clone(),
            });
        }
    }

    if primitives.is_empty() {
        return Err("model contains no mesh primitives".to_string());
    }

    Ok(ModelMesh { primitives })
}

const MAX_BONES: usize = 96;
const MODEL_UNITS_TO_SOURCE: f32 = 39.3701;

fn skin_palette(primitive: &PrimitiveMesh, skeleton: &[BoneTransform]) -> Vec<f32> {
    // blend each joint with its inverse bind matrix
    let mut palette = vec![0.0; MAX_BONES * 16];

    for (joint_index, _joint_name) in primitive.joint_names.iter().enumerate() {
        if joint_index >= MAX_BONES {
            break;
        }

        let Some(transform) = skeleton.get(joint_index) else {
            palette[joint_index * 16..joint_index * 16 + 16]
                .copy_from_slice(Mat4::IDENTITY.as_ref());
            continue;
        };

        let inverse_bind = primitive
            .inverse_bind_matrices
            .get(joint_index)
            .copied()
            .unwrap_or(Mat4::IDENTITY);
        let correction = Mat4::from_scale(Vec3::splat(MODEL_UNITS_TO_SOURCE));
        let skin = transform.matrix * correction * inverse_bind;
        palette[joint_index * 16..joint_index * 16 + 16]
            .copy_from_slice(skin.to_cols_array().as_ref());
    }

    for joint_index in primitive.joint_names.len().min(MAX_BONES)..MAX_BONES {
        palette[joint_index * 16..joint_index * 16 + 16].copy_from_slice(Mat4::IDENTITY.as_ref());
    }

    palette
}

fn build_program(glow: &glow::Context) -> Result<glow::Program, String> {
    let vertex = include_str!("shaders/model.vert");
    let fragment = include_str!("shaders/model.frag");
    opengl::build_program(glow, vertex, fragment)
}

pub fn model_matrix() -> [f32; 16] {
    Mat4::IDENTITY.to_cols_array()
}
