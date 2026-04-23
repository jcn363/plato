use crate::geom::lerp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Color {
    Gray(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    #[inline]
    pub fn gray(&self) -> u8 {
        match *self {
            Color::Gray(level) => level,
            Color::Rgb(red, green, blue) => rgb_to_grayscale_scalar(red, green, blue),
        }
    }

    #[inline]
    pub fn rgb(&self) -> [u8; 3] {
        match *self {
            Color::Gray(level) => [level; 3],
            Color::Rgb(red, green, blue) => [red, green, blue],
        }
    }

    #[inline]
    pub fn from_rgb(rgb: &[u8]) -> Color {
        Color::Rgb(rgb[0], rgb[1], rgb[2])
    }

    #[inline]
    pub fn apply<F>(&self, f: F) -> Color
    where
        F: Fn(u8) -> u8,
    {
        match *self {
            Color::Gray(level) => Color::Gray(f(level)),
            Color::Rgb(red, green, blue) => Color::Rgb(f(red), f(green), f(blue)),
        }
    }

    #[inline]
    pub fn lerp(&self, color: Color, alpha: f32) -> Color {
        match (*self, color) {
            (Color::Gray(l1), Color::Gray(l2)) => {
                Color::Gray(lerp(l1 as f32, l2 as f32, alpha) as u8)
            }
            (Color::Rgb(red, green, blue), Color::Gray(level)) => Color::Rgb(
                lerp(red as f32, level as f32, alpha) as u8,
                lerp(green as f32, level as f32, alpha) as u8,
                lerp(blue as f32, level as f32, alpha) as u8,
            ),
            (Color::Gray(level), Color::Rgb(red, green, blue)) => Color::Rgb(
                lerp(level as f32, red as f32, alpha) as u8,
                lerp(level as f32, green as f32, alpha) as u8,
                lerp(level as f32, blue as f32, alpha) as u8,
            ),
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => Color::Rgb(
                lerp(r1 as f32, r2 as f32, alpha) as u8,
                lerp(g1 as f32, g2 as f32, alpha) as u8,
                lerp(b1 as f32, b2 as f32, alpha) as u8,
            ),
        }
    }

    #[inline]
    pub fn invert(&mut self) {
        match self {
            Color::Gray(level) => *level = 255 - *level,
            Color::Rgb(red, green, blue) => {
                *red = 255 - *red;
                *green = 255 - *green;
                *blue = 255 - *blue;
            }
        }
    }

    #[inline]
    pub fn shift(&mut self, drift: u8) {
        match self {
            Color::Gray(level) => *level = level.saturating_sub(drift),
            Color::Rgb(red, green, blue) => {
                *red = red.saturating_sub(drift);
                *green = green.saturating_sub(drift);
                *blue = blue.saturating_sub(drift);
            }
        }
    }
}

/// Scalar RGB to grayscale conversion
#[inline]
fn rgb_to_grayscale_scalar(red: u8, green: u8, blue: u8) -> u8 {
    (red as f32 * 0.2126 + green as f32 * 0.7152 + blue as f32 * 0.0722) as u8
}

/// Bulk RGB to grayscale conversion using luminance formula
///
/// Converts RGB pixel data to grayscale using the ITU-R BT.709 luminance formula:
/// Y = 0.2126 * R + 0.7152 * G + 0.0722 * B
///
/// # Arguments
/// * `rgb_data` - Slice of RGB data in RGBRGB... format (3 bytes per pixel)
///
/// # Returns
/// A vector of grayscale values (1 byte per pixel)
pub fn rgb_to_grayscale_bulk(rgb_data: &[u8]) -> Vec<u8> {
    let len = rgb_data.len() / 3;
    let mut result = Vec::with_capacity(len);
    for chunk in rgb_data.chunks_exact(3) {
        if chunk.len() == 3 {
            result.push(rgb_to_grayscale_scalar(chunk[0], chunk[1], chunk[2]));
        }
    }
    result
}

macro_rules! gray {
    ($a:expr) => {
        $crate::color::Color::Gray($a)
    };
}

pub const GRAY00: Color = gray!(0x00);
pub const GRAY01: Color = gray!(0x11);
pub const GRAY02: Color = gray!(0x22);
pub const GRAY03: Color = gray!(0x33);
pub const GRAY04: Color = gray!(0x44);
pub const GRAY05: Color = gray!(0x55);
pub const GRAY06: Color = gray!(0x66);
pub const GRAY07: Color = gray!(0x77);
pub const GRAY08: Color = gray!(0x88);
pub const GRAY09: Color = gray!(0x99);
pub const GRAY10: Color = gray!(0xAA);
pub const GRAY11: Color = gray!(0xBB);
pub const GRAY12: Color = gray!(0xCC);
pub const GRAY13: Color = gray!(0xDD);
pub const GRAY14: Color = gray!(0xEE);
pub const GRAY15: Color = gray!(0xFF);
pub const GRAYF4: Color = gray!(244);
pub const GRAY5C: Color = gray!(92);

pub const BLACK: Color = GRAY00;
pub const WHITE: Color = GRAY15;

pub const TEXT_NORMAL: [Color; 3] = [WHITE, BLACK, GRAY05];
pub const TEXT_BUMP_SMALL: [Color; 3] = [GRAY13, BLACK, GRAY07];
pub const TEXT_BUMP_LARGE: [Color; 3] = [GRAY11, BLACK, BLACK];

pub const TEXT_INVERTED_SOFT: [Color; 3] = [GRAY05, WHITE, WHITE];
pub const TEXT_INVERTED_HARD: [Color; 3] = [BLACK, WHITE, GRAY09];

pub const SEPARATOR_NORMAL: Color = GRAY10;
pub const SEPARATOR_STRONG: Color = GRAY07;

pub const KEYBOARD_BG: Color = GRAY11;
pub const BATTERY_FILL: Color = GRAY12;
pub const READING_PROGRESS: Color = GRAY07;

pub const PROGRESS_FULL: Color = GRAY05;
pub const PROGRESS_EMPTY: Color = GRAY13;
pub const PROGRESS_VALUE: Color = GRAY06;

pub const DARK_BACKGROUND: Color = GRAY02;
pub const DARK_FOREGROUND: Color = GRAY13;
pub const DARK_TEXT_NORMAL: [Color; 3] = [GRAY13, GRAY02, GRAY08];
pub const DARK_TEXT_BUMP_SMALL: [Color; 3] = [GRAY09, GRAY13, GRAY07];
pub const DARK_TEXT_BUMP_LARGE: [Color; 3] = [GRAY09, GRAY13, GRAY08];
pub const DARK_TEXT_INVERTED_SOFT: [Color; 3] = [GRAY07, GRAY13, GRAY13];
pub const DARK_TEXT_INVERTED_HARD: [Color; 3] = [GRAY13, GRAY02, GRAY08];
pub const DARK_KEYBOARD_BG: Color = GRAY03;
pub const DARK_SEPARATOR: Color = GRAY05;
pub const DARK_SEPARATOR_STRONG: Color = GRAY02;
pub const DARK_READING_PROGRESS: Color = GRAY02;
pub const DARK_PROGRESS_FULL: Color = GRAY02;
pub const DARK_PROGRESS_EMPTY: Color = GRAY10;
pub const DARK_PROGRESS_VALUE: Color = GRAY04;
pub const DARK_BATTERY_FILL: Color = GRAY02;

pub const SEPIA_BACKGROUND: Color = GRAYF4;
pub const SEPIA_FOREGROUND: Color = GRAY5C;

// Highlight colors
pub const YELLOW: Color = Color::Rgb(255, 255, 0);
pub const GREEN: Color = Color::Rgb(0, 255, 0);
pub const BLUE: Color = Color::Rgb(0, 0, 255);
pub const RED: Color = Color::Rgb(255, 0, 0);
pub const ORANGE: Color = Color::Rgb(255, 165, 0);
pub const PURPLE: Color = Color::Rgb(128, 0, 128);

/// Get the background color for the current theme
///
/// # Arguments
/// * `dark` - Whether dark mode is enabled
///
/// # Returns
/// The appropriate background color (WHITE for light mode, DARK_BACKGROUND for dark mode)
#[inline]
pub fn background(dark: bool) -> Color {
    if dark {
        DARK_BACKGROUND
    } else {
        WHITE
    }
}

/// Get the foreground color for the current theme
///
/// # Arguments
/// * `dark` - Whether dark mode is enabled
///
/// # Returns
/// The appropriate foreground color (BLACK for light mode, DARK_FOREGROUND for dark mode)
#[inline]
pub fn foreground(dark: bool) -> Color {
    if dark {
        DARK_FOREGROUND
    } else {
        BLACK
    }
}

/// Get the normal text color gradient for the current theme
///
/// Returns a 3-color gradient for text rendering (light, normal, dark)
///
/// # Arguments
/// * `dark` - Whether dark mode is enabled
///
/// # Returns
/// Array of 3 colors for text gradient
#[inline]
pub fn text_normal(dark: bool) -> [Color; 3] {
    if dark {
        DARK_TEXT_NORMAL
    } else {
        TEXT_NORMAL
    }
}

/// Get the small bump text color gradient for the current theme
///
/// Returns a 3-color gradient for small text emphasis
///
/// # Arguments
/// * `dark` - Whether dark mode is enabled
///
/// # Returns
/// Array of 3 colors for text gradient
#[inline]
pub fn text_bump_small(dark: bool) -> [Color; 3] {
    if dark {
        DARK_TEXT_BUMP_SMALL
    } else {
        TEXT_BUMP_SMALL
    }
}

/// Get the separator color for the current theme
///
/// # Arguments
/// * `dark` - Whether dark mode is enabled
///
/// # Returns
/// The appropriate separator color
#[inline]
pub fn separator(dark: bool) -> Color {
    if dark {
        DARK_SEPARATOR
    } else {
        SEPARATOR_NORMAL
    }
}

/// Get the keyboard background color for the current theme
///
/// # Arguments
/// * `dark` - Whether dark mode is enabled
///
/// # Returns
/// The appropriate keyboard background color
#[inline]
pub fn keyboard_bg(dark: bool) -> Color {
    if dark {
        DARK_KEYBOARD_BG
    } else {
        KEYBOARD_BG
    }
}

/// Get the hard inverted text color gradient for the current theme
///
/// Returns a 3-color gradient for hard inverted text (high contrast)
///
/// # Arguments
/// * `dark` - Whether dark mode is enabled
///
/// # Returns
/// Array of 3 colors for text gradient
#[inline]
pub fn text_inverted_hard(dark: bool) -> [Color; 3] {
    if dark {
        DARK_TEXT_INVERTED_HARD
    } else {
        TEXT_INVERTED_HARD
    }
}

/// Get the soft inverted text color gradient for the current theme
///
/// Returns a 3-color gradient for soft inverted text (medium contrast)
///
/// # Arguments
/// * `dark` - Whether dark mode is enabled
///
/// # Returns
/// Array of 3 colors for text gradient
#[inline]
pub fn text_inverted_soft(dark: bool) -> [Color; 3] {
    if dark {
        DARK_TEXT_INVERTED_SOFT
    } else {
        TEXT_INVERTED_SOFT
    }
}

/// Get the large bump text color gradient for the current theme
///
/// Returns a 3-color gradient for large text emphasis
///
/// # Arguments
/// * `dark` - Whether dark mode is enabled
///
/// # Returns
/// Array of 3 colors for text gradient
#[inline]
pub fn text_bump_large(dark: bool) -> [Color; 3] {
    if dark {
        DARK_TEXT_BUMP_LARGE
    } else {
        TEXT_BUMP_LARGE
    }
}

#[inline]
pub fn separator_strong(dark: bool) -> Color {
    if dark {
        DARK_SEPARATOR_STRONG
    } else {
        SEPARATOR_STRONG
    }
}

#[inline]
pub fn reading_progress(dark: bool) -> Color {
    if dark {
        DARK_READING_PROGRESS
    } else {
        READING_PROGRESS
    }
}

#[inline]
pub fn progress_full(dark: bool) -> Color {
    if dark {
        DARK_PROGRESS_FULL
    } else {
        PROGRESS_FULL
    }
}

#[inline]
pub fn progress_empty(dark: bool) -> Color {
    if dark {
        DARK_PROGRESS_EMPTY
    } else {
        PROGRESS_EMPTY
    }
}

#[inline]
pub fn progress_value(dark: bool) -> Color {
    if dark {
        DARK_PROGRESS_VALUE
    } else {
        PROGRESS_VALUE
    }
}

#[inline]
pub fn battery_fill(dark: bool) -> Color {
    if dark {
        DARK_BATTERY_FILL
    } else {
        BATTERY_FILL
    }
}

// ============================================================================
// Mobile platform color integration
// ============================================================================

/// Check if running on mobile platform (Android/iOS)
#[inline]
fn is_mobile_platform() -> bool {
    std::env::var("ANDROID_ROOT").is_ok() || std::env::var("IPHONE_SIMULATOR_ROOT").is_ok()
}

/// Get platform-optimized background color
/// Uses mobile colorful themes on Android/iOS, grayscale on e-ink
#[inline]
pub fn platform_background(dark: bool) -> Color {
    if is_mobile_platform() {
        crate::mobile_theme::mobile_background()
    } else {
        background(dark)
    }
}

/// Get platform-optimized text color
#[inline]
pub fn platform_text_primary(dark: bool) -> Color {
    if is_mobile_platform() {
        crate::mobile_theme::mobile_text_primary()
    } else {
        foreground(dark)
    }
}

/// Get platform-optimized surface color (for cards/panels)
#[inline]
pub fn platform_surface() -> Color {
    if is_mobile_platform() {
        crate::mobile_theme::mobile_surface()
    } else {
        WHITE
    }
}

/// Get platform-optimized accent color (blue)
/// Returns vibrant color on mobile, grayscale on e-ink
#[inline]
pub fn platform_accent_primary() -> Color {
    if is_mobile_platform() {
        crate::mobile_theme::mobile_accent_blue()
    } else {
        GRAY07 // Neutral gray for e-ink
    }
}

/// Get platform-optimized success color (green)
#[inline]
pub fn platform_accent_success() -> Color {
    if is_mobile_platform() {
        crate::mobile_theme::mobile_accent_green()
    } else {
        GRAY05 // Darker gray for e-ink
    }
}

/// Get platform-optimized warning color (amber)
#[inline]
pub fn platform_accent_warning() -> Color {
    if is_mobile_platform() {
        crate::mobile_theme::mobile_accent_amber()
    } else {
        GRAY09 // Medium gray for e-ink
    }
}

/// Get platform-optimized error color (red)
#[inline]
pub fn platform_accent_error() -> Color {
    if is_mobile_platform() {
        crate::mobile_theme::mobile_accent_red()
    } else {
        GRAY03 // Dark gray for e-ink
    }
}
