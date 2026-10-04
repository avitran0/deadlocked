use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use glam::Vec3;

use crate::{cs2::CS2, os::process::Process};

const MIN_MODEL_POINTER: usize = 0x1_0000;
const MAX_MODEL_POINTER: usize = 1 << 47;

// CModel and render-mesh fields (these aren't in the client schema).
const CMODEL_MESHES: usize = 0x78;
const MESH_HITBOX_DATA: usize = 0x168;
const HITBOX_COUNT: usize = 0x28;
const HITBOX_ARRAY: usize = 0x30;
const REMAP_COUNT: usize = 0x220;
const REMAP_TABLE: usize = 0x228;
const MESH_A: usize = 0x240;
const MESH_B: usize = 0x2f0;
const HITBOX_STRIDE: usize = 0x70;
const MAX_HITBOXES: usize = 20;
const MAX_BONE_REMAPS: usize = 512;

#[derive(Clone, Copy)]
pub struct HitboxDefinition {
    pub bone_index: usize,
    pub min: Vec3,
    pub max: Vec3,
    pub radius: f32,
    pub group_id: u8,
    pub shape: u8,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct HitboxCacheKey {
    cmodel: usize,
    hitbox_set: usize,
}

type HitboxCache = HashMap<HitboxCacheKey, Vec<HitboxDefinition>>;

static HITBOX_CACHE: OnceLock<Mutex<HitboxCache>> = OnceLock::new();

pub fn read_model_hitboxes(
    process: &Process,
    cmodel: usize,
    hitbox_set: usize,
) -> Option<Vec<HitboxDefinition>> {
    if hitbox_set != 0 || !is_model_pointer(cmodel) {
        return None;
    }

    let cache = HITBOX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = HitboxCacheKey { cmodel, hitbox_set };
    if let Some(hitboxes) = cache.lock().ok()?.get(&key) {
        return Some(hitboxes.clone());
    }

    let hitboxes = extract_model_hitboxes(process, cmodel)?;
    cache.lock().ok()?.insert(key, hitboxes.clone());
    Some(hitboxes)
}

fn extract_model_hitboxes(process: &Process, cmodel: usize) -> Option<Vec<HitboxDefinition>> {
    let render_mesh_list: usize = process.read(cmodel + CMODEL_MESHES);
    if !is_model_pointer(render_mesh_list) {
        return None;
    }
    let render_meshes: usize = process.read(render_mesh_list);
    if !is_model_pointer(render_meshes) {
        return None;
    }
    let hitbox_data: usize = process.read(render_meshes + MESH_HITBOX_DATA);
    if !is_model_pointer(hitbox_data) {
        return None;
    }

    let count = process.read::<i32>(hitbox_data + HITBOX_COUNT);
    if !(1..=MAX_HITBOXES as i32).contains(&count) {
        return None;
    }
    let records: usize = process.read(hitbox_data + HITBOX_ARRAY);
    if !is_model_pointer(records) {
        return None;
    }

    let remap_count = process.read::<i32>(cmodel + REMAP_COUNT);
    if !(1..=MAX_BONE_REMAPS as i32).contains(&remap_count) {
        return None;
    }
    let remap_table: usize = process.read(cmodel + REMAP_TABLE);
    let mesh_a: usize = process.read(cmodel + MESH_A);
    let mesh_b: usize = process.read(cmodel + MESH_B);
    if !is_model_pointer(remap_table) || !is_model_pointer(mesh_a) || !is_model_pointer(mesh_b) {
        return None;
    }

    let offset_a = process.read::<u16>(mesh_a) as usize;
    let offset_b = process.read::<u16>(mesh_b) as usize;
    let remap = process.read_typed_vec::<i16>(remap_table, size_of::<i16>(), remap_count as usize);

    let mut hitboxes = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let record = records + index * HITBOX_STRIDE;
        let hitbox_remap_index = process.read::<u16>(record + 0x48) as usize;
        let remap_index = hitbox_remap_index + offset_a + offset_b;
        let Some(&bone_index) = remap.get(remap_index) else {
            continue;
        };
        if !(0..96).contains(&bone_index) {
            continue;
        }

        let min: Vec3 = process.read(record + 0x18);
        let max: Vec3 = process.read(record + 0x24);
        let radius = process.read::<f32>(record + 0x30);
        let group_id = process.read::<u32>(record + 0x38);
        let shape = process.read::<u32>(record + 0x3c);
        if !min.is_finite()
            || !max.is_finite()
            || !radius.is_finite()
            || !(0.0..=128.0).contains(&radius)
            || group_id > u8::MAX as u32
            || shape > 2
        {
            continue;
        }

        hitboxes.push(HitboxDefinition {
            bone_index: bone_index as usize,
            min,
            max,
            radius,
            group_id: group_id as u8,
            shape: shape as u8,
        });
    }

    hitboxes
        .iter()
        .any(|hitbox| hitbox.group_id == 1)
        .then_some(hitboxes)
}

fn is_model_pointer(pointer: usize) -> bool {
    (MIN_MODEL_POINTER..MAX_MODEL_POINTER).contains(&pointer)
}

pub fn hitbox_set_index(cs2: &CS2, scene_node: usize) -> usize {
    cs2.process
        .read::<u8>(scene_node + cs2.offsets.model_data.hitbox_set) as usize
}
