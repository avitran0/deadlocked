use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use serde::Deserialize;
use ureq::Agent;

use crate::config::BASE_PATH;

mod convert;
mod keyvalues;

const VRF_REPO: &str = "ValveResourceFormat/ValveResourceFormat";
const CLI_ASSET_NAME: &str = "cli-linux-x64.zip";
// both ship as base CS2 content regardless of which agents the user owns.
// used as the fallback mesh for a player whose equipped agent isn't in
// agent_models.json: players using their team's plain default skin (no
// agent purchased/selected) don't have a real entry in items_game.txt's
// per-def_index item list, so they need a team-correct fallback here
// rather than one fixed mesh for everyone (a Terrorist falling back to a
// Counter-Terrorist model looks obviously wrong)
pub const FALLBACK_AGENT_T: &str = "tm_phoenix";
pub const FALLBACK_AGENT_CT: &str = "ctm_fbi";
const AGENT_INDEX_FILE: &str = "agent_models.json";

#[derive(Default, Clone)]
pub enum ExtractStatus {
    #[default]
    Idle,
    Running,
    Progress {
        done: usize,
        total: usize,
    },
    Done(PathBuf),
    Error(String),
}

#[derive(Deserialize)]
struct Release {
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// finds the user's own CS2 install, downloads (and caches) VRF's CLI if
/// needed, reads the user's own items_game.txt to find every agent
/// currently in the game, extracts+converts each distinct one, and writes
/// an index mapping each agent's item def_index to its mesh file. reads
/// only; never redistributes CS2 assets.
pub fn extract_all_agent_models(
    status: &std::sync::Arc<utils::Mutex<ExtractStatus>>,
) -> Result<PathBuf, String> {
    let cs2_root = find_cs2_install().ok_or("could not find a Counter-Strike 2 install")?;
    let vpk_path = cs2_root.join("game/csgo/pak01_dir.vpk");
    if !vpk_path.exists() {
        return Err(format!("{} not found", vpk_path.display()));
    }
    let cli = ensure_vrf_cli()?;

    let items_text = run_vrf_extract_raw(&cli, &vpk_path, "scripts/items/items_game.txt")?;
    let items_text = String::from_utf8_lossy(&items_text);
    let agent_table = parse_agent_table(&items_text);
    if agent_table.is_empty() {
        return Err("no agent entries found in items_game.txt".to_string());
    }

    // many def_indices share the same underlying model (cosmetic-only
    // variants), so extract each distinct model path once
    let mut model_to_stem: HashMap<&str, &str> = HashMap::new();
    for model_path in agent_table.values() {
        let stem = model_stem(model_path);
        model_to_stem.insert(model_path.as_str(), stem);
    }
    let unique_models: Vec<&str> = model_to_stem.keys().copied().collect();

    let total = unique_models.len();
    *status.lock() = ExtractStatus::Progress { done: 0, total };

    for (i, model_path) in unique_models.iter().enumerate() {
        let stem = model_to_stem[model_path];
        let out_path = BASE_PATH.join(format!("player_model_{stem}.dlms"));
        if out_path.exists() {
            *status.lock() = ExtractStatus::Progress { done: i + 1, total };
            continue;
        }

        let vmdl_path = format!("{}_c", model_path); // .vmdl -> .vmdl_c
        match run_vrf_extraction(&cli, &vpk_path, &vmdl_path)
            .and_then(|glb| convert::glb_to_dlms(&glb))
        {
            Ok(dlms_bytes) => {
                let _ = std::fs::write(&out_path, dlms_bytes);
            }
            Err(err) => {
                utils::warn!("skipping agent model {model_path}: {err}");
            }
        }
        *status.lock() = ExtractStatus::Progress { done: i + 1, total };
    }

    let index: HashMap<u16, String> = agent_table
        .into_iter()
        .map(|(def_index, model_path)| (def_index, model_stem(&model_path).to_string()))
        .collect();
    let index_path = BASE_PATH.join(AGENT_INDEX_FILE);
    let index_json = serde_json::to_string(&index).map_err(|e| e.to_string())?;
    std::fs::write(&index_path, index_json).map_err(|e| e.to_string())?;

    Ok(index_path)
}

/// loads the def_index -> model stem table written by
/// extract_all_agent_models(), if it exists yet
pub fn load_agent_index() -> HashMap<u16, String> {
    let Ok(text) = std::fs::read_to_string(BASE_PATH.join(AGENT_INDEX_FILE)) else {
        return HashMap::new();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

fn model_stem(model_path: &str) -> &str {
    Path::new(model_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(FALLBACK_AGENT_CT)
}

/// items_game.txt's "items" block appears multiple times and must be
/// merged (KeyValues allows duplicate keys at the same level); this pulls
/// out def_index -> model_player for every entry that's an actual agent
/// character model, not a glove/weapon skin or other cosmetic
fn parse_agent_table(items_game_text: &str) -> HashMap<u16, String> {
    let root = keyvalues::parse(items_game_text);
    let Some(items_game) = root
        .iter()
        .find(|(k, _)| k == "items_game")
        .and_then(|(_, v)| v.as_block())
    else {
        return HashMap::new();
    };

    let mut table = HashMap::new();
    for (key, value) in items_game {
        if key != "items" {
            continue;
        }
        let Some(entries) = value.as_block() else {
            continue;
        };
        for (def_index, item) in entries {
            let Ok(def_index) = def_index.parse::<u16>() else {
                continue;
            };
            let Some(item) = item.as_block() else {
                continue;
            };
            let Some(model) = item
                .iter()
                .find(|(k, _)| k == "model_player")
                .and_then(|(_, v)| v.as_str())
            else {
                continue;
            };
            if model.starts_with("agents/models/") && !model.contains("/shared/") {
                table.insert(def_index, model.to_string());
            }
        }
    }
    table
}

fn find_cs2_install() -> Option<PathBuf> {
    let home = PathBuf::from(std::env::var("HOME").ok()?);

    let steam_roots = [
        home.join(".local/share/Steam"),
        home.join(".steam/steam"),
        home.join(".steam/root"),
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
    ];

    let mut library_paths: Vec<PathBuf> = Vec::new();
    for root in &steam_roots {
        library_paths.push(root.clone());
        library_paths.extend(parse_library_folders(
            &root.join("steamapps/libraryfolders.vdf"),
        ));
    }

    library_paths
        .into_iter()
        .map(|library| library.join("steamapps/common/Counter-Strike Global Offensive"))
        .find(|path| path.join("game/csgo/pak01_dir.vpk").exists())
}

/// steam's libraryfolders.vdf is a simple nested "key" "value" text format;
/// this only pulls out the "path" entries it actually needs, not a general
/// vdf parser
fn parse_library_folders(vdf_path: &Path) -> Vec<PathBuf> {
    let Ok(text) = std::fs::read_to_string(vdf_path) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("\"path\"")?;
            let value = rest.trim().trim_matches('"');
            Some(PathBuf::from(value.replace("\\\\", "\\")))
        })
        .collect()
}

fn ensure_vrf_cli() -> Result<PathBuf, String> {
    let cache_dir = BASE_PATH.join("vrf_cli");
    let cli_path = cache_dir.join("Source2Viewer-CLI");
    if cli_path.exists() {
        return Ok(cli_path);
    }

    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;

    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into();

    let release_url = format!("https://api.github.com/repos/{VRF_REPO}/releases/latest");
    let mut resp = agent
        .get(&release_url)
        .header("User-Agent", "deadlocked")
        .header("Accept", "application/vnd.github.v3+json")
        .call()
        .map_err(|e| format!("failed to query VRF releases: {e}"))?;
    let release: Release = resp
        .body_mut()
        .read_json()
        .map_err(|e| format!("failed to parse VRF release info: {e}"))?;

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == CLI_ASSET_NAME)
        .ok_or_else(|| format!("no {CLI_ASSET_NAME} asset in the latest VRF release"))?;

    // the cli binary itself is ~100MB uncompressed; the zip is well over
    // ureq's 10MB read_to_vec default
    let mut resp = agent
        .get(&asset.browser_download_url)
        .header("User-Agent", "deadlocked")
        .call()
        .map_err(|e| format!("failed to download VRF cli: {e}"))?;
    let zip_bytes = resp
        .body_mut()
        .with_config()
        .limit(200 * 1024 * 1024)
        .read_to_vec()
        .map_err(|e| format!("failed to read VRF cli download: {e}"))?;

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes))
        .map_err(|e| format!("failed to open VRF cli zip: {e}"))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("failed to read VRF cli zip entry: {e}"))?;
        let Some(name) = entry.enclosed_name() else {
            continue;
        };
        let out_path = cache_dir.join(name);
        let mut out_file = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out_file).map_err(|e| e.to_string())?;
    }

    if !cli_path.exists() {
        return Err("VRF cli zip did not contain Source2Viewer-CLI".to_string());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&cli_path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
    }

    Ok(cli_path)
}

/// runs the VRF cli against a single file inside the vpk without
/// decompiling it, for plain script/text files like items_game.txt
fn run_vrf_extract_raw(cli: &Path, vpk_path: &Path, file_path: &str) -> Result<Vec<u8>, String> {
    let out_path = BASE_PATH.join("vrf_cli/raw_out");
    let _ = std::fs::remove_file(&out_path);

    let output = Command::new(cli)
        .arg("-i")
        .arg(vpk_path)
        .arg("-f")
        .arg(file_path)
        .arg("-o")
        .arg(&out_path)
        .output()
        .map_err(|e| format!("failed to run VRF cli: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "VRF cli exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    std::fs::read(&out_path)
        .map_err(|e| format!("VRF cli did not produce {}: {e}", out_path.display()))
}

fn run_vrf_extraction(cli: &Path, vpk_path: &Path, model_path: &str) -> Result<Vec<u8>, String> {
    let out_dir = BASE_PATH.join("vrf_cli/out");
    let _ = std::fs::remove_dir_all(&out_dir);
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

    let output = Command::new(cli)
        .arg("-i")
        .arg(vpk_path)
        .arg("-f")
        .arg(model_path)
        .arg("-o")
        .arg(&out_dir)
        .arg("-d")
        .arg("--gltf_export_format")
        .arg("glb")
        .arg("--gltf_export_animations")
        .arg("--gltf_animation_list")
        .arg("none_such_clip")
        .output()
        .map_err(|e| format!("failed to run VRF cli: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "VRF cli exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let glb_path = out_dir.join(model_path).with_extension("glb");
    std::fs::read(&glb_path)
        .map_err(|e| format!("VRF cli did not produce {}: {e}", glb_path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_library_folders_vdf() {
        let dir = std::env::temp_dir().join("deadlocked_test_libraryfolders");
        std::fs::create_dir_all(&dir).unwrap();
        let vdf_path = dir.join("libraryfolders.vdf");
        std::fs::write(
            &vdf_path,
            "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"/mnt/games/Steam\"\n\t}\n}\n",
        )
        .unwrap();

        let paths = parse_library_folders(&vdf_path);
        assert_eq!(paths, vec![PathBuf::from("/mnt/games/Steam")]);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn parses_agent_table_with_duplicate_items_blocks() {
        let text = r#"
"items_game"
{
    "items"
    {
        "100"
        {
            "name" "customplayer_ctm_fbi"
            "model_player" "agents/models/ctm_fbi/ctm_fbi.vmdl"
        }
        "101"
        {
            "name" "some_glove"
            "model_player" "agents/models/shared/arms/glove_sporty/glove_sporty.vmdl"
        }
    }
    "items"
    {
        "200"
        {
            "name" "customplayer_tm_phoenix"
            "model_player" "agents/models/tm_phoenix/tm_phoenix.vmdl"
        }
    }
}
"#;
        let table = parse_agent_table(text);
        assert_eq!(table.len(), 2);
        assert_eq!(
            table.get(&100).map(String::as_str),
            Some("agents/models/ctm_fbi/ctm_fbi.vmdl")
        );
        assert_eq!(
            table.get(&200).map(String::as_str),
            Some("agents/models/tm_phoenix/tm_phoenix.vmdl")
        );
        assert!(
            !table.contains_key(&101),
            "glove entries must be filtered out"
        );
    }

    // find_cs2_install() and extract_all_agent_models() both depend on real
    // machine/network state (a real CS2 install, GitHub reachability) and
    // are not suitable as unit tests; verified manually this session
    // against a real install (141 agent entries, 79 unique models,
    // def_index 5300 -> agents/models/ctm_fbi/ctm_fbi.vmdl), see the
    // session notes.
}
