use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::EXTERNAL_CARD_ROOT;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ExternalStorageSettings {
    pub enabled: bool,
    pub path: PathBuf,
    pub auto_import: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct CoverEditorSettings {
    pub default_width: u32,
    pub default_height: u32,
    pub allow_custom_sizes: bool,
    pub jpeg_quality: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PluginSettings {
    pub enabled: bool,
    pub plugins_dir: PathBuf,
    pub allow_network: bool,
    pub allow_filesystem: bool,
    pub timeout_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct BackgroundSyncSettings {
    pub enabled: bool,
    pub wifi_only: bool,
    pub sync_on_open: bool,
    pub sync_on_close: bool,
    pub sync_interval_minutes: u32,
    pub auto_wifi: bool,
    pub keep_wifi_on: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginTrigger {
    OnBookImport,
    OnBookOpen,
    OnBookClose,
    OnSyncComplete,
    OnStartup,
    OnShutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub path: PathBuf,
    pub triggers: Vec<PluginTrigger>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct CloudSyncSettings {
    pub enabled: bool,
    pub sync_method: CloudSyncMethod,
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub remote_path: String,
    pub auto_sync: bool,
    pub last_sync: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CloudSyncMethod {
    WebDAV,
    KoboCloud,
}

impl Default for ExternalStorageSettings {
    fn default() -> Self {
        ExternalStorageSettings {
            enabled: false,
            path: PathBuf::from(EXTERNAL_CARD_ROOT),
            auto_import: true,
        }
    }
}

impl Default for CoverEditorSettings {
    fn default() -> Self {
        CoverEditorSettings {
            default_width: 600,
            default_height: 800,
            allow_custom_sizes: true,
            jpeg_quality: 85,
        }
    }
}

impl Default for PluginSettings {
    fn default() -> Self {
        PluginSettings {
            enabled: false,
            plugins_dir: PathBuf::from("plugins"),
            allow_network: false,
            allow_filesystem: true,
            timeout_seconds: 30,
        }
    }
}

impl Default for BackgroundSyncSettings {
    fn default() -> Self {
        BackgroundSyncSettings {
            enabled: false,
            wifi_only: true,
            sync_on_open: true,
            sync_on_close: false,
            sync_interval_minutes: 30,
            auto_wifi: true,
            keep_wifi_on: false,
        }
    }
}

impl Default for CloudSyncSettings {
    fn default() -> Self {
        CloudSyncSettings {
            enabled: false,
            sync_method: CloudSyncMethod::WebDAV,
            url: None,
            username: None,
            password: None,
            remote_path: "/".to_string(),
            auto_sync: false,
            last_sync: None,
        }
    }
}
