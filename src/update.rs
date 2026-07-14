use std::time::Duration;

use serde::Deserialize;
use ureq::Agent;

const REPO: &str = "avitran0/deadlocked";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Default, Clone, PartialEq)]
pub enum UpdateStatus {
    #[default]
    UpToDate,
    Available {
        version: String,
        url: String,
    },
    Error(String),
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
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
        UpdateStatus::Available {
            version: release.tag_name,
            url: release.html_url,
        }
    } else {
        UpdateStatus::UpToDate
    }
}
