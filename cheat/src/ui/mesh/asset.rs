use std::io;

use glam::Mat4;

// binary format produced by crate::mesh_extract, one shared joint table per
// file since every submesh binds to the same skeleton
#[allow(dead_code)]
pub struct Submesh {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub joint_indices: Vec<[u16; 4]>,
    pub joint_weights: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

pub struct MeshAsset {
    // joints are matched by index, not name, see Player::skeleton_transforms
    #[allow(dead_code)]
    pub joint_names: Vec<String>,
    pub inverse_bind: Vec<Mat4>,
    // -1 = no parent within this skeleton
    pub parent_indices: Vec<i32>,
    pub submeshes: Vec<Submesh>,
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn bytes(&mut self, len: usize) -> io::Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(len)
            .filter(|&end| end <= self.data.len())
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "dlms truncated"))?;
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }

    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.bytes(2)?.try_into().unwrap()))
    }

    fn f32(&mut self) -> io::Result<f32> {
        Ok(f32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }

    fn string(&mut self) -> io::Result<String> {
        let len = self.u32()? as usize;
        let bytes = self.bytes(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn vec3(&mut self) -> io::Result<[f32; 3]> {
        Ok([self.f32()?, self.f32()?, self.f32()?])
    }

    fn vec2(&mut self) -> io::Result<[f32; 2]> {
        Ok([self.f32()?, self.f32()?])
    }

    fn joint4(&mut self) -> io::Result<[u16; 4]> {
        Ok([self.u16()?, self.u16()?, self.u16()?, self.u16()?])
    }

    fn weight4(&mut self) -> io::Result<[f32; 4]> {
        Ok([self.f32()?, self.f32()?, self.f32()?, self.f32()?])
    }

    fn mat4(&mut self) -> io::Result<Mat4> {
        let mut cols = [0.0f32; 16];
        for value in &mut cols {
            *value = self.f32()?;
        }
        Ok(Mat4::from_cols_array(&cols))
    }
}

impl MeshAsset {
    pub fn load(data: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(data);

        let magic = cursor.bytes(4)?;
        if magic != b"DLMS" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "bad dlms magic"));
        }
        let version = cursor.u32()?;
        if version != 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported dlms version {version}"),
            ));
        }

        let joint_count = cursor.u32()? as usize;
        let mut joint_names = Vec::with_capacity(joint_count);
        let mut inverse_bind = Vec::with_capacity(joint_count);
        for _ in 0..joint_count {
            joint_names.push(cursor.string()?);
            inverse_bind.push(cursor.mat4()?);
        }
        let mut parent_indices = Vec::with_capacity(joint_count);
        for _ in 0..joint_count {
            parent_indices.push(cursor.i32()?);
        }

        let submesh_count = cursor.u32()? as usize;
        let mut submeshes = Vec::with_capacity(submesh_count);
        for _ in 0..submesh_count {
            let name = cursor.string()?;
            let vertex_count = cursor.u32()? as usize;
            let index_count = cursor.u32()? as usize;

            let mut positions = Vec::with_capacity(vertex_count);
            for _ in 0..vertex_count {
                positions.push(cursor.vec3()?);
            }
            let mut normals = Vec::with_capacity(vertex_count);
            for _ in 0..vertex_count {
                normals.push(cursor.vec3()?);
            }
            let mut uvs = Vec::with_capacity(vertex_count);
            for _ in 0..vertex_count {
                uvs.push(cursor.vec2()?);
            }
            let mut joint_indices = Vec::with_capacity(vertex_count);
            for _ in 0..vertex_count {
                joint_indices.push(cursor.joint4()?);
            }
            let mut joint_weights = Vec::with_capacity(vertex_count);
            for _ in 0..vertex_count {
                joint_weights.push(cursor.weight4()?);
            }
            let mut indices = Vec::with_capacity(index_count);
            for _ in 0..index_count {
                indices.push(cursor.u32()?);
            }

            submeshes.push(Submesh {
                name,
                positions,
                normals,
                uvs,
                joint_indices,
                joint_weights,
                indices,
            });
        }

        Ok(Self {
            joint_names,
            inverse_bind,
            parent_indices,
            submeshes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// hand-built DLMS v4 buffer: 2 joints, 1 triangle
    fn synthetic_dlms() -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"DLMS");
        out.extend_from_slice(&4u32.to_le_bytes());

        let joints = ["root", "child"];
        out.extend_from_slice(&(joints.len() as u32).to_le_bytes());
        for name in joints {
            out.extend_from_slice(&(name.len() as u32).to_le_bytes());
            out.extend_from_slice(name.as_bytes());
            // identity matrix
            for i in 0..16 {
                let v: f32 = if i % 5 == 0 { 1.0 } else { 0.0 };
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
        // root has no parent, child's parent is root
        for parent in [-1i32, 0] {
            out.extend_from_slice(&parent.to_le_bytes());
        }

        out.extend_from_slice(&1u32.to_le_bytes()); // submesh_count
        let name = "test_submesh";
        out.extend_from_slice(&(name.len() as u32).to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&3u32.to_le_bytes()); // vertex_count
        out.extend_from_slice(&3u32.to_le_bytes()); // index_count

        let positions = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        for p in positions {
            for v in p {
                out.extend_from_slice(&(v as f32).to_le_bytes());
            }
        }
        for _ in 0..3 {
            out.extend_from_slice(&0.0f32.to_le_bytes());
            out.extend_from_slice(&0.0f32.to_le_bytes());
            out.extend_from_slice(&1.0f32.to_le_bytes());
        }
        for _ in 0..3 {
            out.extend_from_slice(&0.0f32.to_le_bytes());
            out.extend_from_slice(&0.0f32.to_le_bytes());
        }
        for _ in 0..3 {
            for joint in [0u16, 1, 0, 0] {
                out.extend_from_slice(&joint.to_le_bytes());
            }
        }
        for _ in 0..3 {
            for weight in [0.5f32, 0.5, 0.0, 0.0] {
                out.extend_from_slice(&weight.to_le_bytes());
            }
        }
        for i in [0u32, 1, 2] {
            out.extend_from_slice(&i.to_le_bytes());
        }

        out
    }

    #[test]
    fn parses_synthetic_mesh() {
        let data = synthetic_dlms();
        let asset = MeshAsset::load(&data).expect("failed to parse dlms");
        assert_eq!(asset.joint_names, vec!["root", "child"]);
        assert_eq!(asset.inverse_bind.len(), 2);
        assert_eq!(asset.parent_indices, vec![-1, 0]);
        assert_eq!(asset.submeshes.len(), 1);

        let body = &asset.submeshes[0];
        assert_eq!(body.positions.len(), body.normals.len());
        assert_eq!(body.positions.len(), body.uvs.len());
        assert_eq!(body.positions.len(), body.joint_indices.len());
        assert_eq!(body.positions.len(), body.joint_weights.len());
        assert!(
            body.indices
                .iter()
                .all(|&i| (i as usize) < body.positions.len())
        );
        for weights in &body.joint_weights {
            let sum: f32 = weights.iter().sum();
            assert!(
                (sum - 1.0).abs() < 0.01,
                "weights should sum to ~1.0, got {sum}"
            );
        }
        for indices in &body.joint_indices {
            for &j in indices {
                assert!((j as usize) < asset.joint_names.len());
            }
        }
    }
}
