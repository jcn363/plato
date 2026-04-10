mod defaults;
mod display;
mod features;
mod interface;
mod library;
mod preset;
mod reading;
mod tools;

use crate::frontlight::LightLevels;
use crate::metadata::{SortMethod, TextAlign};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub use self::preset::{guess_frontlight, LightPreset};
pub use defaults::*;
pub use display::*;
pub use features::*;
pub use interface::*;
pub use library::*;
pub use reading::*;
pub use tools::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Settings {
    pub selected_library: usize,
    pub keyboard_layout: String,
    pub frontlight: bool,
    pub wifi: bool,
    pub inverted: bool,
    pub dark_mode: bool,
    pub sleep_cover: bool,
    pub sleep_cover_fill: bool,
    pub auto_share: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_lock: Option<RotationLock>,
    pub button_scheme: ButtonScheme,
    pub auto_suspend: f32,
    pub auto_power_off: f32,
    pub language: String,
    pub locale: String,
    pub ui_font: UiFont,
    pub time_format: String,
    pub date_format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_urls_queue: Option<PathBuf>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
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

impl Default for ImportSettings {
    fn default() -> Self {
        ImportSettings {
            unshare_trigger: true,
            startup_trigger: true,
            sync_metadata: true,
            metadata_kinds: ["epub", "pdf"].iter().map(|k| k.to_string()).collect(),
            allowed_kinds: [
                "pdf", "epub", "fb2", "fbz", "txt", "xps", "oxps", "mobi", "cbz",
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
        }
    }
}
