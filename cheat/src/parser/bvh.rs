use glam::Vec3;
use serde::{Deserialize, Serialize};

const MAX_LEAF_COUNT: usize = 8;
const MAX_PENETRATION_DISTANCE: f32 = 3000.0;
const MIN_PENETRATION_MODIFIER: f32 = 0.1;
const DEFAULT_DAMAGE_LOSS_MODIFIER: f32 = 0.16;
const THIN_GLASS_DAMAGE_LOSS_MODIFIER: f32 = 0.05;
const THIN_GLASS_MAX_THICKNESS: f32 = 6.0;
const THICKNESS_LOSS_DIVISOR: f32 = 24.0;

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

    #[allow(unused)]
    pub fn min(&self) -> &Vec3 {
        &self.min
    }

    #[allow(unused)]
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

    pub fn ray_intersect(&self, origin: Vec3, direction: Vec3, max_t: f32) -> bool {
        let mut near = 0.0f32;
        let mut far = max_t;
        for axis in 0..3 {
            let component = direction[axis];
            if component == 0.0 {
                if origin[axis] < self.min[axis] || origin[axis] > self.max[axis] {
                    return false;
                }
                continue;
            }

            let first = (self.min[axis] - origin[axis]) / component;
            let second = (self.max[axis] - origin[axis]) / component;
            near = near.max(first.min(second));
            far = far.min(first.max(second));
            if near > far {
                return false;
            }
        }
        far >= 0.0 && near <= max_t
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SurfaceMaterial {
    pub penetration_modifier: f32,
    pub damage_modifier: f32,
    pub surface_type: u16,
}

impl Default for SurfaceMaterial {
    fn default() -> Self {
        Self {
            penetration_modifier: 0.0,
            damage_modifier: 0.0,
            surface_type: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub material: usize,
    pub shape: usize,
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, material: usize, shape: usize) -> Self {
        Self {
            v0,
            v1,
            v2,
            material,
            shape,
        }
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
#[derive(Serialize, Deserialize)]
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

#[derive(Clone, Copy)]
struct MaterialHit {
    distance: f32,
    material: SurfaceMaterial,
    shape: usize,
}

fn append_shape_walls(crossings: &[MaterialHit], walls: &mut Vec<[MaterialHit; 2]>) -> bool {
    let (pairs, remainder) = crossings.as_chunks::<2>();
    if !remainder.is_empty() {
        return false;
    }
    walls.extend_from_slice(pairs);
    true
}

#[repr(C)]
#[derive(Serialize, Deserialize)]
pub struct Bvh {
    nodes: Vec<BvhNode>,
    triangles: Vec<Triangle>,
    materials: Vec<SurfaceMaterial>,
    root: Option<usize>,
}

impl Bvh {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            triangles: Vec::new(),
            materials: Vec::new(),
            root: None,
        }
    }

    pub fn set(&mut self, triangles: Vec<Triangle>, materials: Vec<SurfaceMaterial>) {
        self.triangles = triangles;
        self.materials = materials;
    }

    pub fn all_triangles(&self) -> &[Triangle] {
        &self.triangles
    }

    pub fn surface_materials(&self) -> &[SurfaceMaterial] {
        &self.materials
    }

    pub fn material(&self, triangle: &Triangle) -> Option<&SurfaceMaterial> {
        self.materials.get(triangle.material)
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
        let ray = end - start;
        let distance = ray.length();
        if !distance.is_finite() || distance <= f32::EPSILON {
            return true;
        }
        let direction = ray / distance;
        self.root
            .is_none_or(|root| !self.segment_intersect_node(root, start, direction, distance))
    }

    pub fn estimate_penetration_damage(
        &self,
        start: Vec3,
        end: Vec3,
        weapon_damage: f32,
        weapon_penetration: f32,
        weapon_range: f32,
        range_modifier: f32,
    ) -> Option<f32> {
        let ray = end - start;
        let distance = ray.length();
        if !distance.is_finite() || distance <= f32::EPSILON || distance > weapon_range {
            return None;
        }
        let root = self.root?;
        let direction = ray / distance;
        let mut hits = Vec::new();
        self.collect_material_hits(root, start, direction, distance, &mut hits);
        hits.sort_by(|a, b| {
            a.shape
                .cmp(&b.shape)
                .then_with(|| a.distance.total_cmp(&b.distance))
        });

        let mut walls = Vec::<[MaterialHit; 2]>::new();
        let mut shape_crossings = Vec::<MaterialHit>::new();
        let mut current_shape = None;
        for hit in hits {
            if current_shape.is_some_and(|shape| shape != hit.shape) {
                if !append_shape_walls(&shape_crossings, &mut walls) {
                    return None;
                }
                shape_crossings.clear();
            }
            current_shape = Some(hit.shape);

            if let Some(last) = shape_crossings.last_mut()
                && (last.distance - hit.distance).abs() <= 0.05
            {
                last.material.penetration_modifier = last
                    .material
                    .penetration_modifier
                    .min(hit.material.penetration_modifier);
                last.material.damage_modifier = last
                    .material
                    .damage_modifier
                    .max(hit.material.damage_modifier);
                if last.material.surface_type != hit.material.surface_type {
                    last.material.surface_type = 0;
                }
            } else {
                shape_crossings.push(hit);
            }
        }
        if !append_shape_walls(&shape_crossings, &mut walls) {
            return None;
        }
        walls.sort_by(|a, b| a[0].distance.total_cmp(&b[0].distance));

        if weapon_damage <= 0.0 || weapon_penetration <= 0.0 || range_modifier <= 0.0 {
            return None;
        }
        if walls.is_empty() && !self.has_line_of_sight(start, end) {
            return None;
        }

        let mut damage = weapon_damage;
        let mut previous_distance = 0.0;
        for (wall_index, wall) in walls.iter().enumerate() {
            if wall_index >= 4 {
                return None;
            }
            let entry = wall[0];
            let exit = wall[1];
            let thickness = exit.distance - entry.distance;
            if entry.distance > MAX_PENETRATION_DISTANCE {
                return None;
            }

            let surface_type = entry.material.surface_type;
            let same_surface_type = surface_type == exit.material.surface_type;
            let mut penetration_modifier = entry
                .material
                .penetration_modifier
                .min(exit.material.penetration_modifier);
            let mut damage_loss_modifier = DEFAULT_DAMAGE_LOSS_MODIFIER;
            if same_surface_type {
                match surface_type as u8 {
                    b'U' | b'W' => penetration_modifier = 3.0,
                    b'L' => penetration_modifier = 2.0,
                    b'G' | b'Y' if thickness < THIN_GLASS_MAX_THICKNESS => {
                        penetration_modifier = 3.0;
                        damage_loss_modifier = THIN_GLASS_DAMAGE_LOSS_MODIFIER;
                    }
                    _ => {}
                }
            }
            if penetration_modifier < MIN_PENETRATION_MODIFIER || !penetration_modifier.is_finite()
            {
                return None;
            }

            damage *= range_modifier.powf((entry.distance - previous_distance) / 500.0);
            let inverse_penetration_modifier = 1.0 / penetration_modifier;
            let penetration_loss =
                (3.0 / weapon_penetration * 1.25) * (inverse_penetration_modifier * 3.0);
            let thickness_loss =
                thickness * thickness * inverse_penetration_modifier / THICKNESS_LOSS_DIVISOR;
            let damage_loss =
                (damage * damage_loss_modifier + penetration_loss + thickness_loss).max(0.0);
            damage -= damage_loss;
            if damage < 1.0 {
                return None;
            }
            damage *= range_modifier.powf(thickness / 500.0);
            previous_distance = exit.distance;
        }

        damage *= range_modifier.powf((distance - previous_distance) / 500.0);
        (damage > 0.0).then_some(damage)
    }

    fn collect_material_hits(
        &self,
        node_idx: usize,
        origin: Vec3,
        direction: Vec3,
        max_t: f32,
        hits: &mut Vec<MaterialHit>,
    ) {
        let node = &self.nodes[node_idx];
        if !node.aabb().ray_intersect(origin, direction, max_t) {
            return;
        }
        match node {
            BvhNode::Leaf { primitives, .. } => {
                for &index in primitives {
                    let triangle = &self.triangles[index];
                    if let Some((distance, _, _)) = triangle.ray_intersect(origin, direction)
                        && distance <= max_t
                    {
                        hits.push(MaterialHit {
                            distance,
                            material: self
                                .materials
                                .get(triangle.material)
                                .copied()
                                .unwrap_or_default(),
                            shape: triangle.shape,
                        });
                    }
                }
            }
            BvhNode::Branch { left, right, .. } => {
                self.collect_material_hits(*left, origin, direction, max_t, hits);
                self.collect_material_hits(*right, origin, direction, max_t, hits);
            }
        }
    }

    fn segment_intersect_node(
        &self,
        node_idx: usize,
        origin: Vec3,
        direction: Vec3,
        max_t: f32,
    ) -> bool {
        let node = &self.nodes[node_idx];
        if !node.aabb().ray_intersect(origin, direction, max_t) {
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
                self.segment_intersect_node(*left, origin, direction, max_t)
                    || self.segment_intersect_node(*right, origin, direction, max_t)
            }
        }
    }

    #[allow(unused)]
    pub fn triangles_near(&self, position: Vec3, radius: f32) -> Vec<&Triangle> {
        let mut result = Vec::new();
        if let Some(root) = self.root {
            self.collect_triangles_near(root, position, radius, &mut result);
        }
        result
    }

    #[allow(unused)]
    pub fn aabbs_near(&self, position: Vec3, radius: f32) -> Vec<&Aabb> {
        let mut result = Vec::new();
        if let Some(root) = self.root {
            self.collect_aabbs_near(root, position, radius, &mut result);
        }
        result
    }

    #[allow(unused)]
    fn collect_triangles_near<'a>(
        &'a self,
        node_idx: usize,
        position: Vec3,
        radius: f32,
        result: &mut Vec<&'a Triangle>,
    ) {
        let node = &self.nodes[node_idx];
        let aabb = node.aabb();

        if !self.sphere_aabb_intersect(position, radius, aabb) {
            return;
        }

        match node {
            BvhNode::Leaf { primitives, .. } => {
                for &idx in primitives {
                    let tri = &self.triangles[idx];
                    if (tri.centroid() - position).length() <= radius {
                        result.push(tri);
                    }
                }
            }
            BvhNode::Branch { left, right, .. } => {
                self.collect_triangles_near(*left, position, radius, result);
                self.collect_triangles_near(*right, position, radius, result);
            }
        }
    }

    #[allow(unused)]
    fn collect_aabbs_near<'a>(
        &'a self,
        node_idx: usize,
        position: Vec3,
        radius: f32,
        result: &mut Vec<&'a Aabb>,
    ) {
        let node = &self.nodes[node_idx];
        let aabb = node.aabb();

        if !self.sphere_aabb_intersect(position, radius, aabb) {
            return;
        }

        if (aabb.centroid() - position).length() <= radius {
            result.push(aabb);
        }

        if let BvhNode::Branch { left, right, .. } = node {
            self.collect_aabbs_near(*left, position, radius, result);
            self.collect_aabbs_near(*right, position, radius, result);
        }
    }

    #[allow(unused)]
    fn sphere_aabb_intersect(&self, sphere_center: Vec3, sphere_radius: f32, aabb: &Aabb) -> bool {
        let closest_point = sphere_center.clamp(aabb.min, aabb.max);
        let distance_sq = (sphere_center - closest_point).length_squared();
        distance_sq <= sphere_radius * sphere_radius
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
