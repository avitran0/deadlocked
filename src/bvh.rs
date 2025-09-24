use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

use glam::Vec3;
use serde::{Deserialize, Serialize};

const MAX_LEAF_COUNT: usize = 8;

#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    pub fn new() -> Self {
        Self {
            min: Vec3::splat(f32::MAX),
            max: Vec3::splat(f32::MIN),
        }
    }

    pub fn min(&self) -> &Vec3 {
        &self.min
    }

    pub fn max(&self) -> &Vec3 {
        &self.max
    }

    pub fn centroid(&self) -> Vec3 {
        (self.min + self.max) / 2.0
    }

    pub fn from_points(points: &[Vec3]) -> Self {
        let mut aabb = Aabb::new();
        for &p in points {
            aabb.expand(p);
        }
        aabb
    }

    pub fn expand(&mut self, p: Vec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn ray_intersect(&self, origin: Vec3, inv_dir: Vec3, max_t: f32) -> bool {
        let t1 = (self.min - origin) * inv_dir;
        let t2 = (self.max - origin) * inv_dir;

        let tmin = t1.min(t2);
        let tmax = t1.max(t2);

        let t_min = tmin.x.max(tmin.y).max(tmin.z);
        let t_max = tmax.x.min(tmax.y).min(tmax.z);

        t_min <= t_max && t_min <= max_t && t_max >= 0.0
    }
}

#[repr(C)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        Self { v0, v1, v2 }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::from_points(&[self.v0, self.v1, self.v2])
    }

    pub fn centroid(&self) -> Vec3 {
        (self.v0 + self.v1 + self.v2) * (1.0 / 3.0)
    }

    pub fn ray_intersect(&self, origin: Vec3, dir: Vec3) -> Option<(f32, f32, f32)> {
        const EPSILON: f32 = 1e-6;
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        let h = dir.cross(edge2);
        let a = edge1.dot(h);

        if a > -EPSILON && a < EPSILON {
            return None;
        }

        let f = 1.0 / a;
        let s = origin - self.v0;
        let u = f * s.dot(h);

        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * dir.dot(q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(q);
        if t > EPSILON { Some((t, u, v)) } else { None }
    }
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
enum BvhNode {
    Branch {
        left: usize,
        right: usize,
        aabb: Aabb,
    },
    Leaf {
        primitives: Vec<usize>,
        aabb: Aabb,
    },
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Bvh {
    nodes: Vec<BvhNode>,
    triangles: Vec<Triangle>,
    root: Option<usize>,
}

impl Bvh {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            triangles: Vec::new(),
            root: None,
        }
    }

    pub fn save(&self, file: &mut File) {
        let mut writer = BufWriter::new(file);
        bincode::serde::encode_into_std_write(self, &mut writer, bincode::config::standard())
            .unwrap();
    }

    pub fn load(file: &mut File) -> Option<Self> {
        let mut reader = BufReader::new(file);
        bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard()).ok()
    }

    pub fn insert(&mut self, triangle: Triangle) -> usize {
        let idx = self.triangles.len();
        self.triangles.push(triangle);
        idx
    }

    #[allow(unused)]
    pub fn triangles(&self, position: &Vec3) -> Vec<&Triangle> {
        self.triangles
            .iter()
            .filter(|tri| (tri.centroid() - position).length() < 1000.0)
            .collect()
    }

    #[allow(unused)]
    pub fn aabbs(&self, position: &Vec3) -> Vec<&Aabb> {
        self.nodes
            .iter()
            .filter_map(|node| {
                let aabb = node.aabb();
                if (aabb.centroid() - position).length() < 1000.0 {
                    Some(aabb)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn build(&mut self) {
        if self.triangles.is_empty() {
            self.root = None;
            return;
        }

        let mut primitives: Vec<usize> = (0..self.triangles.len()).collect();
        self.nodes.clear();
        self.root = Some(self.build_recursive(&mut primitives));
    }

    fn build_recursive(&mut self, primitives: &mut [usize]) -> usize {
        if primitives.len() <= MAX_LEAF_COUNT {
            let aabb = primitives.iter().fold(Aabb::new(), |acc, &idx| {
                acc.merge(&self.triangles[idx].aabb())
            });
            return self.create_leaf(primitives, aabb);
        }

        let centroid_bounds = primitives.iter().fold(Aabb::new(), |mut acc, &idx| {
            acc.expand(self.triangles[idx].centroid());
            acc
        });

        let extent = centroid_bounds.max - centroid_bounds.min;
        let axis = if extent.x > extent.y && extent.x > extent.z {
            0
        } else if extent.y > extent.z {
            1
        } else {
            2
        };

        primitives.sort_by(|&a_idx, &b_idx| {
            let a_cent = self.triangles[a_idx].centroid();
            let b_cent = self.triangles[b_idx].centroid();
            a_cent[axis].partial_cmp(&b_cent[axis]).unwrap()
        });

        let mid = primitives.len() / 2;
        let (left_prims, right_prims) = primitives.split_at_mut(mid);

        let left = self.build_recursive(left_prims);
        let right = self.build_recursive(right_prims);

        let left_aabb = self.nodes[left].aabb();
        let right_aabb = self.nodes[right].aabb();
        let aabb = left_aabb.merge(right_aabb);

        self.create_internal(left, right, aabb)
    }

    fn create_leaf(&mut self, primitives: &[usize], aabb: Aabb) -> usize {
        let node = BvhNode::Leaf {
            primitives: primitives.to_vec(),
            aabb,
        };
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    fn create_internal(&mut self, left: usize, right: usize, aabb: Aabb) -> usize {
        let node = BvhNode::Branch { left, right, aabb };
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    pub fn has_line_of_sight(&self, start: Vec3, end: Vec3) -> bool {
        let dir = end - start;
        let distance = dir.length();

        let dir_norm = dir / distance;
        let inv_dir = 1.0 / dir_norm;

        if let Some(root) = self.root {
            !self.segment_intersect_node(root, start, dir_norm, inv_dir, distance)
        } else {
            true
        }
    }

    fn segment_intersect_node(
        &self,
        node_idx: usize,
        origin: Vec3,
        direction: Vec3,
        inv_dir: Vec3,
        max_t: f32,
    ) -> bool {
        let node = &self.nodes[node_idx];

        if !node.aabb().ray_intersect(origin, inv_dir, max_t) {
            return false;
        }

        match node {
            BvhNode::Leaf { primitives, .. } => {
                for &idx in primitives {
                    if let Some((t, _, _)) = self.triangles[idx].ray_intersect(origin, direction)
                        && t >= 0.0
                        && t <= max_t
                    {
                        return true;
                    }
                }
                false
            }
            BvhNode::Branch { left, right, .. } => {
                self.segment_intersect_node(*left, origin, direction, inv_dir, max_t)
                    || self.segment_intersect_node(*right, origin, direction, inv_dir, max_t)
            }
        }
    }
}

impl BvhNode {
    fn aabb(&self) -> &Aabb {
        match self {
            BvhNode::Branch { aabb, .. } => aabb,
            BvhNode::Leaf { aabb, .. } => aabb,
        }
    }
}
