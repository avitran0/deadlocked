use std::collections::{HashMap, HashSet};
use std::mem::size_of;

use bytemuck::{Pod, Zeroable};

use crate::{cs2::CS2, parser::bvh::Triangle};

const MAX_VECTOR_ITEMS: usize = 2_000_000;

#[derive(Default, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct PhysicsBodyData {
    root: i32,
    _pad0: u32,
    count_a: i32,
    _pad1: u32,
    count_b: i32,
    _pad2: u32,
    nodes_ptr: usize,
    _pad3: [u8; 32],
    body_type: u32,
    _pad4: [u8; 20],
}

const _: () = assert!(size_of::<PhysicsBodyData>() == 88);

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

    let mut stack = Vec::with_capacity(32);
    let mut visited_nodes = Vec::new();
    let mut hull_face_vertices = Vec::with_capacity(16);
    let mut hull_visited_edges = Vec::new();

    let mut vtable_cache = HashMap::with_capacity(16);

    for body_index in 0..body_count as usize {
        let body_address = bodies + body_index * 88;
        
        let body_data: PhysicsBodyData = cs2.process.read(body_address);
        if body_data.body_type != 2 {
            continue;
        }

        let root = body_data.root;
        let nodes_ptr = body_data.nodes_ptr;
        let count_a = body_data.count_a;
        let count_b = body_data.count_b;

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

        stack.clear();
        stack.push(root);

        visited_nodes.clear();
        visited_nodes.resize(count_a as usize, false);

        while let Some(index) = stack.pop() {
            if index < 0 || index >= count_a {
                continue;
            }

            let idx = index as usize;
            if visited_nodes[idx] {
                continue;
            }
            visited_nodes[idx] = true;

            let node = nodes[idx];
            if node.left == -1 && node.right == -1 {
                if node.shape != 0 && seen_shapes.insert(node.shape) {
                    process_shape(
                        cs2, 
                        node.shape, 
                        &mut triangles, 
                        &mut hull_face_vertices, 
                        &mut hull_visited_edges,
                        &mut vtable_cache,
                    );
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

fn process_shape(
    cs2: &CS2, 
    shape: usize, 
    triangles: &mut Vec<Triangle>,
    face_vertices: &mut Vec<glam::Vec3>,
    visited_edges: &mut Vec<bool>,
    vtable_cache: &mut HashMap<usize, String>,
) {
    if cs2.process.read::<u64>(shape + 0x50) & 1 == 0 {
        return;
    }

    let vtable: usize = cs2.process.read(shape);
    if vtable == 0 {
        return;
    }

    if let Some(type_name) = vtable_cache.get(&vtable) {
        dispatch_shape(type_name, cs2, shape, triangles, face_vertices, visited_edges);
        return;
    }

    let rtti: usize = cs2.process.read(vtable - 0x08);
    if rtti == 0 { return; }
    let name_ptr: usize = cs2.process.read(rtti + 0x08);
    if name_ptr == 0 { return; }
    
    let type_name = cs2.process.read_string(name_ptr);
    dispatch_shape(&type_name, cs2, shape, triangles, face_vertices, visited_edges);
    
    vtable_cache.insert(vtable, type_name);
}

#[inline(always)]
fn dispatch_shape(
    type_name: &str,
    cs2: &CS2,
    shape: usize,
    triangles: &mut Vec<Triangle>,
    face_vertices: &mut Vec<glam::Vec3>,
    visited_edges: &mut Vec<bool>,
) {
    match type_name {
        "12CRnMeshShape" => process_mesh(cs2, shape, triangles),
        "12CRnHullShape" => process_hull(cs2, shape, triangles, face_vertices, visited_edges),
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

fn process_hull(
    cs2: &CS2, 
    shape: usize, 
    triangles: &mut Vec<Triangle>,
    face_vertices: &mut Vec<glam::Vec3>,
    visited_edges: &mut Vec<bool>,
) {
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

    let faces: Vec<u8> = cs2.process.read_typed_vec(
        faces.data, 
        size_of::<u8>(), 
        faces.count as usize
    );

    if vertices.is_empty() || edges.is_empty() {
        return;
    }

    for &start in &faces {
        let start = start as usize;
        if start >= edges.len() {
            continue;
        }
        let mut current = start;
        
        face_vertices.clear();
        visited_edges.clear();
        visited_edges.resize(edges.len(), false);
        
        let mut count = 0;
        loop {
            if current >= edges.len() || visited_edges[current] {
                break;
            }
            visited_edges[current] = true;

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
            
            count += 1;
            if count >= edges.len() {
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
