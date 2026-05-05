//! Accessibility Module
//!
#![allow(clippy::redundant_pattern_matching)]

//! This module provides accessibility features for Plato:
//! - Bionic Reading: Bold first half of words for faster reading
//! - Auto-Pace: Automatic page turning with adjustable WPM
//! - Dyslexia-friendly fonts: OpenDyslexic, Atkinson Hyperlegible, Lexend
//! - High contrast and color blindness modes.

pub mod auto_pace;
pub mod bionic_reading;

// Re-exports
pub use auto_pace::{calculate_reading_time, AutoPace};
pub use bionic_reading::{apply_bionic_reading, process_bionic_text, split_word_bionic};

/// List of bundled accessibility fonts
pub const ACCESSIBILITY_FONTS: &[(&str, &str)] = &[
    ("opendyslexic", "OpenDyslexic-Regular.otf"),
    ("atkinson", "AtkinsonHyperlegible-Regular.ttf"),
    ("lexend", "Lexend-Regular.ttf"),
];

/// Check if a font family name is an accessibility font
pub fn is_accessibility_font(family: &str) -> bool {
    ACCESSIBILITY_FONTS
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case(family))
}

/// Get the font filename for an accessibility font family
pub fn get_accessibility_font(family: &str) -> Option<&'static str> {
    ACCESSIBILITY_FONTS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(family))
        .map(|(_, filename)| *filename)
}
