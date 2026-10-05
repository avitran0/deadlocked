use std::collections::HashSet;

use bytemuck::{Pod, Zeroable};

use crate::{
    cs2::CS2,
    parser::bvh::{SurfaceMaterial, Triangle},
};

const MAX_VECTOR_ITEMS: usize = 2_000_000;
const MAX_MESH_MATERIALS: usize = u8::MAX as usize + 1;
const MAX_SURFACE_MATERIALS: usize = u16::MAX as usize + 1;
const SHAPE_DEFAULT_MATERIAL: usize = 0x20;
const MESH_MATERIAL_COUNT: usize = 0xD0;
const MESH_MATERIALS: usize = 0xD8;
const MATERIAL_RECORD_SIZE: usize = 0x30;
const MATERIAL_RECORD_PROPERTY: usize = 0x28;
const SURFACE_PROPERTY_INDEX: usize = 0x10;

pub fn read_bvh(cs2: &CS2) -> Option<(Vec<Triangle>, Vec<SurfaceMaterial>)> {
    let mut materials = read_surface_materials(cs2);
    let resolver = MaterialResolver {
        game_material_count: materials.len(),
        fallback_material: materials.len(),
    };
    materials.push(SurfaceMaterial::default());

    let world: usize = cs2.process.read(cs2.offsets.direct.vphys_world);
    if world == 0 {
        return None;
    }
    let inner: usize = cs2.process.read(world + 0x30);
    if inner == 0 {
        return None;
    }
    let bodies: usize = cs2.process.read(inner + 0x118);
    if bodies == 0 {
        return None;
    }
    let body_count: i32 = cs2.process.read(bodies + 0x268);
    if body_count <= 0 {
        return None;
    }

    let mut triangles = Vec::new();
    let mut seen_shapes = HashSet::new();

    for body_index in 0..body_count {
        let body = bodies + body_index as usize * 88;
        if cs2.process.read::<u32>(body + 0x40) != 2 {
            continue;
        }

        let root: i32 = cs2.process.read(body);
        let nodes_ptr: usize = cs2.process.read(body + 0x18);
        let count_a: i32 = cs2.process.read(body + 0x08);
        let count_b: i32 = cs2.process.read(body + 0x10);
        let node_count = count_a.max(count_b);
        if nodes_ptr == 0 || node_count <= 0 || node_count > 0x100000 || root >= node_count {
            continue;
        }

        let nodes: Vec<OuterNode> =
            cs2.process
                .read_typed_vec(nodes_ptr, size_of::<OuterNode>(), node_count as usize);
        let mut leaves = Vec::with_capacity(256);
        let mut stack = Vec::with_capacity(128);
        stack.push(root);

        while let Some(index) = stack.pop() {
            if index < 0 || index >= node_count {
                continue;
            }
            let node = nodes[index as usize];
            if node.left == -1 && node.right == -1 {
                leaves.push(node.shape);
            }
            if node.left != -1 {
                stack.push(node.left);
            }
            if node.right != -1 {
                stack.push(node.right);
            }
        }

        for shape in leaves {
            if seen_shapes.insert(shape) {
                process_shape(cs2, shape, resolver, &mut triangles);
            }
        }
    }

    (!triangles.is_empty()).then_some((triangles, materials))
}

fn read_surface_materials(cs2: &CS2) -> Vec<SurfaceMaterial> {
    let controller = cs2.offsets.direct.surface_properties;
    if controller == 0 {
        utils::warn!("empty surface properties");
        return Vec::new();
    }
    let entries: UtlVector = cs2.process.read(controller + 0x20);
    if !valid_vector(entries)
        || entries.count == 0
        || entries.count as usize > MAX_SURFACE_MATERIALS
    {
        utils::warn!("invalid surface properties vector");
        return Vec::new();
    }
    let entries: Vec<SurfaceMaterialEntry> = cs2.process.read_typed_vec(
        entries.data,
        size_of::<SurfaceMaterialEntry>(),
        entries.count as usize,
    );
    let materials: Vec<_> = entries
        .into_iter()
        .filter_map(|entry| {
            if entry.penetration_modifier.is_finite() && entry.damage_modifier.is_finite() {
                Some(SurfaceMaterial {
                    penetration_modifier: entry.penetration_modifier,
                    damage_modifier: entry.damage_modifier,
                    surface_type: entry.surface_type,
                })
            } else {
                None
            }
        })
        .collect();

    utils::info!("parsed {} surface properties", materials.len());
    materials
}

fn process_shape(
    cs2: &CS2,
    shape: usize,
    resolver: MaterialResolver,
    triangles: &mut Vec<Triangle>,
) {
    let interacts_as: u64 = cs2.process.read(shape + 0x50);
    if interacts_as & 0xffff == 0 || interacts_as == 0x40000008 || interacts_as == 0x40000030 {
        return;
    }

    let shape_id = triangles.len();
    match rtti_name(cs2, shape).as_str() {
        "12CRnMeshShape" => process_mesh(cs2, shape, resolver, shape_id, triangles),
        "12CRnHullShape" => process_hull(cs2, shape, resolver, shape_id, triangles),
        _ => {}
    }
}

fn process_mesh(
    cs2: &CS2,
    shape: usize,
    resolver: MaterialResolver,
    shape_id: usize,
    triangles: &mut Vec<Triangle>,
) {
    let mesh: usize = cs2.process.read(shape + 0xC0);
    if mesh == 0 {
        return;
    }

    let vertices: UtlVector = cs2.process.read(mesh + cs2.offsets.mesh.vertices);
    let indices: UtlVector = cs2.process.read(mesh + cs2.offsets.mesh.triangles);
    let vertices: Vec<glam::Vec3> = cs2.process.read_typed_vec(
        vertices.data,
        size_of::<glam::Vec3>(),
        vertices.count as usize,
    );
    let indices: Vec<Tri> =
        cs2.process
            .read_typed_vec(indices.data, size_of::<Tri>(), indices.count as usize);

    let mesh_materials: UtlVector = cs2.process.read(mesh + cs2.offsets.mesh.materials);
    let mesh_materials: Vec<u8> = if valid_vector(mesh_materials) {
        cs2.process.read_typed_vec(
            mesh_materials.data,
            size_of::<u8>(),
            mesh_materials.count as usize,
        )
    } else {
        Vec::new()
    };
    let fallback_material = resolver.shape_material(cs2, shape + SHAPE_DEFAULT_MATERIAL);
    let resolved_materials = resolved_mesh_materials(cs2, shape, resolver.game_material_count);

    for (triangle_index, triangle) in indices.iter().enumerate() {
        let [a, b, c] = triangle.idx;
        if a < 0 || b < 0 || c < 0 {
            continue;
        }
        let (a, b, c) = (a as usize, b as usize, c as usize);
        let Some((&v0, &v1, &v2)) = vertices
            .get(a)
            .zip(vertices.get(b))
            .zip(vertices.get(c))
            .map(|((v0, v1), v2)| (v0, v1, v2))
        else {
            continue;
        };
        if (v1 - v0).cross(v2 - v0).length_squared() <= f32::EPSILON {
            continue;
        }

        let material = resolve_mesh_material(
            mesh_materials.get(triangle_index).copied(),
            &resolved_materials,
            fallback_material,
        );
        triangles.push(Triangle::new(v0, v1, v2, material, shape_id));
    }
}

fn process_hull(
    cs2: &CS2,
    shape: usize,
    resolver: MaterialResolver,
    shape_id: usize,
    triangles: &mut Vec<Triangle>,
) {
    let hull: usize = cs2.process.read(shape + 0xB8);
    if hull == 0 {
        return;
    }
    let raw_scale: f32 = cs2.process.read(shape + 0xB4);
    let scale = if raw_scale.is_finite() && raw_scale > 0.0 {
        raw_scale
    } else {
        1.0
    };
    let vertices: UtlVector = cs2.process.read(hull + cs2.offsets.hull.vertices);
    let edges: UtlVector = cs2.process.read(hull + cs2.offsets.hull.edges);
    let faces: UtlVector = cs2.process.read(hull + cs2.offsets.hull.faces);
    if vertices.data == 0 || edges.data == 0 || faces.data == 0 {
        return;
    }

    let vertices: Vec<glam::Vec3> = cs2.process.read_typed_vec(
        vertices.data,
        size_of::<glam::Vec3>(),
        vertices.count as usize,
    );
    let edges: Vec<HalfEdge> =
        cs2.process
            .read_typed_vec(edges.data, size_of::<HalfEdge>(), edges.count as usize);
    let faces: Vec<u8> = (0..faces.count)
        .map(|index| cs2.process.read(faces.data + index as usize))
        .collect();
    let material = resolver.shape_material(cs2, shape + SHAPE_DEFAULT_MATERIAL);

    for &face_start in &faces {
        let start = face_start as usize;
        let mut current = start;
        let mut face_vertices = Vec::new();
        let mut closed = false;

        for _ in 0..edges.len().min(64) {
            if current >= edges.len() {
                break;
            }
            let edge = edges[current];
            let vertex = edge.origin as usize;
            if vertex >= vertices.len() {
                face_vertices.clear();
                break;
            }
            face_vertices.push(vertices[vertex] * scale);
            current = edge.next as usize;
            if current == start {
                closed = true;
                break;
            }
        }
        if !closed || face_vertices.len() < 3 {
            continue;
        }
        for index in 1..face_vertices.len() - 1 {
            let (v0, v1, v2) = (
                face_vertices[0],
                face_vertices[index],
                face_vertices[index + 1],
            );
            if (v1 - v0).cross(v2 - v0).length_squared() > f32::EPSILON {
                triangles.push(Triangle::new(v0, v1, v2, material, shape_id));
            }
        }
    }
}

#[derive(Clone, Copy)]
struct MaterialResolver {
    game_material_count: usize,
    fallback_material: usize,
}

impl MaterialResolver {
    fn shape_material(self, cs2: &CS2, record: usize) -> usize {
        surface_material_index(cs2, record, self.game_material_count)
            .map(usize::from)
            .unwrap_or(self.fallback_material)
    }
}

fn valid_vector(vector: UtlVector) -> bool {
    vector.count >= 0
        && vector.count as usize <= MAX_VECTOR_ITEMS
        && (vector.count == 0 || vector.data != 0)
}

fn resolved_mesh_materials(cs2: &CS2, shape: usize, material_count: usize) -> Vec<Option<usize>> {
    let count: u32 = cs2.process.read(shape + MESH_MATERIAL_COUNT);
    let data: usize = cs2.process.read(shape + MESH_MATERIALS);
    if count as usize > MAX_MESH_MATERIALS || (count != 0 && data == 0) {
        return Vec::new();
    }
    (0..count as usize)
        .map(|index| {
            surface_material_index(cs2, data + index * MATERIAL_RECORD_SIZE, material_count)
                .map(usize::from)
        })
        .collect()
}

fn surface_material_index(cs2: &CS2, record: usize, material_count: usize) -> Option<u16> {
    let property: usize = cs2.process.read(record + MATERIAL_RECORD_PROPERTY);
    if property == 0 {
        return None;
    }
    let index: u16 = cs2.process.read(property + SURFACE_PROPERTY_INDEX);
    ((index as usize) < material_count).then_some(index)
}

fn resolve_mesh_material(
    material: Option<u8>,
    resolved_materials: &[Option<usize>],
    fallback_material: usize,
) -> usize {
    material
        .and_then(|index| resolved_materials.get(index as usize).copied().flatten())
        .unwrap_or(fallback_material)
}

fn rtti_name(cs2: &CS2, object: usize) -> String {
    let vtable: usize = cs2.process.read(object);
    if vtable == 0 {
        return String::new();
    }
    let rtti: usize = cs2.process.read(vtable - 0x08);
    if rtti == 0 {
        return String::new();
    }
    let name: usize = cs2.process.read(rtti + 0x08);
    if name == 0 {
        return String::new();
    }
    cs2.process.read_string(name)
}

#[repr(C)]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
struct UtlVector {
    count: i32,
    _pad: i32,
    data: usize,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
struct SurfaceMaterialEntry {
    _pad1: [u8; 8],
    penetration_modifier: f32,
    damage_modifier: f32,
    _unknown_10: [u8; 4],
    surface_type: u16,
    _pad_16: u16,
    _pad2: [u8; 8],
}

#[repr(C)]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
struct OuterNode {
    _pad1: [u8; 12],
    left: i32,
    _pad2: [u8; 12],
    right: i32,
    _pad3: [u8; 8],
    shape: usize,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
struct HalfEdge {
    next: u8,
    _twin: u8,
    origin: u8,
    _face: u8,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
struct Tri {
    idx: [i32; 3],
}
