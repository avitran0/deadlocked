use std::{
    fs::{create_dir_all, read, read_link, read_to_string, rename, write},
    path::PathBuf,
};

use crate::{config::BASE_PATH, os::process::Process, parser::bvh::Bvh};

// bump when the serialized layout of Bvh changes
const BVH_FORMAT_VERSION: u32 = 1;

/// reads ClientVersion from game/csgo/steam.inf, relative to game/bin/linuxsteamrt64/cs2
fn game_version(process: &Process) -> Option<String> {
    let exe = read_link(format!("/proc/{}/exe", process.pid)).ok()?;
    let steam_inf =
        read_to_string(exe.parent()?.parent()?.parent()?.join("csgo/steam.inf")).ok()?;
    steam_inf
        .lines()
        .find_map(|line| line.trim().strip_prefix("ClientVersion="))
        .map(|version| version.trim().to_owned())
}

fn cache_file(map: &str) -> Option<PathBuf> {
    let valid = !map.is_empty()
        && map
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    valid.then(|| BASE_PATH.join("bvh").join(format!("{map}.bvh")))
}

pub fn load(process: &Process, map: &str) -> Option<Bvh> {
    let path = cache_file(map)?;
    let bytes = read(&path).ok()?;
    // check the header first, so an outdated layout is not reported as a parse error
    let ((format_version, game_version), bvh) =
        postcard::take_from_bytes::<(u32, String)>(&bytes).ok()?;
    if format_version != BVH_FORMAT_VERSION || Some(game_version) != self::game_version(process) {
        utils::info!("bvh cache for {map} is outdated");
        return None;
    }
    match postcard::from_bytes(bvh) {
        Ok(bvh) => Some(bvh),
        Err(err) => {
            utils::warn!("failed to parse bvh cache {}: {err}", path.display());
            None
        }
    }
}

pub fn save(process: &Process, map: &str, bvh: &Bvh) {
    let (Some(path), Some(game_version)) = (cache_file(map), game_version(process)) else {
        return;
    };
    let bytes = match postcard::to_stdvec(&(BVH_FORMAT_VERSION, game_version, bvh)) {
        Ok(bytes) => bytes,
        Err(err) => {
            utils::warn!("failed to encode bvh cache: {err}");
            return;
        }
    };
    // write to a temp file and rename so a crash never leaves a truncated cache
    let tmp = path.with_extension("bvh.tmp");
    let result = create_dir_all(BASE_PATH.join("bvh"))
        .and_then(|_| write(&tmp, bytes))
        .and_then(|_| rename(&tmp, &path));
    match result {
        Ok(_) => utils::info!("saved bvh cache to {}", path.display()),
        Err(err) => utils::warn!("failed to write bvh cache {}: {err}", path.display()),
    }
}
