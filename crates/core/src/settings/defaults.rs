//! Default settings values and constants
//!
//! This module provides default values for settings. Where possible,
//! constants are re-exported from the canonical source in `crate::consts`
//! per Single Source of Truth rule.

use crate::metadata::TextAlign;

// Re-export path and file constants from canonical source in consts::settings
pub use crate::consts::settings::{
    COVER_SPECIAL_PATH, DEFAULT_FONT_PATH, EXTERNAL_CARD_ROOT, INTERNAL_CARD_ROOT,
    LOGO_SPECIAL_PATH, SETTINGS_PATH,
};

// Re-export HTML/rendering constants from canonical source in consts::html
pub use crate::consts::html::{HYPHEN_PENALTY, STRETCH_TOLERANCE};

pub const PLATO_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default font size in points
pub const DEFAULT_FONT_SIZE: f32 = 11.0;

/// Default dictionary font size in points
pub const DEFAULT_DICTIONARY_FONT_SIZE: f32 = 11.0;

/// Default margin width in pixels (at 300 DPI)
pub const DEFAULT_MARGIN_WIDTH: i32 = 8;

/// Default line height as a multiplier (1.2 = 120% of font size)
pub const DEFAULT_LINE_HEIGHT: f32 = 1.2;

/// Default font family name
pub const DEFAULT_FONT_FAMILY: &str = "Libertinus Serif";

/// Default text alignment
pub const DEFAULT_TEXT_ALIGN: TextAlign = TextAlign::Left;

/// File kinds that should use dithered rendering
pub const DEFAULT_DITHERED_KINDS: &[&str] = &["cbz", "jpg", "png", "jpeg"];
