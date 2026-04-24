//! Centralized constants module for shared values across Plato
//!
//! This module provides a Single Source of Truth for constants that are used
//! across multiple modules, following AGENTS.md rules.
//!
//! # Design Principles
//! - Define constants in the module that owns the concept
//! - Re-export from here for convenience, but prefer importing from original modules
//! - Document the canonical source for each constant group
//! - Never duplicate literal values - always reference a named constant

// Re-export from unit.rs - canonical source for measurement constants
pub use crate::unit::{
    BASE_DPI, CENTIMETERS_PER_INCH, DEFAULT_DPI, MILLIMETERS_PER_INCH, PICAS_PER_INCH,
    POINTS_PER_INCH,
};

// Re-export from settings/defaults.rs - canonical source for default settings values
pub use crate::settings::{
    DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT, DEFAULT_MARGIN_WIDTH,
    DEFAULT_TEXT_ALIGN,
};

/// UI rendering constants based on 300 DPI reference.
///
/// These constants are used for consistent UI element sizing across the application.
/// All values are specified at the base DPI (300) and scaled appropriately.
pub mod ui {
    use std::time::Duration;

    /// Base DPI reference for all UI constants (300 DPI)
    pub const UI_BASE_DPI: f32 = 300.0;

    // Border thicknesses in pixels, at 300 DPI.
    pub const THICKNESS_SMALL: f32 = 1.5;
    pub const THICKNESS_MEDIUM: f32 = 2.0;
    pub const THICKNESS_LARGE: f32 = 3.0;

    // Border radii in pixels, at 300 DPI.
    pub const BORDER_RADIUS_SMALL: f32 = 6.0;
    pub const BORDER_RADIUS_MEDIUM: f32 = 9.0;
    pub const BORDER_RADIUS_LARGE: f32 = 12.0;

    // Big and small bar heights in pixels, at 300 DPI.
    // On the *Aura ONE*, the height is exactly `2 * sb + 10 * bb`.
    pub const SMALL_BAR_HEIGHT: f32 = 121.0;
    pub const BIG_BAR_HEIGHT: f32 = 163.0;

    /// Delay before closing ignition
    pub const CLOSE_IGNITION_DELAY: Duration = Duration::from_millis(150);

    /// Maximum delay for update completion
    pub const MAX_UPDATE_DELAY: Duration = Duration::from_millis(600);
}

/// System and file system constants
pub mod system {
    /// Maximum path length for file operations
    pub const MAX_PATH_LENGTH: usize = 4096;

    /// Maximum file name length
    pub const MAX_FILENAME_LENGTH: usize = 255;

    /// Page cache size in megabytes for standard Kobo devices (256-512MB RAM)
    pub const PAGE_CACHE_SIZE_MB: usize = 20;

    /// Page cache size for Elipsa devices (1GB RAM)
    pub const ELIPSA_PAGE_CACHE_SIZE_MB: usize = 40;

    /// Page cache size for Android devices (12GB+ RAM)
    pub const ANDROID_PAGE_CACHE_SIZE_MB: usize = 100;

    /// Preload ahead pages count for standard devices
    pub const PRELOAD_AHEAD_PAGES: usize = 2;

    /// Preload ahead pages count for Elipsa (more RAM allows more preloading)
    pub const ELIPSA_PRELOAD_AHEAD_PAGES: usize = 3;

    /// Preload ahead pages count for Android (abundant RAM)
    pub const ANDROID_PRELOAD_AHEAD_PAGES: usize = 5;

    /// Preload behind pages count
    pub const PRELOAD_BEHIND_PAGES: usize = 1;

    /// Preload behind pages count for Elipsa
    pub const ELIPSA_PRELOAD_BEHIND_PAGES: usize = 2;

    /// Preload behind pages count for Android
    pub const ANDROID_PRELOAD_BEHIND_PAGES: usize = 3;
}

/// Buffer pool size constants
pub mod buffer_pool {
    /// Thumbnail buffer size for standard Kobo devices (1MB)
    pub const THUMBNAIL_BUFFER_SIZE: usize = 1024 * 1024;

    /// Thumbnail buffer size for Elipsa devices (2MB)
    pub const ELIPSA_THUMBNAIL_BUFFER_SIZE: usize = 2 * 1024 * 1024;

    /// Thumbnail buffer size for Android devices (4MB)
    pub const ANDROID_THUMBNAIL_BUFFER_SIZE: usize = 4 * 1024 * 1024;

    /// Document buffer size for standard Kobo devices (4MB)
    pub const DOCUMENT_BUFFER_SIZE: usize = 4 * 1024 * 1024;

    /// Document buffer size for Elipsa devices (8MB)
    pub const ELIPSA_DOCUMENT_BUFFER_SIZE: usize = 8 * 1024 * 1024;

    /// Document buffer size for Android devices (16MB)
    pub const ANDROID_DOCUMENT_BUFFER_SIZE: usize = 16 * 1024 * 1024;
}

/// PDF manipulation constants
pub mod pdf {
    /// Maximum file size in MB for PDF operations
    pub const MAX_FILE_SIZE_MB: u64 = 50;

    /// Warning threshold for file size in MB
    pub const WARNING_FILE_SIZE_MB: u64 = 30;

    /// Maximum pages warning threshold
    pub const MAX_PAGES_WARNING: usize = 300;

    /// Hard limit for maximum pages
    pub const MAX_PAGES_HARD_LIMIT: usize = 500;

    /// Chunk size for page processing
    pub const CHUNK_SIZE: usize = 10;

    /// Kobo memory limit in MB
    pub const KOBO_MEMORY_LIMIT_MB: u64 = 256;
}

/// Gesture recognition constants
pub mod gesture {
    use std::time::Duration;

    /// Tap jitter tolerance in millimeters
    pub const TAP_JITTER_MM: f32 = 6.0;

    /// Hold jitter tolerance in millimeters
    pub const HOLD_JITTER_MM: f32 = 1.5;

    /// Short hold delay duration
    pub const HOLD_DELAY_SHORT: Duration = Duration::from_millis(666);

    /// Long hold delay duration
    pub const HOLD_DELAY_LONG: Duration = Duration::from_millis(1333);
}

/// Frontlight hardware constants
pub mod frontlight {
    /// Frontlight interface path
    pub const FRONTLIGHT_INTERFACE: &str = "/sys/class/backlight";

    /// Aura ONE white LED
    pub const FRONTLIGHT_WHITE_A: &str = "lm3630a_led1b";

    /// Aura ONE red LED
    pub const FRONTLIGHT_RED_A: &str = "lm3630a_led1a";

    /// Aura ONE green LED
    pub const FRONTLIGHT_GREEN_A: &str = "lm3630a_ledb";

    /// Aura H₂O Edition 2 white LED
    pub const FRONTLIGHT_WHITE_B: &str = "lm3630a_ledb";

    /// Aura H₂O Edition 2 orange LED
    pub const FRONTLIGHT_ORANGE_B: &str = "lm3630a_leda";

    /// Brightness value file name
    pub const FRONTLIGHT_VALUE: &str = "brightness";

    /// Max brightness file name
    pub const FRONTLIGHT_MAX_VALUE: &str = "max_brightness";

    /// Power control file name
    pub const FRONTLIGHT_POWER: &str = "bl_power";

    /// Power on value
    pub const FRONTLIGHT_POWER_ON: i16 = 31;

    /// Power off value
    pub const FRONTLIGHT_POWER_OFF: i16 = 0;
}

/// Font and text rendering constants
pub mod font {
    /// Default minimum font size in points
    pub const DEFAULT_MIN_FONT_SIZE: f32 = 4.0;

    /// Maximum reasonable font size in points
    pub const MAX_FONT_SIZE: f32 = 72.0;

    /// Minimum reasonable font size in points
    pub const MIN_FONT_SIZE: f32 = 4.0;
}

/// Library and metadata constants
pub mod library {
    /// Metadata filename
    pub const METADATA_FILENAME: &str = ".metadata.json";

    /// FAT32 epoch filename
    pub const FAT32_EPOCH_FILENAME: &str = ".fat32-epoch";

    /// Reading states directory name
    pub const READING_STATES_DIRNAME: &str = ".reading-states";

    /// Thumbnail previews directory name
    pub const THUMBNAIL_PREVIEWS_DIRNAME: &str = ".thumbnail-previews";
}

/// Thumbnail generation constants
pub mod thumbnail {
    /// Default thumbnail width in pixels
    pub const THUMBNAIL_WIDTH: u32 = 240;

    /// Default thumbnail height in pixels
    pub const THUMBNAIL_HEIGHT: u32 = 320;

    /// Default number of worker threads for standard Kobo devices (256-512MB RAM)
    pub const DEFAULT_WORKER_COUNT: usize = 2;

    /// Worker threads for Elipsa devices (1GB RAM, 4-core Allwinner B300)
    pub const ELIPSA_WORKER_COUNT: usize = 3;

    /// Worker threads for Android devices (abundant RAM, 8+ cores)
    pub const ANDROID_WORKER_COUNT: usize = 4;

    /// Default cache size for standard Kobo devices
    pub const DEFAULT_CACHE_SIZE: usize = 20;

    /// Cache size for Elipsa devices (1GB RAM allows larger cache)
    pub const ELIPSA_CACHE_SIZE: usize = 35;

    /// Cache size for Android devices (abundant RAM)
    pub const ANDROID_CACHE_SIZE: usize = 50;
}

/// Input and interaction constants
pub mod input {
    /// Input history size
    pub const INPUT_HISTORY_SIZE: usize = 32;

    /// Tap jitter tolerance in millimeters for e-ink devices
    pub const EINK_TAP_JITTER_MM: f32 = 6.0;

    /// Tap jitter tolerance in millimeters for mobile devices (tighter)
    pub const MOBILE_TAP_JITTER_MM: f32 = 4.0;

    /// Hold delay for e-ink devices (conservative)
    pub const EINK_HOLD_DELAY_MS: u64 = 666;

    /// Hold delay for mobile devices (more responsive)
    pub const MOBILE_HOLD_DELAY_MS: u64 = 400;

    /// Touch polling rate for e-ink (60Hz)
    pub const EINK_TOUCH_POLL_RATE: u32 = 60;

    /// Touch polling rate for mobile (120Hz)
    pub const MOBILE_TOUCH_POLL_RATE: u32 = 120;
}

/// Mobile platform constants
pub mod mobile {
    use std::time::Duration;

    /// Target FPS for mobile animations (OnePlus Nord 2 5G: 90Hz)
    pub const MOBILE_TARGET_FPS: u32 = 90;

    /// Animation duration multiplier for mobile (faster animations)
    pub const MOBILE_ANIMATION_SPEED: f32 = 0.7;

    /// Sync interval for mobile background sync (5 minutes)
    pub const MOBILE_SYNC_INTERVAL: Duration = Duration::from_secs(300);

    /// Conservative sync interval for e-ink (1 hour)
    pub const EINK_SYNC_INTERVAL: Duration = Duration::from_secs(3600);

    /// Max concurrent downloads on mobile
    pub const MOBILE_MAX_CONCURRENT_DOWNLOADS: usize = 4;

    /// Max concurrent downloads on e-ink (battery conservative)
    pub const EINK_MAX_CONCURRENT_DOWNLOADS: usize = 1;

    /// Thumbnail quality for mobile (higher, better displays)
    pub const MOBILE_THUMBNAIL_QUALITY: u8 = 85;

    /// Thumbnail quality for e-ink (lower, sufficient)
    pub const EINK_THUMBNAIL_QUALITY: u8 = 75;

    /// Image cache size on mobile (300MB)
    pub const MOBILE_IMAGE_CACHE_MB: usize = 300;

    /// Image cache size on e-ink (20MB)
    pub const EINK_IMAGE_CACHE_MB: usize = 20;

    /// I/O buffer size for UFS 3.1 (mobile)
    pub const MOBILE_IO_BUFFER_SIZE: usize = 128 * 1024;

    /// I/O buffer size for eMMC (e-ink)
    pub const EINK_IO_BUFFER_SIZE: usize = 32 * 1024;

    /// Heap usage percentage on mobile (75%)
    pub const MOBILE_HEAP_PERCENT: u8 = 75;

    /// Heap usage percentage on e-ink (50%)
    pub const EINK_HEAP_PERCENT: u8 = 50;
}

/// Settings and configuration constants
pub mod settings {
    /// Settings file path (relative for Kobo, XDG config for desktop)
    pub const SETTINGS_PATH: &str = "Settings.toml";

    /// Default font path
    pub const DEFAULT_FONT_PATH: &str = "/mnt/onboard/fonts";

    /// Internal storage root path
    pub const INTERNAL_CARD_ROOT: &str = "/mnt/onboard";

    /// External storage (SD card) root path
    pub const EXTERNAL_CARD_ROOT: &str = "/mnt/sd";

    /// Special path prefix for logo
    pub const LOGO_SPECIAL_PATH: &str = "logo:";

    /// Special path prefix for cover
    pub const COVER_SPECIAL_PATH: &str = "cover:";
}

/// Desktop Linux (XDG Base Directory) paths
pub mod desktop {
    /// System data directory for installed resources
    pub const SYSTEM_DATA_DIR: &str = "/usr/share/plato";

    /// XDG config directory name
    pub const XDG_CONFIG_DIRNAME: &str = "plato";

    /// XDG data directory name
    pub const XDG_DATA_DIRNAME: &str = "plato";

    /// Desktop settings file path (within XDG config)
    pub const DESKTOP_SETTINGS_FILENAME: &str = "Settings.toml";
}

/// HTML and document rendering constants
pub mod html {
    /// Default document width for HTML engine
    pub const DEFAULT_WIDTH: u32 = 1404;

    /// Default document height for HTML engine
    pub const DEFAULT_HEIGHT: u32 = 1872;

    /// Hyphen penalty value for line breaking
    pub const HYPHEN_PENALTY: i32 = 50;

    /// Stretch tolerance for line justification
    pub const STRETCH_TOLERANCE: f32 = 1.26;
}

/// Time-related constants
pub mod time {
    use std::time::Duration;

    /// Standard timeout for operations
    pub const STANDARD_TIMEOUT: Duration = Duration::from_secs(30);

    /// Short delay for UI feedback
    pub const SHORT_DELAY: Duration = Duration::from_millis(100);

    /// Medium delay for transitions
    pub const MEDIUM_DELAY: Duration = Duration::from_millis(300);
}
