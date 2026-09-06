use std::collections::HashSet;

use egui_glow::glow::{self, HasContext as _};
use glam::Mat4;

use super::asset::{MeshAsset, Submesh};

// GLSL uniform array size; CS2 agent skeletons have 94 joints
const MAX_BONES: usize = 128;

/// maps each joint to itself if in `named`, else its nearest ancestor that is
fn resolve_visibility_sources(parent_indices: &[i32], named: &HashSet<usize>) -> Vec<usize> {
    (0..parent_indices.len())
        .map(|joint| {
            if named.contains(&joint) {
                return joint;
            }
            let mut current = joint;
            // bounded in case of a malformed/cyclic parent chain
            for _ in 0..parent_indices.len() {
                let Some(&parent) = parent_indices.get(current) else {
                    return joint;
                };
                let Ok(parent) = usize::try_from(parent) else {
                    return joint; // no ancestor found, falls back to "visible"
                };
                if named.contains(&parent) {
                    return parent;
                }
                current = parent;
            }
            joint
        })
        .collect()
}

const VERTEX_SHADER: &str = r#"#version 330 core
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in uvec4 in_joints;
layout(location = 3) in vec4 in_weights;

uniform mat4 u_bone_matrices[128];
uniform mat4 u_view_projection;
// 1.0 = visible, 0.0 = behind cover, blended per-vertex like skinning
uniform float u_bone_visibility[128];
// 1.0 = on screen, 0.0 = off screen / behind the camera, same blending
uniform float u_bone_on_screen[128];

out vec3 v_normal;
out float v_visibility;
out float v_on_screen;

void main() {
    mat4 skin =
        in_weights.x * u_bone_matrices[in_joints.x] +
        in_weights.y * u_bone_matrices[in_joints.y] +
        in_weights.z * u_bone_matrices[in_joints.z] +
        in_weights.w * u_bone_matrices[in_joints.w];

    vec4 world_position = skin * vec4(in_position, 1.0);
    gl_Position = u_view_projection * world_position;
    v_normal = mat3(skin) * in_normal;
    v_visibility =
        in_weights.x * u_bone_visibility[in_joints.x] +
        in_weights.y * u_bone_visibility[in_joints.y] +
        in_weights.z * u_bone_visibility[in_joints.z] +
        in_weights.w * u_bone_visibility[in_joints.w];
    v_on_screen =
        in_weights.x * u_bone_on_screen[in_joints.x] +
        in_weights.y * u_bone_on_screen[in_joints.y] +
        in_weights.z * u_bone_on_screen[in_joints.z] +
        in_weights.w * u_bone_on_screen[in_joints.w];
}
"#;

const FRAGMENT_SHADER: &str = r#"#version 330 core
in vec3 v_normal;
in float v_visibility;
in float v_on_screen;
uniform vec4 u_color;
uniform vec4 u_outline_color;
out vec4 frag_color;

void main() {
    if (v_on_screen < 0.5) {
        discard;
    }

    float rim = 1.0 - clamp(abs(normalize(v_normal).z), 0.0, 1.0);
    // outline alpha is the blend strength at full rim; 0 = untouched
    vec3 rgb = mix(u_color.rgb, u_outline_color.rgb, rim * u_outline_color.a);
    rgb *= mix(0.35, 1.0, v_visibility);
    frag_color = vec4(rgb, u_color.a * mix(0.55, 1.0, rim));
}
"#;

// buffer handles aren't retained past upload, only the VAO is needed to draw
struct GpuSubmesh {
    vao: glow::VertexArray,
    index_count: i32,
}

pub struct MeshRenderer {
    program: glow::Program,
    submeshes: Vec<GpuSubmesh>,
    inverse_bind: Vec<Mat4>,
    visibility_source: Vec<usize>,
    u_bone_matrices: glow::UniformLocation,
    u_view_projection: glow::UniformLocation,
    u_color: glow::UniformLocation,
    u_outline_color: glow::UniformLocation,
    u_bone_visibility: glow::UniformLocation,
    u_bone_on_screen: glow::UniformLocation,
}

impl MeshRenderer {
    /// # Safety
    /// `gl` must be current on the calling thread.
    pub unsafe fn new(
        gl: &glow::Context,
        asset: &MeshAsset,
        named: &HashSet<usize>,
    ) -> Result<Self, String> {
        unsafe {
            let program = compile_program(gl, VERTEX_SHADER, FRAGMENT_SHADER)?;

            let u_bone_matrices = gl
                .get_uniform_location(program, "u_bone_matrices[0]")
                .ok_or("missing u_bone_matrices uniform")?;
            let u_view_projection = gl
                .get_uniform_location(program, "u_view_projection")
                .ok_or("missing u_view_projection uniform")?;
            let u_color = gl
                .get_uniform_location(program, "u_color")
                .ok_or("missing u_color uniform")?;
            let u_outline_color = gl
                .get_uniform_location(program, "u_outline_color")
                .ok_or("missing u_outline_color uniform")?;
            let u_bone_visibility = gl
                .get_uniform_location(program, "u_bone_visibility[0]")
                .ok_or("missing u_bone_visibility uniform")?;
            let u_bone_on_screen = gl
                .get_uniform_location(program, "u_bone_on_screen[0]")
                .ok_or("missing u_bone_on_screen uniform")?;

            let mut submeshes = Vec::with_capacity(asset.submeshes.len());
            for submesh in &asset.submeshes {
                submeshes.push(upload_submesh(gl, submesh)?);
            }

            Ok(Self {
                program,
                submeshes,
                inverse_bind: asset.inverse_bind[..asset.inverse_bind.len().min(MAX_BONES)]
                    .to_vec(),
                visibility_source: resolve_visibility_sources(
                    &asset.parent_indices[..asset.parent_indices.len().min(MAX_BONES)],
                    named,
                ),
                u_bone_matrices,
                u_view_projection,
                u_color,
                u_outline_color,
                u_bone_visibility,
                u_bone_on_screen,
            })
        }
    }

    pub fn skin_matrices(&self, live_bones: &[shared::BoneTransform]) -> Vec<Mat4> {
        // mesh data is in meters, live bones in inches
        const METERS_TO_INCHES: f32 = 39.3701;
        let correction = Mat4::from_scale(glam::Vec3::splat(METERS_TO_INCHES));
        self.inverse_bind
            .iter()
            .enumerate()
            .map(|(joint, inverse_bind)| match live_bones.get(joint) {
                Some(bone) => {
                    Mat4::from_rotation_translation(bone.rotation, bone.position)
                        * correction
                        * *inverse_bind
                }
                None => Mat4::IDENTITY,
            })
            .collect()
    }

    /// # Safety
    /// `gl` must be current on the calling thread.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn draw(
        &self,
        gl: &glow::Context,
        skin_matrices: &[Mat4],
        view_projection: Mat4,
        color: [f32; 4],
        outline_color: [f32; 4],
        mode: crate::config::player::ModelEspMode,
        bone_visibility: &[f32],
        bone_on_screen: &[f32],
    ) {
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_matrix_4_f32_slice(
                Some(&self.u_view_projection),
                false,
                &view_projection.to_cols_array(),
            );
            gl.uniform_4_f32(Some(&self.u_color), color[0], color[1], color[2], color[3]);
            gl.uniform_4_f32(
                Some(&self.u_outline_color),
                outline_color[0],
                outline_color[1],
                outline_color[2],
                outline_color[3],
            );
            let count = skin_matrices.len().min(self.inverse_bind.len());
            let flat: Vec<f32> = skin_matrices[..count]
                .iter()
                .flat_map(|m| m.to_cols_array())
                .collect();
            gl.uniform_matrix_4_f32_slice(Some(&self.u_bone_matrices), false, &flat);

            let visibility: Vec<f32> = self
                .visibility_source
                .iter()
                .map(|&source| bone_visibility.get(source).copied().unwrap_or(1.0))
                .collect();
            gl.uniform_1_f32_slice(Some(&self.u_bone_visibility), &visibility);

            let on_screen: Vec<f32> = self
                .visibility_source
                .iter()
                .map(|&source| bone_on_screen.get(source).copied().unwrap_or(1.0))
                .collect();
            gl.uniform_1_f32_slice(Some(&self.u_bone_on_screen), &on_screen);

            gl.enable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            let wireframe = mode == crate::config::player::ModelEspMode::Wireframe;
            if wireframe {
                gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
            }

            for submesh in &self.submeshes {
                gl.bind_vertex_array(Some(submesh.vao));
                gl.draw_elements(glow::TRIANGLES, submesh.index_count, glow::UNSIGNED_INT, 0);
            }
            gl.bind_vertex_array(None);

            if wireframe {
                gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
            }
            gl.disable(glow::DEPTH_TEST);
        }
    }
}

unsafe fn compile_shader(
    gl: &glow::Context,
    kind: u32,
    source: &str,
) -> Result<glow::Shader, String> {
    unsafe {
        let shader = gl.create_shader(kind)?;
        gl.shader_source(shader, source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            let log = gl.get_shader_info_log(shader);
            gl.delete_shader(shader);
            return Err(log);
        }
        Ok(shader)
    }
}

unsafe fn compile_program(
    gl: &glow::Context,
    vertex_source: &str,
    fragment_source: &str,
) -> Result<glow::Program, String> {
    unsafe {
        let vertex = compile_shader(gl, glow::VERTEX_SHADER, vertex_source)?;
        let fragment = compile_shader(gl, glow::FRAGMENT_SHADER, fragment_source)?;

        let program = gl.create_program()?;
        gl.attach_shader(program, vertex);
        gl.attach_shader(program, fragment);
        gl.link_program(program);

        gl.detach_shader(program, vertex);
        gl.detach_shader(program, fragment);
        gl.delete_shader(vertex);
        gl.delete_shader(fragment);

        if !gl.get_program_link_status(program) {
            let log = gl.get_program_info_log(program);
            gl.delete_program(program);
            return Err(log);
        }
        Ok(program)
    }
}

unsafe fn upload_buffer<T: bytemuck::Pod>(
    gl: &glow::Context,
    target: u32,
    data: &[T],
) -> Result<glow::Buffer, String> {
    unsafe {
        let buffer = gl.create_buffer()?;
        gl.bind_buffer(target, Some(buffer));
        gl.buffer_data_u8_slice(target, bytemuck::cast_slice(data), glow::STATIC_DRAW);
        Ok(buffer)
    }
}

unsafe fn upload_submesh(gl: &glow::Context, submesh: &Submesh) -> Result<GpuSubmesh, String> {
    unsafe {
        let vao = gl.create_vertex_array()?;
        gl.bind_vertex_array(Some(vao));

        upload_buffer(gl, glow::ARRAY_BUFFER, &submesh.positions)?;
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

        upload_buffer(gl, glow::ARRAY_BUFFER, &submesh.normals)?;
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 0, 0);

        upload_buffer(gl, glow::ARRAY_BUFFER, &submesh.joint_indices)?;
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_i32(2, 4, glow::UNSIGNED_SHORT, 0, 0);

        upload_buffer(gl, glow::ARRAY_BUFFER, &submesh.joint_weights)?;
        gl.enable_vertex_attrib_array(3);
        gl.vertex_attrib_pointer_f32(3, 4, glow::FLOAT, false, 0, 0);

        let ebo = gl.create_buffer()?;
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&submesh.indices),
            glow::STATIC_DRAW,
        );

        gl.bind_vertex_array(None);

        Ok(GpuSubmesh {
            vao,
            index_count: submesh.indices.len() as i32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player_bones() -> HashSet<usize> {
        use strum::IntoEnumIterator as _;
        shared::Bones::iter()
            .map(|bone| bone.u64() as usize)
            .collect()
    }

    #[test]
    fn named_bones_resolve_to_themselves() {
        // index 1 = Bones::Hip, a real named joint
        let parents = vec![-1i32, -1];
        assert_eq!(resolve_visibility_sources(&parents, &player_bones())[1], 1);
    }

    #[test]
    fn unnamed_joint_inherits_nearest_named_ancestor() {
        // 0 = unnamed root, 1 = Hip (named), 8 = an unnamed gap in the
        // enum, 23 = unnamed finger tip chained back up to RightHand (15)
        let mut parents = vec![-1i32; 24];
        parents[1] = 0; // Hip's parent is the unnamed root
        parents[15] = 1; // RightHand's parent chain eventually reaches Hip
        parents[8] = 15; // an unnamed joint parented to RightHand
        parents[23] = 8; // fingertip parented to that unnamed joint

        let sources = resolve_visibility_sources(&parents, &player_bones());
        assert_eq!(sources[1], 1, "Hip is named, resolves to itself");
        assert_eq!(sources[15], 15, "RightHand is named, resolves to itself");
        assert_eq!(sources[8], 15, "inherits RightHand");
        assert_eq!(sources[23], 15, "inherits RightHand through the chain");
    }

    #[test]
    fn joint_with_no_named_ancestor_falls_back_to_itself() {
        let parents = vec![-1i32, 0];
        let sources = resolve_visibility_sources(&parents, &player_bones());
        assert_eq!(sources[0], 0);
        assert_eq!(
            sources[1], 1,
            "no named ancestor, falls back to default-visible"
        );
    }
}
