use std::time::Duration;

use serde::Deserialize;
use ureq::Agent;

const REPO: &str = "avitran0/deadlocked";
const VERSION: &str = env!("CARGO_PKG_VERSION");
// the release asset uploaded for the compiled binary; matches CARGO_PKG_NAME
const BINARY_ASSET_NAME: &str = "deadlocked";

#[derive(Default, Clone, PartialEq)]
pub enum UpdateStatus {
    #[default]
    UpToDate,
    Available {
        version: String,
        /// the release notes page, for "what's new"
        html_url: String,
        /// direct download link for the compiled binary, if the release
        /// has one attached
        asset_url: Option<String>,
    },
    Error(String),
}

#[derive(Default, Clone)]
pub enum ApplyStatus {
    #[default]
    Idle,
    Downloading,
    Done,
    Error(String),
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn check() -> UpdateStatus {
    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(1)))
        .build()
        .into();

    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");

    let mut resp = match agent
        .get(&url)
        .header("User-Agent", "deadlocked")
        .header("Accept", "application/vnd.github.v3+json")
        .call()
    {
        Ok(resp) => resp,
        Err(ureq::Error::StatusCode(code)) => {
            return UpdateStatus::Error(format!("GitHub API returned {code}"));
        }
        Err(e) => return UpdateStatus::Error(e.to_string()),
    };

    let release: Release = match resp.body_mut().read_json() {
        Ok(r) => r,
        Err(e) => return UpdateStatus::Error(e.to_string()),
    };

    let latest = release.tag_name.trim_start_matches('v');

    if latest != VERSION {
        let asset_url = release
            .assets
            .iter()
            .find(|a| a.name == BINARY_ASSET_NAME)
            .map(|a| a.browser_download_url.clone());

        UpdateStatus::Available {
            version: release.tag_name,
            html_url: release.html_url,
            asset_url,
        }
    } else {
        UpdateStatus::UpToDate
    }
}

/// downloads the compiled release binary and atomically replaces the
/// currently running executable. does not restart the process: the
/// currently running instance keeps working (linux does not care that its
/// own backing file changed), the new binary takes effect on next launch.
pub fn apply_update(asset_url: &str) -> Result<(), String> {
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = current_exe
        .parent()
        .ok_or("current executable has no parent directory")?;

    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .build()
        .into();

    let mut resp = agent
        .get(asset_url)
        .header("User-Agent", "deadlocked")
        .call()
        .map_err(|e| format!("failed to download update: {e}"))?;
    let bytes = resp
        .body_mut()
        .with_config()
        .limit(100 * 1024 * 1024)
        .read_to_vec()
        .map_err(|e| format!("failed to read update download: {e}"))?;

    if bytes.get(0..4) != Some(&[0x7f, b'E', b'L', b'F']) {
        return Err("downloaded file is not a valid executable".to_string());
    }

    let tmp_path = dir.join(".deadlocked.update.tmp");
    std::fs::write(&tmp_path, &bytes).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
    }

    std::fs::rename(&tmp_path, &current_exe).map_err(|e| e.to_string())?;

    Ok(())
}
