use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    config::BASE_PATH,
    cs2::{CS2, bvh::read_bvh},
    parser::bvh::Bvh,
};

pub mod bvh;

#[derive(Deserialize)]
struct BvhCache {
    build_date: String,
    bvh: Bvh,
}

#[derive(Serialize)]
struct BvhCacheRef<'a> {
    build_date: &'a str,
    bvh: &'a Bvh,
}

pub fn read_map(cs2: &CS2) -> Option<Bvh> {
    let (triangles, materials) = read_bvh(cs2)?;
    let mut bvh = Bvh::new();
    bvh.set(triangles, materials);
    bvh.build();
    Some(bvh)
}

pub fn load_map(map_name: &str, build_date: &str) -> Option<Bvh> {
    if build_date.is_empty() {
        return None;
    }

    let path = bvh_cache_path(map_name);
    let bytes = std::fs::read(path).ok()?;
    let cache: BvhCache = postcard::from_bytes(&bytes).ok()?;
    if cache.build_date != build_date {
        return None;
    }
    Some(cache.bvh)
}

pub fn save_map(map_name: &str, build_date: &str, bvh: &Bvh) {
    if build_date.is_empty() {
        return;
    }

    let path = bvh_cache_path(map_name);
    let cache = BvhCacheRef { build_date, bvh };
    let Ok(bytes) = postcard::to_stdvec(&cache) else {
        return;
    };

    let temp_path = path.with_extension("bvh.tmp");
    if std::fs::write(&temp_path, bytes).is_ok() {
        let _ = std::fs::rename(temp_path, path);
    }
}

fn bvh_cache_path(map_name: &str) -> PathBuf {
    let maps_dir = BASE_PATH.join("bvh");
    std::fs::create_dir_all(&maps_dir).expect("failed to create BVH cache directory");
    let bvh_name = if map_name.ends_with(".vpk") {
        map_name.replace(".vpk", ".bvh")
    } else {
        format!("{map_name}.bvh")
    };
    maps_dir.join(bvh_name)
}

pub fn clear_cache() {
    let maps_dir = BASE_PATH.join("bvh");
    if maps_dir.exists() {
        std::fs::remove_dir_all(maps_dir).expect("failed to clear BVH cache");
    }
}
