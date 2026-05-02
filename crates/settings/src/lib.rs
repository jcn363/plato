//! Plato Settings Module
//!
//! This crate provides settings management for Plato.

pub use plato_core::settings::{
    guess_frontlight, load_settings, save_settings, AccessibilitySettings, AiSettings,
    CalibreSettings, CloudStorageSettings, ConfigManager, DictionarySettings, EpubToPdfSettings,
    GestureAction, GestureSettings, GoodreadsSettings, ImportSettings, LightPreset, PocketSettings,
    ReaderPreset, ReadingGoals, SearchSettings, Settings, ThemeMode, ThemeSchedule, ThemeSettings,
    TimeOfDay,
};
