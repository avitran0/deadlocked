use std::collections::HashSet;

use bytemuck::{Pod, Zeroable};

use crate::{cs2::CS2, parser::bvh::Triangle};

const MAX_VECTOR_ITEMS: usize = 2_000_000;

pub fn read_bvh(cs2: &CS2) -> Option<Vec<Triangle>> {
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
    if body_count <= 0 || body_count as usize > MAX_VECTOR_ITEMS {
        return None;
    }

    let mut triangles = Vec::new();
    let mut seen_shapes = HashSet::new();

    for body_index in 0..body_count as usize {
        let body = bodies + body_index * 88;
        if cs2.process.read::<u32>(body + 0x40) != 2 {
            continue;
        }

        let root: i32 = cs2.process.read(body);
        let nodes_ptr: usize = cs2.process.read(body + 0x18);
        let count_a: i32 = cs2.process.read(body + 0x08);
        let count_b: i32 = cs2.process.read(body + 0x10);
        if nodes_ptr == 0
            || count_a <= 0
            || count_a != count_b
            || count_a as usize > MAX_VECTOR_ITEMS
            || root < 0
            || root >= count_a
        {
            continue;
        }

        let nodes: Vec<OuterNode> =
            cs2.process
                .read_typed_vec(nodes_ptr, size_of::<OuterNode>(), count_a as usize);
        if nodes.len() != count_a as usize {
            continue;
        }

        let mut stack = vec![root];
        let mut visited = HashSet::new();
        while let Some(index) = stack.pop() {
            if index < 0 || index >= count_a || !visited.insert(index) {
                continue;
            }
            let node = nodes[index as usize];
            if node.left == -1 && node.right == -1 {
                if node.shape != 0 && seen_shapes.insert(node.shape) {
                    process_shape(cs2, node.shape, &mut triangles);
                }
                continue;
            }
            if node.left >= 0 {
                stack.push(node.left);
            }
            if node.right >= 0 {
                stack.push(node.right);
            }
        }
    }

    (!triangles.is_empty()).then_some(triangles)
}

fn process_shape(cs2: &CS2, shape: usize, triangles: &mut Vec<Triangle>) {
    // m_nInteractsAs: retain only world geometry (bit 0). this excludes
    // clip/trigger-style collision volumes that otherwise occlude visibility.
    if cs2.process.read::<u64>(shape + 0x50) & 1 == 0 {
        return;
    }
    match rtti_name(cs2, shape).as_str() {
        "12CRnMeshShape" => process_mesh(cs2, shape, triangles),
        "12CRnHullShape" => process_hull(cs2, shape, triangles),
        _ => {}
    }
}

fn process_mesh(cs2: &CS2, shape: usize, triangles: &mut Vec<Triangle>) {
    let mesh: usize = cs2.process.read(shape + 0xC0);
    if mesh == 0 {
        return;
    }
    let vertices: UtlVector = cs2.process.read(mesh + 0x30);
    let indices: UtlVector = cs2.process.read(mesh + 0x48);
    if !valid_vector(vertices) || !valid_vector(indices) {
        return;
    }

    let vertices: Vec<glam::Vec3> = cs2.process.read_typed_vec(
        vertices.data,
        size_of::<glam::Vec3>(),
        vertices.count as usize,
    );
    let indices: Vec<Tri> =
        cs2.process
            .read_typed_vec(indices.data, size_of::<Tri>(), indices.count as usize);
    for tri in indices {
        let [a, b, c] = tri.idx;
        if a < 0 || b < 0 || c < 0 {
            continue;
        }
        let (a, b, c) = (a as usize, b as usize, c as usize);
        let (Some(&v0), Some(&v1), Some(&v2)) = (vertices.get(a), vertices.get(b), vertices.get(c))
        else {
            continue;
        };
        if (v1 - v0).cross(v2 - v0).length_squared() <= f32::EPSILON {
            continue;
        }
        triangles.push(Triangle { v0, v1, v2 });
    }
}

fn process_hull(cs2: &CS2, shape: usize, triangles: &mut Vec<Triangle>) {
    let hull: usize = cs2.process.read(shape + 0xB8);
    if hull == 0 {
        return;
    }
    let scale: f32 = cs2.process.read(shape + 0xB0);
    if !scale.is_finite() {
        return;
    }
    let vertices: UtlVector = cs2.process.read(hull + 0x70);
    let edges: UtlVector = cs2.process.read(hull + 0xC8);
    let faces: UtlVector = cs2.process.read(hull + 0xE0);
    if !valid_vector(vertices) || !valid_vector(edges) || !valid_vector(faces) {
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
    let faces: Vec<u8> = (0..faces.count as usize)
        .map(|i| cs2.process.read(faces.data + i))
        .collect();
    if vertices.is_empty() || edges.is_empty() {
        return;
    }

    for &start in &faces {
        let start = start as usize;
        if start >= edges.len() {
            continue;
        }
        let mut current = start;
        let mut face_vertices = Vec::new();
        let mut visited = HashSet::new();
        loop {
            if current >= edges.len() || !visited.insert(current) {
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
                break;
            }
            if visited.len() >= edges.len() {
                face_vertices.clear();
                break;
            }
        }
        if current != start || face_vertices.len() < 3 {
            continue;
        }
        for i in 1..face_vertices.len() - 1 {
            let (v0, v1, v2) = (face_vertices[0], face_vertices[i], face_vertices[i + 1]);
            if (v1 - v0).cross(v2 - v0).length_squared() > f32::EPSILON {
                triangles.push(Triangle { v0, v1, v2 });
            }
        }
    }
}

fn valid_vector(vector: UtlVector) -> bool {
    vector.count >= 0
        && vector.count as usize <= MAX_VECTOR_ITEMS
        && (vector.count == 0 || vector.data != 0)
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
