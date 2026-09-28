use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Mutex, OnceLock},
};

use glam::Vec3;

use crate::{cs2::CS2, os::process::Process};

const MODEL_OBJECT_BYTES: usize = 0x100;
const MODEL_GRAPH_DEPTH: usize = 6;
const MAX_MODEL_OBJECTS: usize = 12_000;
const MAX_HITBOXES: usize = 64;
const HITBOX_SET_DATA_OFFSET: usize = 0x60;
const HITBOX_SET_COUNT_OFFSET: usize = 0x68;
const HITBOX_RECORD_SIZE: usize = 0x70;

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
    model_address: usize,
    hitbox_set_index: usize,
}

type HitboxCache = HashMap<HitboxCacheKey, Vec<HitboxDefinition>>;

static HITBOX_CACHE: OnceLock<Mutex<HitboxCache>> = OnceLock::new();

pub fn read_model_hitboxes(
    process: &Process,
    model: usize,
    hitbox_set: usize,
) -> Option<Vec<HitboxDefinition>> {
    if hitbox_set != 0 {
        return None;
    }

    let cache = HITBOX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = HitboxCacheKey {
        model_address: model,
        hitbox_set_index: hitbox_set,
    };
    if let Some(hitboxes) = cache.lock().ok()?.get(&key) {
        return Some(hitboxes.clone());
    }

    let hitboxes = extract_model_hitboxes(process, model)?;
    cache.lock().ok()?.insert(key, hitboxes.clone());
    Some(hitboxes)
}

fn extract_model_hitboxes(process: &Process, model: usize) -> Option<Vec<HitboxDefinition>> {
    if model == 0 {
        return None;
    }

    let mut pending = VecDeque::from([(model, 0usize)]);
    let mut visited = HashSet::new();

    while let Some((object, depth)) = pending.pop_front() {
        let object = object & !0xF;
        if depth > MODEL_GRAPH_DEPTH || !visited.insert(object) {
            continue;
        }
        if visited.len() > MAX_MODEL_OBJECTS {
            break;
        }

        if let Some(hitboxes) = read_hitbox_set(process, object) {
            return Some(hitboxes);
        }

        if depth == MODEL_GRAPH_DEPTH {
            continue;
        }
        for pointer in process.read_typed_vec::<usize>(
            object,
            size_of::<usize>(),
            MODEL_OBJECT_BYTES / size_of::<usize>(),
        ) {
            let pointer = pointer & !0xF;
            if pointer != 0 && !visited.contains(&pointer) {
                pending.push_back((pointer, depth + 1));
            }
        }
    }

    None
}

fn read_hitbox_set(process: &Process, object: usize) -> Option<Vec<HitboxDefinition>> {
    let data = process.read::<usize>(object + HITBOX_SET_DATA_OFFSET);
    let count = process.read::<u32>(object + HITBOX_SET_COUNT_OFFSET) as usize;
    if !(10..=MAX_HITBOXES).contains(&count) || data == 0 {
        return None;
    }

    let mut hitboxes = Vec::with_capacity(count);
    for index in 0..count {
        let record = data + index * HITBOX_RECORD_SIZE;
        let _name = read_name(process, process.read(record))?;
        let _surface = read_name(process, process.read(record + 0x08))?;
        let bone_name = read_name(process, process.read(record + 0x10))?;
        let min: Vec3 = process.read(record + 0x18);
        let max: Vec3 = process.read(record + 0x24);
        let radius: f32 = process.read(record + 0x30);
        let group_id = process.read::<u32>(record + 0x38);
        let shape = process.read::<u32>(record + 0x3C);
        let bone_index = bone_index(&bone_name)?;

        if !min.is_finite()
            || !max.is_finite()
            || !radius.is_finite()
            || !(0.0..=128.0).contains(&radius)
            || group_id > u8::MAX as u32
            || shape > 2
        {
            return None;
        }

        hitboxes.push(HitboxDefinition {
            bone_index,
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

fn read_name(process: &Process, pointer: usize) -> Option<String> {
    if pointer == 0 {
        return None;
    }
    let name = process.read_string(pointer);
    valid_name(&name).then_some(name)
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 80
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'.')
}

fn bone_index(name: &str) -> Option<usize> {
    Some(match name {
        "pelvis" => 1,
        "spine_0" => 2,
        "spine_1" => 3,
        "spine_2" => 4,
        "spine_3" => 5,
        "neck_0" => 6,
        "head_0" => 7,
        "clavicle_l" => 8,
        "arm_upper_l" => 9,
        "arm_lower_l" => 10,
        "hand_l" => 11,
        "clavicle_r" => 12,
        "arm_upper_r" => 13,
        "arm_lower_r" => 14,
        "hand_r" => 15,
        "leg_upper_l" => 17,
        "leg_lower_l" => 18,
        "ankle_l" => 19,
        "leg_upper_r" => 20,
        "leg_lower_r" => 21,
        "ankle_r" => 22,
        _ => return None,
    })
}

pub fn hitbox_set_index(cs2: &CS2, scene_node: usize) -> usize {
    cs2.process
        .read::<u8>(scene_node + cs2.offsets.model_data.hitbox_set) as usize
}
