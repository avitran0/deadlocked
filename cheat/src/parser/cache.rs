use std::{
    fs::{File, create_dir_all, read, rename},
    io::Write,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use serde::{Deserialize, Serialize};

use crate::{config::BASE_PATH, os::process::Process, parser::bvh::Bvh};

// bump when the serialized layout of Bvh changes
const BVH_FORMAT_VERSION: u32 = 1;

pub static BVH_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = BASE_PATH.join("bvh");
    if !path.exists() {
        let _ = create_dir_all(&path);
    }
    path
});

#[derive(Serialize, Deserialize)]
struct CachedBvh {
    format_version: u32,
    game_version: String,
    bvh: Bvh,
}

/// reads ClientVersion from game/csgo/steam.inf, relative to the running cs2 binary
pub fn game_version(process: &Process) -> Option<String> {
    // exe lives in game/bin/linuxsteamrt64/cs2
    let game_dir = process.exe_path()?.parent()?.parent()?.parent()?.to_path_buf();
    let steam_inf = std::fs::read_to_string(game_dir.join("csgo").join("steam.inf")).ok()?;
    steam_inf.lines().find_map(|line| {
        line.trim()
            .strip_prefix("ClientVersion=")
            .map(|version| version.trim().to_owned())
    })
}

fn cache_file(map: &str) -> Option<PathBuf> {
    let name = Path::new(map).file_name()?.to_str()?;
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return None;
    }
    Some(BVH_PATH.join(format!("{name}.bvh")))
}

pub fn load(map: &str, game_version: &str) -> Option<Bvh> {
    let path = cache_file(map)?;
    let bytes = read(&path).ok()?;
    let cached: CachedBvh = match postcard::from_bytes(&bytes) {
        Ok(cached) => cached,
        Err(err) => {
            utils::warn!("failed to parse bvh cache {}: {err}", path.display());
            return None;
        }
    };
    if cached.format_version != BVH_FORMAT_VERSION || cached.game_version != game_version {
        utils::info!("bvh cache for {map} is outdated");
        return None;
    }
    Some(cached.bvh)
}

pub fn save(map: &str, game_version: &str, bvh: Bvh) -> Bvh {
    let Some(path) = cache_file(map) else {
        return bvh;
    };
    let cached = CachedBvh {
        format_version: BVH_FORMAT_VERSION,
        game_version: game_version.to_owned(),
        bvh,
    };
    match postcard::to_stdvec(&cached) {
        Ok(bytes) => {
            // write to a temp file and rename so a crash never leaves a truncated cache
            let tmp = path.with_extension("bvh.tmp");
            let result = File::create(&tmp)
                .and_then(|mut file| file.write_all(&bytes))
                .and_then(|_| rename(&tmp, &path));
            match result {
                Ok(_) => utils::info!("saved bvh cache to {}", path.display()),
                Err(err) => utils::warn!("failed to write bvh cache {}: {err}", path.display()),
            }
        }
        Err(err) => utils::warn!("failed to encode bvh cache: {err}"),
    }
    cached.bvh
}
