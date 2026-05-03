//! Settings and Configuration Module
//!
//! This module provides centralized configuration management for Plato,
//! including settings loading, validation, and saving.
//!
//! ## Architecture
//!
//! The settings module is organized by functional domain:
//!
//! - **defaults.rs**: Default values and constants
//! - **manager.rs**: Centralized configuration loading/saving with validation
//! - **reading.rs**: Reader view settings (font, margins, layout)
//! - **library.rs**: Library configuration (paths, modes, hooks)
//! - **interface.rs**: UI settings (home view, navigation, intermissions)
//! - **display.rs**: Display settings (battery, night light)
//! - **features.rs**: Feature toggles (external storage, plugins, sync)
//! - **preset.rs**: Frontlight presets
//! - **theme.rs**: Theme and color settings
//! - **thumbnail.rs**: Thumbnail generation settings
//! - **tools.rs**: PDF tool settings
//!
//! ## Module Hierarchy
//!
//! ```text
//! settings/
//! ├── mod.rs          (main Settings struct and validation)
//! ├── manager.rs      (ConfigManager for load/save)
//! ├── defaults.rs     (constants and defaults)
//! ├── reading.rs      (reader settings)
//! ├── library.rs      (library settings)
//! ├── interface.rs    (UI settings)
//! ├── display.rs      (display settings)
//! ├── features.rs     (feature settings)
//! ├── preset.rs       (frontlight presets)
//! ├── theme.rs        (theme settings)
//! ├── thumbnail.rs    (thumbnail settings)
//! └── tools.rs        (PDF tool settings)
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use plato_core::settings::{Settings, ConfigManager};
//!
//! // Load with validation
//! let settings = ConfigManager::new().load()?;
//!
//! // Or use defaults if file missing
//! let settings = ConfigManager::new().load_or_default();
//!
//! // Enhanced: User profiles for different configurations
//! let profile_settings = ConfigManager::new().load_profile("reading")?;
//! ```

mod defaults;
mod display;
mod features;
mod interface;
mod library;
mod manager;
mod opds;
mod preset;
mod reading;
mod theme;
mod thumbnail;
mod tools;

use crate::validation::{validate_finite_f32, validate_range, validate_string_length};
use anyhow::{bail, Context, Error};

use crate::frontlight::LightLevels;
use crate::metadata::{SortMethod, TextAlign};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub use self::preset::{guess_frontlight, LightPreset};
pub use self::theme::{ThemeMode, ThemeSchedule, ThemeSettings, TimeOfDay};
pub use defaults::*;
pub use display::*;
pub use features::*;
pub use interface::*;
pub use library::*;
pub use manager::{load_settings, save_settings, ConfigManager};
pub use opds::*;
pub use reading::*;
pub use thumbnail::*;
pub use tools::*;

#[derive(Debug, Clone, Serialize, Deserialize, validator::Validate)]
#[serde(default, rename_all = "kebab-case")]
pub struct Settings {
    #[validate(range(min = 0))]
    pub selected_library: usize,
    #[validate(length(min = 1, max = 50))]
    pub keyboard_layout: String,
    pub frontlight: bool,
    pub wifi: bool,
    pub inverted: bool,
    pub dark_mode: bool,
    pub theme_settings: ThemeSettings,
    pub sleep_cover: bool,
    pub sleep_cover_fill: bool,
    pub auto_share: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_lock: Option<RotationLock>,
    pub button_scheme: ButtonScheme,
    #[validate(range(min = 0.0, max = 3600.0))]
    pub auto_suspend: f32,
    #[validate(range(min = 0.0, max = 3600.0))]
    pub auto_power_off: f32,
    #[validate(length(min = 2, max = 10))]
    pub language: String,
    #[validate(length(min = 2, max = 10))]
    pub locale: String,
    pub ui_font: UiFont,
    #[validate(length(min = 1, max = 20))]
    pub time_format: String,
    #[validate(length(min = 1, max = 20))]
    pub date_format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_urls_queue: Option<PathBuf>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[validate(length(min = 1))]
    pub libraries: Vec<LibrarySettings>,
    pub intermissions: Intermissions,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub frontlight_presets: Vec<LightPreset>,
    pub home: HomeSettings,
    pub reader: ReaderSettings,
    pub import: ImportSettings,
    pub dictionary: DictionarySettings,
    pub sketch: SketchSettings,
    pub calculator: CalculatorSettings,
    pub battery: BatterySettings,
    pub frontlight_levels: LightLevels,
    pub reading_goals: ReadingGoals,
    pub night_light_schedule: NightLightSchedule,
    pub gestures: GestureSettings,
    pub search: SearchSettings,
    pub reader_presets: Vec<ReaderPreset>,
    pub external_storage: ExternalStorageSettings,
    pub cover_editor: CoverEditorSettings,
    pub plugin_settings: PluginSettings,
    pub background_sync: BackgroundSyncSettings,
    pub cloud_sync: CloudSyncSettings,
    pub thumbnail: ThumbnailSettings,
    pub opds: OpdsSettings,
    pub accessibility: AccessibilitySettings,
    pub calibre: CalibreSettings,
    pub epub_to_pdf: EpubToPdfSettings,
    pub goodreads: GoodreadsSettings,
    pub pocket: PocketSettings,
    pub cloud_storage: CloudStorageSettings,
    pub ai: AiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct AiSettings {
    /// Enable AI features
    pub enabled: bool,
    /// AI provider (ollama, openai, claude)
    pub provider: String,
    /// Model name
    pub model: String,
    /// Endpoint for local/cloud providers
    pub endpoint: Option<String>,
    /// API key for cloud providers
    pub api_key: Option<String>,
    /// Enable spoiler protection in reader
    pub spoiler_protection: bool,
    /// Enable semantic search
    pub semantic_search: bool,
    /// Memory threshold (MB) - AI disabled below this
    pub memory_threshold_mb: usize,
    /// Allow AI on low battery
    pub allow_on_low_battery: bool,
    /// Enable AI caching
    pub caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
}

impl AiSettings {
    pub fn can_run(&self, total_ram_mb: usize) -> bool {
        self.enabled && total_ram_mb >= self.memory_threshold_mb
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct AccessibilitySettings {
    /// High contrast mode
    pub high_contrast: bool,
    /// Letter spacing (em units)
    pub letter_spacing: f32,
    /// Word spacing (em units)
    pub word_spacing: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Large text mode scale factor
    pub large_text_scale: f32,
    /// Focus mode enabled
    pub focus_mode: bool,
    /// Color blindness mode (none, deuteranopia, protanopia, tritanopia)
    pub color_blindness_mode: String,
    /// Dyslexic-friendly font
    pub dyslexic_font: bool,
    /// Dyslexia font family (opendyslexic, atkinson, lexend)
    pub dyslexic_font_family: String,
    /// Bionic reading mode - bold first half of words
    pub bionic_reading: bool,
    /// Bionic reading intensity (0.0 to 1.0, how much of word to bold)
    pub bionic_intensity: f32,
    /// Auto-pace: automatic page turn with adjustable speed
    pub auto_pace: bool,
    /// Auto-pace speed in words per minute (100-600)
    pub auto_pace_wpm: u32,
    /// Enable dyslexia-friendly fonts bundling
    pub use_accessibility_fonts: bool,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            high_contrast: false,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            line_height: 1.2, // Valid default within range [0.5, 3.0]
            large_text_scale: 1.0,
            focus_mode: false,
            color_blindness_mode: "none".to_string(),
            dyslexic_font: false,
            dyslexic_font_family: "opendyslexic".to_string(),
            bionic_reading: false,
            bionic_intensity: 0.5,
            auto_pace: false,
            auto_pace_wpm: 300,
            use_accessibility_fonts: true,
        }
    }
}

impl AccessibilitySettings {
    pub fn validate(&self) -> Result<(), Error> {
        validate_finite_f32(self.letter_spacing, "letter_spacing", 0.0, 1.0)?;
        validate_finite_f32(self.word_spacing, "word_spacing", 0.0, 2.0)?;
        validate_finite_f32(self.line_height, "line_height", 0.5, 3.0)?;
        validate_finite_f32(self.large_text_scale, "large_text_scale", 1.0, 3.0)?;
        validate_finite_f32(self.bionic_intensity, "bionic_intensity", 0.0, 1.0)?;
        validate_range(self.auto_pace_wpm, 100, 600, "auto_pace_wpm")?;
        // Validate dyslexia font family
        let valid_families = ["opendyslexic", "atkinson", "lexend"];
        if !valid_families.contains(&self.dyslexic_font_family.as_str()) {
            bail!(
                "dyslexic_font_family must be one of: {}",
                valid_families.join(", ")
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct CalibreSettings {
    /// Calibre Content Server host
    pub host: String,
    /// Calibre Content Server port
    pub port: u16,
    /// Calibre Content Server username (optional)
    pub username: Option<String>,
    /// Calibre Content Server password (optional)
    pub password: Option<String>,
    /// Auto-sync on Wi-Fi connection
    pub auto_sync: bool,
    /// Sync metadata (ratings, tags, collections)
    pub sync_metadata: bool,
    /// Last sync timestamp
    pub last_sync: Option<i64>,
}

impl CalibreSettings {
    pub fn validate(&self) -> Result<(), Error> {
        if self.host.is_empty() {
            bail!("Calibre host cannot be empty");
        }
        if self.port == 0 {
            bail!("Calibre port must be between 1 and 65535");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct EpubToPdfSettings {
    /// Page size (A4, A5, letter, custom)
    pub page_size: String,
    /// Custom page width (mm)
    pub custom_width: Option<f32>,
    /// Custom page height (mm)
    pub custom_height: Option<f32>,
    /// Margin top (mm)
    pub margin_top: f32,
    /// Margin bottom (mm)
    pub margin_bottom: f32,
    /// Margin left (mm)
    pub margin_left: f32,
    /// Margin right (mm)
    pub margin_right: f32,
    /// Font embedding
    pub embed_fonts: bool,
    /// Image quality (1-100)
    pub image_quality: u8,
}

impl EpubToPdfSettings {
    pub fn validate(&self) -> Result<(), Error> {
        if !["A4", "A5", "letter", "custom"].contains(&self.page_size.as_str()) {
            bail!("Invalid page size: {}", self.page_size);
        }
        if self.page_size == "custom" {
            if let Some(w) = self.custom_width {
                if w <= 0.0 || w > 1000.0 {
                    bail!("Custom width must be between 0 and 1000 mm");
                }
            }
            if let Some(h) = self.custom_height {
                if h <= 0.0 || h > 1000.0 {
                    bail!("Custom height must be between 0 and 1000 mm");
                }
            }
        }
        validate_range(self.image_quality, 1, 100, "image_quality")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct GoodreadsSettings {
    /// Goodreads API key
    pub api_key: Option<String>,
    /// Goodreads API secret
    pub api_secret: Option<String>,
    /// OAuth access token
    pub access_token: Option<String>,
    /// OAuth access token secret
    pub access_token_secret: Option<String>,
    /// Auto-sync reading progress
    pub auto_sync: bool,
    /// Sync shelves
    pub sync_shelves: bool,
    /// Sync reviews
    pub sync_reviews: bool,
}

impl GoodreadsSettings {
    pub fn validate(&self) -> Result<(), Error> {
        if let Some(key) = &self.api_key {
            if key.is_empty() {
                bail!("Goodreads API key cannot be empty");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct PocketSettings {
    /// Pocket consumer key
    pub consumer_key: Option<String>,
    /// Pocket access token
    pub access_token: Option<String>,
    /// Auto-sync on Wi-Fi connection
    pub auto_sync: bool,
    /// Sync reading progress
    pub sync_progress: bool,
    /// Archive after reading
    pub archive_after_reading: bool,
}

impl PocketSettings {
    pub fn validate(&self) -> Result<(), Error> {
        if let Some(key) = &self.consumer_key {
            if key.is_empty() {
                bail!("Pocket consumer key cannot be empty");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct CloudStorageSettings {
    /// Dropbox access token
    pub dropbox_token: Option<String>,
    /// Google Drive access token
    pub google_drive_token: Option<String>,
    /// OneDrive access token
    pub onedrive_token: Option<String>,
    /// Auto-sync on Wi-Fi connection
    pub auto_sync: bool,
    /// Sync reading progress
    pub sync_progress: bool,
    /// Sync annotations
    pub sync_annotations: bool,
}

impl CloudStorageSettings {
    pub fn validate(&self) -> Result<(), Error> {
        // No required fields for cloud storage - tokens are optional
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ImportSettings {
    pub unshare_trigger: bool,
    pub startup_trigger: bool,
    pub sync_metadata: bool,
    pub metadata_kinds: FxHashSet<String>,
    pub allowed_kinds: FxHashSet<String>,
    pub enable_duplicates_detection: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ReaderPreset {
    pub name: String,
    pub font_size: Option<f32>,
    pub font_family: Option<String>,
    pub text_align: Option<TextAlign>,
    pub margin_width: Option<i32>,
    pub line_height: Option<f32>,
    pub continuous_fit_to_width: Option<bool>,
    pub manga_mode: Option<bool>,
}

impl ReaderPreset {
    /// Validates reader preset values are within acceptable ranges
    ///
    /// # Validation Rules
    /// - name: 1 to 100 characters (required)
    /// - font_size: 4.0 to 72.0 points (if specified)
    /// - margin_width: 0 to 100 (if specified)
    /// - line_height: 0.5 to 3.0 (if specified)
    pub fn validate(&self) -> Result<(), Error> {
        // Name is required and must be reasonable length
        validate_string_length(&self.name, "reader_preset.name", 1, 100)?;

        // Validate optional font_size
        if let Some(size) = self.font_size {
            validate_finite_f32(size, "reader_preset.font_size", 4.0, 72.0)?;
        }

        // Validate optional margin_width
        if let Some(margin) = self.margin_width {
            validate_range(margin, 0, 100, "reader_preset.margin_width")?;
        }

        // Validate optional line_height
        if let Some(height) = self.line_height {
            validate_finite_f32(height, "reader_preset.line_height", 0.5, 3.0)?;
        }

        // Validate optional font_family length
        if let Some(ref family) = self.font_family {
            validate_string_length(family, "reader_preset.font_family", 1, 100)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct DictionarySettings {
    pub margin_width: i32,
    pub font_size: f32,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub languages: BTreeMap<String, Vec<String>>,
}

impl Default for DictionarySettings {
    fn default() -> Self {
        DictionarySettings {
            font_size: DEFAULT_DICTIONARY_FONT_SIZE,
            margin_width: 4,
            languages: BTreeMap::new(),
        }
    }
}

impl DictionarySettings {
    /// Validates dictionary settings are within acceptable ranges
    pub fn validate(&self) -> Result<(), Error> {
        // Font size must be reasonable (4 to 72 points)
        validate_finite_f32(self.font_size, "dictionary.font_size", 4.0, 72.0)?;

        // Margin width must be reasonable (0 to 50)
        validate_range(self.margin_width, 0, 50, "dictionary.margin_width")?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ReadingGoals {
    pub daily_minutes: u32,
    pub weekly_books: u32,
    pub enabled: bool,
}

impl Default for ReadingGoals {
    fn default() -> Self {
        ReadingGoals {
            daily_minutes: 30,
            weekly_books: 1,
            enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct GestureSettings {
    pub swipe_left: GestureAction,
    pub swipe_right: GestureAction,
    pub swipe_up: GestureAction,
    pub swipe_down: GestureAction,
    pub double_tap: GestureAction,
    pub long_press: GestureAction,
    pub corner_tap: bool,
    pub pinch_to_zoom: bool,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GestureAction {
    NextPage,
    PreviousPage,
    ToggleBars,
    GoToPage,
    ToggleInverted,
    ToggleDithered,
    None,
}

impl Default for GestureSettings {
    fn default() -> Self {
        GestureSettings {
            swipe_left: GestureAction::NextPage,
            swipe_right: GestureAction::PreviousPage,
            swipe_up: GestureAction::ToggleBars,
            swipe_down: GestureAction::GoToPage,
            double_tap: GestureAction::None,
            long_press: GestureAction::None,
            corner_tap: true,
            pinch_to_zoom: true,
        }
    }
}

impl GestureSettings {
    /// Validates gesture settings
    ///
    /// Currently ensures gesture action values are valid (they're enums so always valid)
    pub fn validate(&self) -> Result<(), Error> {
        // GestureAction is an enum, so values are always valid
        // This method exists for consistency and future validation
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SearchSettings {
    pub enable_regex: bool,
    pub enable_global: bool,
    pub save_history: bool,
    pub history_size: usize,
}

impl Default for SearchSettings {
    fn default() -> Self {
        SearchSettings {
            enable_regex: true,
            enable_global: false,
            save_history: true,
            history_size: 50,
        }
    }
}

impl SearchSettings {
    /// Validates search settings are within acceptable ranges
    pub fn validate(&self) -> Result<(), Error> {
        // History size must be reasonable (0 to 1000 entries)
        validate_range(self.history_size, 0, 1000, "search.history_size")?;

        Ok(())
    }
}

impl Default for ImportSettings {
    fn default() -> Self {
        ImportSettings {
            unshare_trigger: true,
            startup_trigger: true,
            sync_metadata: true,
            metadata_kinds: ["epub", "kepub", "pdf"]
                .iter()
                .map(|k| k.to_string())
                .collect(),
            allowed_kinds: [
                "pdf", "epub", "kepub", "fb2", "fbz", "txt", "xps", "oxps", "mobi", "cbz",
            ]
            .iter()
            .map(|k| k.to_string())
            .collect(),
            enable_duplicates_detection: false,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            selected_library: 0,
            libraries: vec![
                LibrarySettings {
                    name: "On Board".to_string(),
                    path: PathBuf::from(INTERNAL_CARD_ROOT),
                    hooks: vec![Hook {
                        path: PathBuf::from("Articles"),
                        program: PathBuf::from("bin/article_fetcher/article_fetcher"),
                        sort_method: Some(SortMethod::Added),
                        first_column: Some(FirstColumn::TitleAndAuthor),
                        second_column: Some(SecondColumn::Progress),
                    }],
                    ..Default::default()
                },
                LibrarySettings {
                    name: "Removable".to_string(),
                    path: PathBuf::from(EXTERNAL_CARD_ROOT),
                    ..Default::default()
                },
                LibrarySettings {
                    name: "Dropbox".to_string(),
                    path: PathBuf::from("/mnt/onboard/.kobo/dropbox"),
                    ..Default::default()
                },
                LibrarySettings {
                    name: "KePub".to_string(),
                    path: PathBuf::from("/mnt/onboard/.kobo/kepub"),
                    ..Default::default()
                },
            ],
            external_urls_queue: Some(PathBuf::from("bin/article_fetcher/urls.txt")),
            keyboard_layout: "English".to_string(),
            frontlight: true,
            wifi: false,
            inverted: false,
            dark_mode: false,
            theme_settings: ThemeSettings::default(),
            sleep_cover: true,
            sleep_cover_fill: true,
            auto_share: false,
            rotation_lock: None,
            button_scheme: ButtonScheme::Natural,
            auto_suspend: 30.0,
            auto_power_off: 3.0,
            language: "en".to_string(),
            locale: "en-GB".to_string(),
            ui_font: UiFont::default(),
            time_format: "%H:%M".to_string(),
            date_format: "%A, %-d %B %Y".to_string(),
            intermissions: Intermissions {
                suspend: PathBuf::from(LOGO_SPECIAL_PATH),
                power_off: PathBuf::from(LOGO_SPECIAL_PATH),
                share: PathBuf::from(LOGO_SPECIAL_PATH),
            },
            home: HomeSettings::default(),
            reader: ReaderSettings::default(),
            import: ImportSettings::default(),
            dictionary: DictionarySettings::default(),
            sketch: SketchSettings::default(),
            calculator: CalculatorSettings::default(),
            battery: BatterySettings::default(),
            frontlight_levels: LightLevels::default(),
            frontlight_presets: Vec::new(),
            reading_goals: ReadingGoals::default(),
            night_light_schedule: NightLightSchedule::default(),
            gestures: GestureSettings::default(),
            search: SearchSettings::default(),
            reader_presets: Vec::new(),
            external_storage: ExternalStorageSettings::default(),
            cover_editor: CoverEditorSettings::default(),
            plugin_settings: PluginSettings::default(),
            background_sync: BackgroundSyncSettings::default(),
            cloud_sync: CloudSyncSettings::default(),
            thumbnail: ThumbnailSettings::default(),
            opds: OpdsSettings::default(),
            accessibility: AccessibilitySettings::default(),
            calibre: CalibreSettings::default(),
            epub_to_pdf: EpubToPdfSettings::default(),
            goodreads: GoodreadsSettings::default(),
            pocket: PocketSettings::default(),
            cloud_storage: CloudStorageSettings::default(),
            ai: AiSettings::default(),
        }
    }
}

impl Settings {
    /// Validates all settings values are within acceptable ranges
    ///
    /// # Errors
    /// Returns detailed error if any setting is invalid
    ///
    /// # Validation Rules
    /// - font_size: 4.0 to 72.0 points
    /// - auto_suspend: 1.0 to 300.0 minutes (5 hours)
    /// - auto_power_off: 0.5 to 24.0 hours
    /// - selected_library: must be within bounds of libraries vector
    /// - language/locale: non-empty strings with reasonable length
    pub fn validate(&self) -> Result<(), Error> {
        // Validate selected library index
        if self.selected_library >= self.libraries.len() && !self.libraries.is_empty() {
            bail!(
                "selected_library ({}) exceeds number of libraries ({})",
                self.selected_library,
                self.libraries.len()
            );
        }

        // Validate font size bounds
        validate_finite_f32(self.reader.font_size, "font_size", 4.0, 72.0)?;

        // Validate auto_suspend (1 minute to 5 hours)
        validate_finite_f32(self.auto_suspend, "auto_suspend", 1.0, 300.0)?;

        // Validate auto_power_off (0.5 hours to 24 hours)
        validate_finite_f32(self.auto_power_off, "auto_power_off", 0.5, 24.0)?;

        // Validate language string
        validate_string_length(&self.language, "language", 1, 10)?;

        // Validate locale string
        validate_string_length(&self.locale, "locale", 1, 20)?;

        // Validate time_format is not empty
        if self.time_format.is_empty() {
            bail!("time_format cannot be empty");
        }
        validate_string_length(&self.time_format, "time_format", 1, 50)?;

        // Validate date_format is not empty
        if self.date_format.is_empty() {
            bail!("date_format cannot be empty");
        }
        validate_string_length(&self.date_format, "date_format", 1, 100)?;

        // Validate keyboard_layout is not empty
        if self.keyboard_layout.is_empty() {
            bail!("keyboard_layout cannot be empty");
        }
        validate_string_length(&self.keyboard_layout, "keyboard_layout", 1, 50)?;

        // Validate sub-settings
        self.reader
            .validate()
            .context("reader settings validation failed")?;
        self.dictionary
            .validate()
            .context("dictionary settings validation failed")?;
        self.gestures
            .validate()
            .context("gesture settings validation failed")?;
        self.search
            .validate()
            .context("search settings validation failed")?;
        self.home
            .validate()
            .context("home settings validation failed")?;
        self.battery
            .validate()
            .context("battery settings validation failed")?;
        self.night_light_schedule
            .validate()
            .context("night light schedule validation failed")?;
        self.cover_editor
            .validate()
            .context("cover editor settings validation failed")?;
        self.thumbnail
            .validate()
            .context("thumbnail settings validation failed")?;
        self.opds
            .validate()
            .context("opds settings validation failed")?;
        self.accessibility
            .validate()
            .context("accessibility settings validation failed")?;

        // Validate all library settings
        for (i, lib) in self.libraries.iter().enumerate() {
            lib.validate()
                .with_context(|| format!("library[{}] validation failed", i))?;
        }

        // Validate all reader presets
        for (i, preset) in self.reader_presets.iter().enumerate() {
            preset
                .validate()
                .with_context(|| format!("reader_preset[{}] validation failed", i))?;
        }

        Ok(())
    }
}
