use crate::settings::PLATO_VERSION;
use anyhow::Error;
use reqwest::blocking::Client;

pub struct UpdateChecker {
    check_url: String,
}

impl UpdateChecker {
    pub fn new(check_url: &str) -> Self {
        UpdateChecker {
            check_url: check_url.to_string(),
        }
    }

    pub fn current_version() -> &'static str {
        PLATO_VERSION
    }

    pub fn check_for_update(&self) -> Result<Option<UpdateInfo>, Error> {
        let client = Client::new();
        let body = client
            .get(&self.check_url)
            .send()
            .map_err(|e| anyhow::anyhow!("Failed to check for updates: {}", e))?
            .text()
            .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?;

        let info = serde_json::from_str::<UpdateInfo>(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse update info: {}", e))?;

        if self.is_newer(&info.version) {
            Ok(Some(info))
        } else {
            Ok(None)
        }
    }

    fn is_newer(&self, remote_version: &str) -> bool {
        let current = Self::parse_version(Self::current_version());
        let remote = Self::parse_version(remote_version);

        if remote.0 != current.0 {
            return remote.0 > current.0;
        }
        if remote.1 != current.1 {
            return remote.1 > current.1;
        }
        remote.2 > current.2
    }

    fn parse_version(version: &str) -> (u32, u32, u32) {
        let parts: Vec<&str> = version.trim_start_matches('v').split('.').collect();
        let major = parts.get(0).and_then(|v| v.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|v| v.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub release_notes: String,
    pub release_date: String,
}

impl Default for UpdateChecker {
    fn default() -> Self {
        Self::new("https://github.com/anomalyco/plato/releases/latest/version.json")
    }
}
