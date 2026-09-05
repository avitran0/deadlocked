use std::io;

use serde_json::Value;

// only submeshes visible on other players are kept; first-person
// arm/sleeve meshes are viewmodel-only
const KEEP_PREFIXES: [&str; 2] = ["thirdperson_body", "thirdperson_default_gloves"];

struct Glb {
    json: Value,
    bin: Vec<u8>,
}

struct RawSubmesh {
    name: String,
    positions: Vec<Vec<f64>>,
    normals: Vec<Vec<f64>>,
    uvs: Vec<Vec<f64>>,
    joints: Vec<Vec<f64>>,
    weights: Vec<Vec<f64>>,
    indices: Vec<u32>,
}

fn parse_glb(data: &[u8]) -> io::Result<Glb> {
    let err = |msg: &str| io::Error::new(io::ErrorKind::InvalidData, msg.to_string());

    if data.len() < 12 || &data[0..4] != b"glTF" {
        return Err(err("not a glb file"));
    }
    let length = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;

    let mut offset = 12;
    let mut json = None;
    let mut bin = None;
    while offset < length {
        let chunk_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        let chunk_type = &data[offset + 4..offset + 8];
        let chunk_start = offset + 8;
        let chunk_data = &data[chunk_start..chunk_start + chunk_len];

        if chunk_type == b"JSON" {
            json = Some(serde_json::from_slice(chunk_data).map_err(|e| err(&e.to_string()))?);
        } else if chunk_type == b"BIN\0" {
            bin = Some(chunk_data.to_vec());
        }
        offset = chunk_start + chunk_len;
    }

    Ok(Glb {
        json: json.ok_or_else(|| err("glb missing JSON chunk"))?,
        bin: bin.unwrap_or_default(),
    })
}

fn component_size(component_type: u64) -> usize {
    match component_type {
        5120 | 5121 => 1,
        5122 | 5123 => 2,
        5125 | 5126 => 4,
        _ => panic!("unknown accessor component type {component_type}"),
    }
}

fn read_component(bin: &[u8], offset: usize, component_type: u64) -> f64 {
    match component_type {
        5120 => bin[offset] as i8 as f64,
        5121 => bin[offset] as f64,
        5122 => i16::from_le_bytes(bin[offset..offset + 2].try_into().unwrap()) as f64,
        5123 => u16::from_le_bytes(bin[offset..offset + 2].try_into().unwrap()) as f64,
        5125 => u32::from_le_bytes(bin[offset..offset + 4].try_into().unwrap()) as f64,
        5126 => f32::from_le_bytes(bin[offset..offset + 4].try_into().unwrap()) as f64,
        _ => panic!("unknown accessor component type {component_type}"),
    }
}

fn type_component_count(type_: &str) -> usize {
    match type_ {
        "SCALAR" => 1,
        "VEC2" => 2,
        "VEC3" => 3,
        "VEC4" => 4,
        "MAT4" => 16,
        other => panic!("unknown accessor type {other}"),
    }
}

/// decodes any accessor into per-vertex rows of f64 (a safe superset of every
/// component type gltf uses), the caller casts down to whatever it needs
fn accessor_rows(glb: &Glb, accessor_idx: usize) -> Vec<Vec<f64>> {
    let accessors = glb.json["accessors"].as_array().unwrap();
    let acc = &accessors[accessor_idx];
    let buffer_views = glb.json["bufferViews"].as_array().unwrap();
    let bv = &buffer_views[acc["bufferView"].as_u64().unwrap() as usize];

    let bv_offset = bv["byteOffset"].as_u64().unwrap_or(0) as usize;
    let acc_offset = acc["byteOffset"].as_u64().unwrap_or(0) as usize;
    let component_type = acc["componentType"].as_u64().unwrap();
    let type_ = acc["type"].as_str().unwrap();
    let count = acc["count"].as_u64().unwrap() as usize;

    let n_comp = type_component_count(type_);
    let comp_size = component_size(component_type);
    let stride = bv
        .get("byteStride")
        .and_then(Value::as_u64)
        .map(|s| s as usize)
        .unwrap_or(n_comp * comp_size);
    let base = bv_offset + acc_offset;

    (0..count)
        .map(|i| {
            let row_start = base + i * stride;
            (0..n_comp)
                .map(|c| read_component(&glb.bin, row_start + c * comp_size, component_type))
                .collect()
        })
        .collect()
}

fn node_mesh_skin_map(glb: &Glb) -> std::collections::HashMap<usize, usize> {
    let mut map = std::collections::HashMap::new();
    let Some(nodes) = glb.json["nodes"].as_array() else {
        return map;
    };
    for node in nodes {
        if let (Some(mesh), Some(skin)) = (node.get("mesh"), node.get("skin")) {
            map.insert(
                mesh.as_u64().unwrap() as usize,
                skin.as_u64().unwrap() as usize,
            );
        }
    }
    map
}

fn write_string(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}

/// converts a glb produced by `run_vrf_extraction` (see its exact VRF CLI
/// invocation in the parent module) into the DLMS v3 binary format
/// `ui::mesh::asset::MeshAsset` loads at runtime
pub fn glb_to_dlms(data: &[u8]) -> Result<Vec<u8>, String> {
    let glb = parse_glb(data).map_err(|e| e.to_string())?;
    let mesh_to_skin = node_mesh_skin_map(&glb);

    let meshes = glb.json["meshes"].as_array().ok_or("glb has no meshes")?;
    let nodes = glb.json["nodes"].as_array().ok_or("glb has no nodes")?;
    let skins = glb.json["skins"].as_array().ok_or("glb has no skins")?;

    let mut joint_names: Option<Vec<String>> = None;
    let mut inverse_bind: Vec<[f32; 16]> = Vec::new();
    let mut submeshes: Vec<RawSubmesh> = Vec::new();

    for (mesh_idx, mesh) in meshes.iter().enumerate() {
        let mesh_name = mesh["name"].as_str().unwrap_or("unnamed").to_string();
        if !KEEP_PREFIXES.iter().any(|p| mesh_name.contains(p)) {
            continue;
        }
        let Some(&skin_idx) = mesh_to_skin.get(&mesh_idx) else {
            continue;
        };
        let skin = &skins[skin_idx];
        let joint_nodes: Vec<usize> = skin["joints"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();
        let this_names: Vec<String> = joint_nodes
            .iter()
            .map(|&n| {
                nodes[n]["name"]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("joint_{n}"))
            })
            .collect();

        let this_inverse_bind: Vec<[f32; 16]> = if let Some(idx) =
            skin["inverseBindMatrices"].as_u64()
        {
            accessor_rows(&glb, idx as usize)
                .into_iter()
                .map(|row| {
                    let mut mat = [0.0f32; 16];
                    for (i, v) in row.into_iter().enumerate() {
                        mat[i] = v as f32;
                    }
                    mat
                })
                .collect()
        } else {
            vec![
                [
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                ];
                joint_nodes.len()
            ]
        };

        match &joint_names {
            None => {
                joint_names = Some(this_names);
                inverse_bind = this_inverse_bind;
            }
            Some(names) if *names != this_names => {
                return Err(format!(
                    "joint order mismatch in {mesh_name}: submeshes do not share one skeleton as assumed"
                ));
            }
            _ => {}
        }

        let primitives = mesh["primitives"]
            .as_array()
            .ok_or("mesh has no primitives")?;
        for prim in primitives {
            let attrs = &prim["attributes"];
            let position_idx = attrs["POSITION"]
                .as_u64()
                .ok_or("primitive missing POSITION")? as usize;
            let positions = accessor_rows(&glb, position_idx);
            let vertex_count = positions.len();

            let normals = attrs
                .get("NORMAL")
                .and_then(Value::as_u64)
                .map(|i| accessor_rows(&glb, i as usize))
                .unwrap_or_else(|| vec![vec![0.0, 0.0, 1.0]; vertex_count]);
            let uvs = attrs
                .get("TEXCOORD_0")
                .and_then(Value::as_u64)
                .map(|i| accessor_rows(&glb, i as usize))
                .unwrap_or_else(|| vec![vec![0.0, 0.0]; vertex_count]);
            let joints = attrs
                .get("JOINTS_0")
                .and_then(Value::as_u64)
                .map(|i| accessor_rows(&glb, i as usize))
                .unwrap_or_else(|| vec![vec![0.0, 0.0, 0.0, 0.0]; vertex_count]);
            let weights = attrs
                .get("WEIGHTS_0")
                .and_then(Value::as_u64)
                .map(|i| accessor_rows(&glb, i as usize))
                .unwrap_or_else(|| vec![vec![1.0, 0.0, 0.0, 0.0]; vertex_count]);
            let indices: Vec<u32> = match prim.get("indices").and_then(Value::as_u64) {
                Some(i) => accessor_rows(&glb, i as usize)
                    .into_iter()
                    .map(|row| row[0] as u32)
                    .collect(),
                None => (0..vertex_count as u32).collect(),
            };

            submeshes.push(RawSubmesh {
                name: mesh_name.clone(),
                positions,
                normals,
                uvs,
                joints,
                weights,
                indices,
            });
        }
    }

    let joint_names =
        joint_names.ok_or("no thirdperson_body/gloves submeshes with a skin found")?;
    if submeshes.is_empty() {
        return Err("no thirdperson_body/gloves submeshes found".to_string());
    }

    let mut out = Vec::new();
    out.extend_from_slice(b"DLMS");
    out.extend_from_slice(&3u32.to_le_bytes());
    out.extend_from_slice(&(joint_names.len() as u32).to_le_bytes());
    for (name, mat) in joint_names.iter().zip(&inverse_bind) {
        write_string(&mut out, name);
        for value in mat {
            out.extend_from_slice(&value.to_le_bytes());
        }
    }

    out.extend_from_slice(&(submeshes.len() as u32).to_le_bytes());
    for submesh in &submeshes {
        let RawSubmesh {
            name,
            positions,
            normals,
            uvs,
            joints,
            weights,
            indices,
        } = submesh;
        write_string(&mut out, name);
        out.extend_from_slice(&(positions.len() as u32).to_le_bytes());
        out.extend_from_slice(&(indices.len() as u32).to_le_bytes());
        for row in positions {
            for &v in &row[..3] {
                out.extend_from_slice(&(v as f32).to_le_bytes());
            }
        }
        for row in normals {
            for &v in &row[..3] {
                out.extend_from_slice(&(v as f32).to_le_bytes());
            }
        }
        for row in uvs {
            for &v in &row[..2] {
                out.extend_from_slice(&(v as f32).to_le_bytes());
            }
        }
        for row in joints {
            for &v in &row[..4] {
                out.extend_from_slice(&(v as u16).to_le_bytes());
            }
        }
        for row in weights {
            for &v in &row[..4] {
                out.extend_from_slice(&(v as f32).to_le_bytes());
            }
        }
        for &i in indices {
            out.extend_from_slice(&i.to_le_bytes());
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::ui::mesh::asset::MeshAsset;

    /// builds a minimal, synthetic (not real game data) .glb: a 1-triangle
    /// "thirdperson_body_test" mesh skinned to a 2-joint skeleton, laid out
    /// the same way VRF's real exports are (mesh node with mesh+skin,
    /// separate joint nodes), so glb_to_dlms can be tested without needing
    /// any real CS2 asset file
    fn synthetic_glb() -> Vec<u8> {
        let mut bin = Vec::new();
        let push_f32 = |bin: &mut Vec<u8>, values: &[f32]| -> (usize, usize) {
            let start = bin.len();
            for v in values {
                bin.extend_from_slice(&v.to_le_bytes());
            }
            (start, bin.len() - start)
        };

        let (positions_off, positions_len) =
            push_f32(&mut bin, &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
        let (normals_off, normals_len) =
            push_f32(&mut bin, &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]);
        let (uvs_off, uvs_len) = push_f32(&mut bin, &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0]);
        let joints_off = bin.len();
        for joint in [0u16, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0] {
            bin.extend_from_slice(&joint.to_le_bytes());
        }
        let joints_len = bin.len() - joints_off;
        let (weights_off, weights_len) = push_f32(
            &mut bin,
            &[0.5, 0.5, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0],
        );
        let indices_off = bin.len();
        for i in [0u32, 1, 2] {
            bin.extend_from_slice(&i.to_le_bytes());
        }
        let indices_len = bin.len() - indices_off;
        let identity: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let (inv_bind_off, inv_bind_len) = {
            let mut both = identity.to_vec();
            both.extend_from_slice(&identity);
            push_f32(&mut bin, &both)
        };

        let json = json!({
            "asset": { "version": "2.0" },
            "buffers": [{ "byteLength": bin.len() }],
            "bufferViews": [
                { "buffer": 0, "byteOffset": positions_off, "byteLength": positions_len },
                { "buffer": 0, "byteOffset": normals_off, "byteLength": normals_len },
                { "buffer": 0, "byteOffset": uvs_off, "byteLength": uvs_len },
                { "buffer": 0, "byteOffset": joints_off, "byteLength": joints_len },
                { "buffer": 0, "byteOffset": weights_off, "byteLength": weights_len },
                { "buffer": 0, "byteOffset": indices_off, "byteLength": indices_len },
                { "buffer": 0, "byteOffset": inv_bind_off, "byteLength": inv_bind_len },
            ],
            "accessors": [
                { "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3" },
                { "bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3" },
                { "bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC2" },
                { "bufferView": 3, "componentType": 5123, "count": 3, "type": "VEC4" },
                { "bufferView": 4, "componentType": 5126, "count": 3, "type": "VEC4" },
                { "bufferView": 5, "componentType": 5125, "count": 3, "type": "SCALAR" },
                { "bufferView": 6, "componentType": 5126, "count": 2, "type": "MAT4" },
            ],
            "meshes": [{
                "name": "agents/models/test/test.vmdl_c.thirdperson_body_test",
                "primitives": [{
                    "attributes": {
                        "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2,
                        "JOINTS_0": 3, "WEIGHTS_0": 4,
                    },
                    "indices": 5,
                }],
            }],
            "skins": [{ "joints": [1, 2], "inverseBindMatrices": 6 }],
            "nodes": [
                { "mesh": 0, "skin": 0 },
                { "name": "root" },
                { "name": "child" },
            ],
        });
        let json_bytes = serde_json::to_vec(&json).unwrap();

        let mut glb = Vec::new();
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2u32.to_le_bytes());
        let total_len = 12 + 8 + json_bytes.len() + 8 + bin.len();
        glb.extend_from_slice(&(total_len as u32).to_le_bytes());

        glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend_from_slice(&json_bytes);

        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(b"BIN\0");
        glb.extend_from_slice(&bin);

        glb
    }

    #[test]
    fn converts_synthetic_glb_and_round_trips_through_the_loader() {
        let glb = synthetic_glb();
        let dlms = glb_to_dlms(&glb).expect("conversion failed");

        let asset = MeshAsset::load(&dlms).expect("failed to parse own dlms output");
        assert_eq!(asset.joint_names, vec!["root", "child"]);
        assert_eq!(asset.inverse_bind.len(), 2);
        assert_eq!(asset.submeshes.len(), 1);

        let body = &asset.submeshes[0];
        assert_eq!(body.positions.len(), 3);
        assert_eq!(body.indices, vec![0, 1, 2]);
        assert_eq!(body.joint_indices[0], [0, 1, 0, 0]);
        assert_eq!(body.joint_weights[0], [0.5, 0.5, 0.0, 0.0]);
    }

    #[test]
    fn rejects_mismatched_joint_order_between_submeshes() {
        // two meshes, each with its own skin, where the second skin lists
        // its joints in reversed order; glb_to_dlms assumes (and checks)
        // that every submesh in one file shares a single skeleton
        let mut bin: Vec<u8> = Vec::new();
        let identity: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let inv_bind_off = bin.len();
        for _ in 0..2 {
            for v in identity {
                bin.extend_from_slice(&v.to_le_bytes());
            }
        }
        let inv_bind_len = bin.len() - inv_bind_off;
        // a single degenerate vertex, just enough for mesh 0's primitive to
        // parse successfully so mesh 1's mismatch check is actually reached
        let position_off = bin.len();
        for v in [0.0f32, 0.0, 0.0] {
            bin.extend_from_slice(&v.to_le_bytes());
        }
        let position_len = bin.len() - position_off;

        let json = json!({
            "asset": { "version": "2.0" },
            "buffers": [{ "byteLength": bin.len() }],
            "bufferViews": [
                { "buffer": 0, "byteOffset": inv_bind_off, "byteLength": inv_bind_len },
                { "buffer": 0, "byteOffset": position_off, "byteLength": position_len },
            ],
            "accessors": [
                { "bufferView": 0, "componentType": 5126, "count": 2, "type": "MAT4" },
                { "bufferView": 1, "componentType": 5126, "count": 1, "type": "VEC3" },
            ],
            "meshes": [
                { "name": "thirdperson_body_a", "primitives": [{ "attributes": { "POSITION": 1 } }] },
                { "name": "thirdperson_body_b", "primitives": [{ "attributes": { "POSITION": 1 } }] },
            ],
            "skins": [
                { "joints": [2, 3], "inverseBindMatrices": 0 },
                { "joints": [3, 2], "inverseBindMatrices": 0 },
            ],
            "nodes": [
                { "mesh": 0, "skin": 0 },
                { "mesh": 1, "skin": 1 },
                { "name": "root" },
                { "name": "child" },
            ],
        });
        let json_bytes = serde_json::to_vec(&json).unwrap();

        let mut glb = Vec::new();
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2u32.to_le_bytes());
        let total_len = 12 + 8 + json_bytes.len() + 8 + bin.len();
        glb.extend_from_slice(&(total_len as u32).to_le_bytes());
        glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend_from_slice(&json_bytes);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(b"BIN\0");
        glb.extend_from_slice(&bin);

        let err = glb_to_dlms(&glb).expect_err("mismatched joint order must be rejected");
        assert!(err.contains("joint order mismatch"), "got: {err}");
    }
}
