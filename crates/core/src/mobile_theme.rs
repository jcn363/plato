//! Mobile-specific colorful themes for Android/iOS platforms
//!
//! This module provides vibrant color themes optimized for OLED/LCD displays
//! on mobile devices. Unlike e-ink optimized themes (grayscale), these
//! themes use full color for accent elements while maintaining readability.
//!
//! Only active on mobile platforms (Android/iOS), detected at runtime.

use crate::color::Color;
use std::sync::LazyLock;

/// Check if running on a mobile platform (Android or iOS)
#[inline]
pub fn is_mobile_platform() -> bool {
    std::env::var("ANDROID_ROOT").is_ok() || std::env::var("IPHONE_SIMULATOR_ROOT").is_ok()
}

/// Mobile theme modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobileThemeMode {
    Light,
    Dark,
    System, // Follows system theme
}

static MOBILE_THEME: LazyLock<std::sync::Mutex<MobileThemeMode>> =
    LazyLock::new(|| std::sync::Mutex::new(MobileThemeMode::System));

/// Get current mobile theme mode
#[inline]
pub fn mobile_theme_mode() -> MobileThemeMode {
    *MOBILE_THEME.lock().expect("MOBILE_THEME lock poisoned")
}

/// Set mobile theme mode
#[inline]
pub fn set_mobile_theme_mode(mode: MobileThemeMode) {
    *MOBILE_THEME.lock().expect("MOBILE_THEME lock poisoned") = mode;
}

// ============================================================================
// Colorful accent colors for mobile themes (OLED optimized)
// ============================================================================

/// Primary accent color - vibrant blue
pub const ACCENT_BLUE: Color = Color::Rgb(66, 133, 244);

/// Secondary accent - teal/cyan
pub const ACCENT_TEAL: Color = Color::Rgb(24, 191, 200);

/// Success color - vibrant green
pub const ACCENT_GREEN: Color = Color::Rgb(52, 199, 89);

/// Warning color - amber/orange
pub const ACCENT_AMBER: Color = Color::Rgb(255, 171, 0);

/// Error color - coral red
pub const ACCENT_RED: Color = Color::Rgb(255, 82, 82);

/// Purple accent
pub const ACCENT_PURPLE: Color = Color::Rgb(156, 39, 176);

/// Pink accent
pub const ACCENT_PINK: Color = Color::Rgb(233, 30, 99);

// ============================================================================
// Light theme colors (Mobile)
// ============================================================================

/// Light theme background - pure white for OLED
pub const MOBILE_LIGHT_BG: Color = Color::Rgb(255, 255, 255);

/// Light theme surface (cards, panels) - off-white
pub const MOBILE_LIGHT_SURFACE: Color = Color::Rgb(250, 250, 250);

/// Light theme primary text - near black
pub const MOBILE_LIGHT_TEXT_PRIMARY: Color = Color::Rgb(33, 33, 33);

/// Light theme secondary text - dark gray
pub const MOBILE_LIGHT_TEXT_SECONDARY: Color = Color::Rgb(97, 97, 97);

/// Light theme divider/border
pub const MOBILE_LIGHT_DIVIDER: Color = Color::Rgb(224, 224, 224);

/// Light theme selected/highlight background
pub const MOBILE_LIGHT_SELECTED: Color = Color::Rgb(232, 240, 254);

// ============================================================================
// Dark theme colors (Mobile OLED optimized)
// ============================================================================

/// Dark theme background - true black for OLED power savings
pub const MOBILE_DARK_BG: Color = Color::Rgb(0, 0, 0);

/// Dark theme surface (cards, panels) - dark gray
pub const MOBILE_DARK_SURFACE: Color = Color::Rgb(18, 18, 18);

/// Dark theme elevated surface - slightly lighter
pub const MOBILE_DARK_SURFACE_ELEVATED: Color = Color::Rgb(30, 30, 30);

/// Dark theme primary text - white
pub const MOBILE_DARK_TEXT_PRIMARY: Color = Color::Rgb(255, 255, 255);

/// Dark theme secondary text - light gray
pub const MOBILE_DARK_TEXT_SECONDARY: Color = Color::Rgb(176, 176, 176);

/// Dark theme divider/border
pub const MOBILE_DARK_DIVIDER: Color = Color::Rgb(48, 48, 48);

/// Dark theme selected/highlight background
pub const MOBILE_DARK_SELECTED: Color = Color::Rgb(32, 42, 64);

// ============================================================================
// Theme accessor functions
// ============================================================================

/// Get background color for current mobile theme
#[inline]
pub fn mobile_background() -> Color {
    if is_mobile_dark_mode() {
        MOBILE_DARK_BG
    } else {
        MOBILE_LIGHT_BG
    }
}

/// Get surface color for current mobile theme
#[inline]
pub fn mobile_surface() -> Color {
    if is_mobile_dark_mode() {
        MOBILE_DARK_SURFACE
    } else {
        MOBILE_LIGHT_SURFACE
    }
}

/// Get primary text color for current mobile theme
#[inline]
pub fn mobile_text_primary() -> Color {
    if is_mobile_dark_mode() {
        MOBILE_DARK_TEXT_PRIMARY
    } else {
        MOBILE_LIGHT_TEXT_PRIMARY
    }
}

/// Get secondary text color for current mobile theme
#[inline]
pub fn mobile_text_secondary() -> Color {
    if is_mobile_dark_mode() {
        MOBILE_DARK_TEXT_SECONDARY
    } else {
        MOBILE_LIGHT_TEXT_SECONDARY
    }
}

/// Get divider color for current mobile theme
#[inline]
pub fn mobile_divider() -> Color {
    if is_mobile_dark_mode() {
        MOBILE_DARK_DIVIDER
    } else {
        MOBILE_LIGHT_DIVIDER
    }
}

/// Get selected/highlight background for current mobile theme
#[inline]
pub fn mobile_selected_bg() -> Color {
    if is_mobile_dark_mode() {
        MOBILE_DARK_SELECTED
    } else {
        MOBILE_LIGHT_SELECTED
    }
}

/// Check if mobile dark mode is active
#[inline]
pub fn is_mobile_dark_mode() -> bool {
    if !is_mobile_platform() {
        return false;
    }

    match mobile_theme_mode() {
        MobileThemeMode::Dark => true,
        MobileThemeMode::Light => false,
        MobileThemeMode::System => {
            // Check system dark mode via environment or fallback to time-based
            std::env::var("SYSTEM_DARK_MODE")
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false)
        }
    }
}

// ============================================================================
// Accent color getters (theme-aware)
// ============================================================================

/// Get accent color with appropriate opacity for current theme
#[inline]
pub fn mobile_accent_blue() -> Color {
    ACCENT_BLUE
}

#[inline]
pub fn mobile_accent_green() -> Color {
    ACCENT_GREEN
}

#[inline]
pub fn mobile_accent_amber() -> Color {
    ACCENT_AMBER
}

#[inline]
pub fn mobile_accent_red() -> Color {
    ACCENT_RED
}

#[inline]
pub fn mobile_accent_purple() -> Color {
    ACCENT_PURPLE
}

#[inline]
pub fn mobile_accent_teal() -> Color {
    ACCENT_TEAL
}

// ============================================================================
// Status bar/navigation colors for mobile
// ============================================================================

/// Status bar background color
#[inline]
pub fn mobile_status_bar_bg() -> Color {
    if is_mobile_dark_mode() {
        MOBILE_DARK_BG
    } else {
        MOBILE_LIGHT_BG
    }
}

/// Navigation bar background color
#[inline]
pub fn mobile_navigation_bar_bg() -> Color {
    if is_mobile_dark_mode() {
        MOBILE_DARK_SURFACE
    } else {
        MOBILE_LIGHT_SURFACE
    }
}

// ============================================================================
// Helper functions for color blending
// ============================================================================

/// Blend color with surface based on elevation (simulating Material Design elevation)
#[inline]
pub fn mobile_elevated_surface(elevation: u8) -> Color {
    let base = mobile_surface();
    let overlay = if is_mobile_dark_mode() {
        Color::Rgb(255, 255, 255) // White overlay for dark theme elevation
    } else {
        Color::Rgb(0, 0, 0) // Black overlay for light theme shadows
    };

    // Elevation 0-5 maps to alpha 0-30
    let alpha = (elevation.min(5) as f32 * 6.0) / 255.0;
    base.lerp(overlay, alpha)
}

/// Get ripple effect color for touch feedback
#[inline]
pub fn mobile_ripple_color() -> Color {
    if is_mobile_dark_mode() {
        Color::Rgb(255, 255, 255)
    } else {
        Color::Rgb(0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_theme_colors_exist() {
        // Verify all color constants are defined
        let _ = MOBILE_LIGHT_BG;
        let _ = MOBILE_DARK_BG;
        let _ = ACCENT_BLUE;
        let _ = ACCENT_GREEN;
    }

    #[test]
    fn test_mobile_theme_mode_default() {
        let mode = mobile_theme_mode();
        assert!(matches!(mode, MobileThemeMode::System));
    }

    #[test]
    fn test_set_mobile_theme_mode() {
        set_mobile_theme_mode(MobileThemeMode::Dark);
        assert!(matches!(mobile_theme_mode(), MobileThemeMode::Dark));

        set_mobile_theme_mode(MobileThemeMode::Light);
        assert!(matches!(mobile_theme_mode(), MobileThemeMode::Light));

        // Reset to default
        set_mobile_theme_mode(MobileThemeMode::System);
    }

    #[test]
    fn test_color_lerp_for_elevation() {
        let elevated = mobile_elevated_surface(2);
        // Should return a valid color, not panic
        match elevated {
            Color::Rgb(_, _, _) | Color::Gray(_) => (),
        }
    }
}
