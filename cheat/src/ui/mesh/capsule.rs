use glam::{Mat4, Vec3};

use super::asset::{MeshAsset, Submesh};

// tessellation; doesn't need to be dense for a debug-visualization shape
const SEGMENTS: usize = 10;
const CAP_RINGS: usize = 3;

/// appends one ring of `SEGMENTS` vertices at height `y`, radius `r`
#[allow(clippy::too_many_arguments)]
fn push_ring(
    positions: &mut Vec<Vec3>,
    normals: &mut Vec<Vec3>,
    rings: &mut Vec<usize>,
    mid: Vec3,
    dir: Vec3,
    tangent: Vec3,
    bitangent: Vec3,
    y: f32,
    r: f32,
    normal_y: f32,
    normal_r: f32,
) {
    rings.push(positions.len());
    for i in 0..SEGMENTS {
        let angle = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
        let (sin, cos) = angle.sin_cos();
        let radial = tangent * cos + bitangent * sin;
        positions.push(mid + dir * y + radial * r);
        normals.push((dir * normal_y + radial * normal_r).normalize());
    }
}

/// builds a solid capsule mesh (cylinder + two hemispheres) from one real
/// hitbox's point0/point1/radius, already in inches like live bone positions
pub fn build_capsule_mesh(point0: Vec3, point1: Vec3, radius: f32) -> MeshAsset {
    let axis = point1 - point0;
    let length = axis.length();
    let dir = if length > 1e-4 {
        axis / length
    } else {
        Vec3::Z
    };
    let arbitrary = if dir.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
    let tangent = dir.cross(arbitrary).normalize();
    let bitangent = dir.cross(tangent);
    let mid = (point0 + point1) * 0.5;
    let half_len = length * 0.5;

    let mut positions: Vec<Vec3> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut rings: Vec<usize> = Vec::new();

    let pole_bottom = positions.len();
    positions.push(mid - dir * (half_len + radius));
    normals.push(-dir);

    for stack in 1..=CAP_RINGS {
        let phi = std::f32::consts::FRAC_PI_2 * (stack as f32 / (CAP_RINGS as f32 + 1.0));
        push_ring(
            &mut positions,
            &mut normals,
            &mut rings,
            mid,
            dir,
            tangent,
            bitangent,
            -half_len - radius * phi.cos(),
            radius * phi.sin(),
            -phi.cos(),
            phi.sin(),
        );
    }
    push_ring(
        &mut positions,
        &mut normals,
        &mut rings,
        mid,
        dir,
        tangent,
        bitangent,
        -half_len,
        radius,
        0.0,
        1.0,
    );
    push_ring(
        &mut positions,
        &mut normals,
        &mut rings,
        mid,
        dir,
        tangent,
        bitangent,
        half_len,
        radius,
        0.0,
        1.0,
    );
    for stack in (1..=CAP_RINGS).rev() {
        let phi = std::f32::consts::FRAC_PI_2 * (stack as f32 / (CAP_RINGS as f32 + 1.0));
        push_ring(
            &mut positions,
            &mut normals,
            &mut rings,
            mid,
            dir,
            tangent,
            bitangent,
            half_len + radius * phi.cos(),
            radius * phi.sin(),
            phi.cos(),
            phi.sin(),
        );
    }

    let pole_top = positions.len();
    positions.push(mid + dir * (half_len + radius));
    normals.push(dir);

    let mut indices: Vec<u32> = Vec::new();

    let first_ring = rings[0];
    for i in 0..SEGMENTS {
        let a = first_ring + i;
        let b = first_ring + (i + 1) % SEGMENTS;
        indices.extend_from_slice(&[pole_bottom as u32, a as u32, b as u32]);
    }

    for pair in rings.windows(2) {
        let (r0, r1) = (pair[0], pair[1]);
        for i in 0..SEGMENTS {
            let j = (i + 1) % SEGMENTS;
            let a0 = (r0 + i) as u32;
            let a1 = (r0 + j) as u32;
            let b0 = (r1 + i) as u32;
            let b1 = (r1 + j) as u32;
            indices.extend_from_slice(&[a0, b0, b1, a0, b1, a1]);
        }
    }

    let last_ring = *rings.last().unwrap();
    for i in 0..SEGMENTS {
        let a = last_ring + i;
        let b = last_ring + (i + 1) % SEGMENTS;
        indices.extend_from_slice(&[pole_top as u32, b as u32, a as u32]);
    }

    let vertex_count = positions.len();
    let submesh = Submesh {
        name: "capsule".to_string(),
        positions: positions.iter().map(|p| p.to_array()).collect(),
        normals: normals.iter().map(|n| n.to_array()).collect(),
        uvs: vec![[0.0, 0.0]; vertex_count],
        joint_indices: vec![[0, 0, 0, 0]; vertex_count],
        joint_weights: vec![[1.0, 0.0, 0.0, 0.0]; vertex_count],
        indices,
    };

    MeshAsset {
        joint_names: vec!["capsule".to_string()],
        inverse_bind: vec![Mat4::IDENTITY],
        parent_indices: vec![-1],
        submeshes: vec![submesh],
    }
}
