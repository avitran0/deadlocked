use std::collections::HashSet;

use bytemuck::{Pod, Zeroable};

use crate::{cs2::CS2, parser::bvh::Triangle};

pub fn read_bvh(cs2: &CS2) -> Option<Vec<Triangle>> {
    let wld: u64 = cs2.process.read(cs2.offsets.direct.vphys_world);
    if wld == 0 {
        return None;
    }
    let inner: u64 = cs2.process.read(wld + 0x30);
    if inner == 0 {
        return None;
    }
    let bods: u64 = cs2.process.read(inner + 0x118);
    if bods == 0 {
        return None;
    }
    let bdcnt: i32 = cs2.process.read(bods + 0x268);
    if bdcnt == 0 {
        return None;
    }

    let mut triangles = Vec::new();

    for idx in 0..bdcnt {
        let bod = bods + idx as u64 * 88;

        let bdty: u32 = cs2.process.read(bod + 0x40);
        if bdty != 2 {
            continue;
        }

        let rt: i32 = cs2.process.read(bod);
        let ndptr: u64 = cs2.process.read(bod + 0x18);
        if ndptr == 0 {
            continue;
        }
        if rt < 0 {
            continue;
        }

        let cnt: i32 = cs2.process.read(bod + 0x08);

        let outer_buf: Vec<OuterNode> =
            cs2.process
                .read_typed_vec(ndptr, size_of::<OuterNode>(), cnt as usize);

        let mut leaves = Vec::with_capacity(256);

        let mut outer_stack = Vec::with_capacity(128);
        outer_stack.push(rt);

        // collect all outer nodes in outer_stack
        while let Some(index) = outer_stack.pop() {
            if index < 0 || index >= cnt {
                continue;
            }

            let node = outer_buf[index as usize];
            // leaf node, no children
            if node.left == -1 && node.right == -1 {
                leaves.push(node.shape);
            }

            // left and/or right children
            if node.left != -1 {
                outer_stack.push(node.left);
            }
            if node.right != -1 {
                outer_stack.push(node.right);
            }
        }

        // work through all nodes
        let mut seen = HashSet::new();
        for shape in leaves {
            seen.insert(shape);
            // process shape, might be either hull or mesh
            process_shape(cs2, shape, &mut triangles);
        }
    }

    if triangles.is_empty() {
        return None;
    }

    Some(triangles)
}

fn process_shape(cs2: &CS2, shape: u64, triangles: &mut Vec<Triangle>) {
    // Mesh: "12CRnMeshShape"
    // Hull: "12CRnHullShape"
    let rtti_name = rtti_name(cs2, shape);
    match rtti_name.as_ref() {
        "12CRnMeshShape" => process_mesh(cs2, shape, triangles),
        "12CRnHullShape" => process_hull(cs2, shape, triangles),
        _ => {}
    }
}

fn process_mesh(cs2: &CS2, shape: u64, triangles: &mut Vec<Triangle>) {
    // RnMesh_t
    let mesh_data: u64 = cs2.process.read(shape + 0xC0);
    if mesh_data == 0 {
        return;
    }

    // CUtlVector<u8>
    let mats: UtlVector = cs2.process.read(mesh_data + 144);
    // todo: ignore meshes with collision
    if mats.count == 0 {
        return;
    }

    // CUtlVector<RnVertex_t>
    let vertices: UtlVector = cs2.process.read(mesh_data + 48);

    // CUtlVector<RnTriangle_t>
    let mesh_triangles: UtlVector = cs2.process.read(mesh_data + 72);

    let vertices: Vec<glam::Vec3> = cs2.process.read_typed_vec(
        vertices.data,
        size_of::<glam::Vec3>(),
        vertices.count as usize,
    );

    let mesh_triangles: Vec<Tri> = cs2.process.read_typed_vec(
        mesh_triangles.data,
        size_of::<Tri>(),
        mesh_triangles.count as usize,
    );

    for triangle in mesh_triangles {
        let v0 = vertices[triangle.idx[0] as usize];
        let v1 = vertices[triangle.idx[1] as usize];
        let v2 = vertices[triangle.idx[2] as usize];

        triangles.push(Triangle { v0, v1, v2 });
    }
}

fn process_hull(cs2: &CS2, shape: u64, triangles: &mut Vec<Triangle>) {
    // RnHull_t
    let data: u64 = cs2.process.read(shape + 0xB8);
    if data == 0 {
        return;
    }

    let scale: f32 = cs2.process.read(shape + 0xB0);

    let vertices: UtlVector = cs2.process.read(data + 136);

    let edges: UtlVector = cs2.process.read(data + 160);

    let faces: UtlVector = cs2.process.read(data + 184);

    let vertices: Vec<glam::Vec3> = cs2.process.read_typed_vec(
        vertices.data,
        size_of::<glam::Vec3>(),
        vertices.count as usize,
    );

    let edges: Vec<HalfEdge> =
        cs2.process
            .read_typed_vec(edges.data, size_of::<HalfEdge>(), edges.count as usize);

    let faces: Vec<u8> = (0..faces.count)
        .map(|index| {
            cs2.process
                .read::<u8>(faces.data + index as u64 * size_of::<u8>() as u64)
        })
        .collect();

    for face_start_edge in faces {
        let mut face_vertices = Vec::new();
        let mut current_edge_idx = face_start_edge;

        loop {
            let edge = &edges[current_edge_idx as usize];
            let vertex = vertices[edge.origin as usize];

            face_vertices.push(vertex * scale);

            current_edge_idx = edge.next;

            if current_edge_idx == face_start_edge {
                break;
            }
        }

        if face_vertices.len() >= 3 {
            for i in 1..(face_vertices.len() - 1) {
                let v0 = face_vertices[0];
                let v1 = face_vertices[i];
                let v2 = face_vertices[i + 1];

                triangles.push(Triangle { v0, v1, v2 });
            }
        }
    }
}

fn rtti_name(cs2: &CS2, vtable: u64) -> String {
    let vtable: u64 = cs2.process.read(vtable);
    let rtti: u64 = cs2.process.read(vtable - 0x08);
    let name_ptr: u64 = cs2.process.read(rtti + 0x08);
    cs2.process.read_string(name_ptr)
}

#[repr(C)]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
pub struct UtlVector {
    pub count: i32,
    _pad: i32,
    pub data: u64,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
struct OuterNode {
    pad1: [u8; 12],
    left: i32, // @ 12
    pad2: [u8; 12],
    right: i32, // @ 28
    pad3: [u8; 8],
    shape: u64, // @ 0x28
}

#[repr(C)]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
struct HalfEdge {
    next: u8,
    twin: u8,
    origin: u8,
    face: u8,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
struct Tri {
    idx: [i32; 3],
}
