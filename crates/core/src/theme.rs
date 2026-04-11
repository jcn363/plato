use std::sync::LazyLock;

use crate::color;
use crate::settings::ThemeMode;

static DARK_MODE: LazyLock<std::sync::Mutex<bool>> = LazyLock::new(|| std::sync::Mutex::new(false));
static THEME_MODE: LazyLock<std::sync::Mutex<ThemeMode>> =
    LazyLock::new(|| std::sync::Mutex::new(ThemeMode::Light));
static AUTO_THRESHOLD: LazyLock<std::sync::Mutex<u16>> =
    LazyLock::new(|| std::sync::Mutex::new(100));

#[inline]
pub fn is_dark_mode() -> bool {
    *DARK_MODE.lock().unwrap()
}

#[inline]
pub fn is_sepia_mode() -> bool {
    *THEME_MODE.lock().unwrap() == ThemeMode::Sepia
}

#[inline]
pub fn theme_mode() -> ThemeMode {
    *THEME_MODE.lock().unwrap()
}

#[inline]
pub fn set_dark_mode(enabled: bool) {
    *DARK_MODE.lock().unwrap() = enabled;
}

#[inline]
pub fn set_theme_mode(mode: ThemeMode) {
    *THEME_MODE.lock().unwrap() = mode;
    match mode {
        ThemeMode::Light | ThemeMode::Sepia => {
            *DARK_MODE.lock().unwrap() = false;
        }
        ThemeMode::Dark => {
            *DARK_MODE.lock().unwrap() = true;
        }
        ThemeMode::Auto => {}
    }
}

#[inline]
pub fn set_auto_threshold(threshold: u16) {
    *AUTO_THRESHOLD.lock().unwrap() = threshold;
}

#[inline]
pub fn auto_threshold() -> u16 {
    *AUTO_THRESHOLD.lock().unwrap()
}

#[inline]
pub fn update_from_light_sensor(light_level: u16) {
    let mode = *THEME_MODE.lock().unwrap();
    if mode == ThemeMode::Auto {
        let threshold = *AUTO_THRESHOLD.lock().unwrap();
        let dark = light_level < threshold;
        *DARK_MODE.lock().unwrap() = dark;
    }
}

#[inline]
pub fn background(dark: bool) -> color::Color {
    if dark {
        color::DARK_BACKGROUND
    } else {
        color::WHITE
    }
}

#[inline]
pub fn foreground(dark: bool) -> color::Color {
    if dark {
        color::DARK_FOREGROUND
    } else {
        color::BLACK
    }
}

#[inline]
pub fn sepia_background() -> color::Color {
    color::SEPIA_BACKGROUND
}

#[inline]
pub fn sepia_foreground() -> color::Color {
    color::SEPIA_FOREGROUND
}
