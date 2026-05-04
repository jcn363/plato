use crate::validation::validate_range;
use anyhow::Error;
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
    pub dropbox_token: Option<String>,
    pub google_drive_token: Option<String>,
    pub onedrive_token: Option<String>,
    pub conflict_resolution: ConflictResolution,
    pub offline_first: bool,
    pub sync_annotations: bool,
    pub sync_highlights: bool,
    pub sync_reading_position: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConflictResolution {
    LastWriteWins,
    PreferLocal,
    PreferRemote,
    Merge,
}

impl Default for ConflictResolution {
    fn default() -> Self {
        ConflictResolution::Merge
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CloudSyncMethod {
    WebDAV,
    KoboCloud,
    Dropbox,
    GoogleDrive,
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

impl CoverEditorSettings {
    /// Validates cover editor settings are within acceptable ranges
    pub fn validate(&self) -> Result<(), Error> {
        // Validate dimensions (must be reasonable cover sizes)
        validate_range(self.default_width, 100, 2000, "cover_editor.default_width")?;
        validate_range(
            self.default_height,
            100,
            2000,
            "cover_editor.default_height",
        )?;

        // Validate JPEG quality (must be 1-100)
        validate_range(self.jpeg_quality, 1, 100, "cover_editor.jpeg_quality")?;

        Ok(())
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
            dropbox_token: None,
            google_drive_token: None,
            onedrive_token: None,
            conflict_resolution: ConflictResolution::default(),
            offline_first: true,
            sync_annotations: true,
            sync_highlights: true,
            sync_reading_position: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SocialSettings {
    pub enabled: bool,
    pub share_to_readwise: bool,
    pub share_to_obsidian: bool,
    pub generate_quote_cards: bool,
    pub quote_card_style: QuoteCardStyle,
    pub export_format: SocialExportFormat,
    pub reading_groups_dir: PathBuf,
    pub show_progress_in_notification: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QuoteCardStyle {
    Classic,
    Modern,
    Minimal,
    Dark,
}

impl Default for QuoteCardStyle {
    fn default() -> Self {
        QuoteCardStyle::Classic
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SocialExportFormat {
    Markdown,
    Json,
    Csv,
    Html,
}

impl Default for SocialExportFormat {
    fn default() -> Self {
        SocialExportFormat::Markdown
    }
}

impl Default for SocialSettings {
    fn default() -> Self {
        SocialSettings {
            enabled: false,
            share_to_readwise: true,
            share_to_obsidian: true,
            generate_quote_cards: true,
            quote_card_style: QuoteCardStyle::default(),
            export_format: SocialExportFormat::default(),
            reading_groups_dir: PathBuf::from("ReadingGroups"),
            show_progress_in_notification: true,
        }
    }
}
