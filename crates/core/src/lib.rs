#[macro_use]
pub mod geom;

pub mod battery;
pub mod buffer_pool;
pub mod color;
pub mod consts;
pub mod context;
pub mod error;
pub use error::{into_plato_err, PlatoError};
pub mod cover_editor;
pub mod device;
pub mod image_formats;
pub use device::{Device, FrontlightKind, KoboDevice, Model, Orientation, CURRENT_DEVICE};
pub use mobile_optimizations::{
    animation_config, is_mobile_platform, memory_config, network_config, power_config,
    recommended_io_buffer_size, recommended_thread_pool_size, storage_config, AnimationConfig,
    MemoryConfig, NetworkConfig, PowerConfig, StorageConfig, TouchConfig,
};
pub use mobile_theme::{mobile_theme_mode, set_mobile_theme_mode, MobileThemeMode};
mod dictionary;
pub mod document;
pub mod eink;
pub mod font;
pub mod framebuffer;
pub mod frontlight;
pub mod gesture;
pub mod helpers;
pub mod i18n;
pub mod input;
pub mod library;
pub mod lightsensor;
pub mod metadata;
pub mod mobile_optimizations;
pub mod mobile_theme;
pub mod opds;
pub mod plugin;
pub mod reading_time;
pub mod rtc;
pub mod settings;
pub mod sync;
pub mod theme;
pub mod thumbnail;
mod unit;
pub mod update;
pub mod validation;
pub mod view;

// TTS module - available on supported platforms (Android, Desktop)
// Not available on Kobo e-readers (no audio hardware)
pub mod tts;

// Desktop TTS via 'tts' crate (Linux/macOS/Windows)
#[cfg(feature = "tts")]
pub mod tts_desktop;

// Android TTS via JNI (requires tts-android feature - only works on Android)
#[cfg(feature = "tts-android")]
pub mod tts_android;

pub use reading_time::{
    estimate_from_page_count, estimate_from_word_count, format_duration, ReadingSpeed,
};

/// Mock implementations for testing
///
/// Provides mock implementations of core traits (Framebuffer, Battery,
/// Frontlight, LightSensor, Document) for unit testing without hardware.
pub mod test_mocks;

pub use anyhow;
pub use chrono;
pub use globset;
pub use image;
pub use png;
pub use rand_core;
pub use rand_xoshiro;
pub use rustc_hash;
pub use serde;
pub use serde_json;
pub use walkdir;
