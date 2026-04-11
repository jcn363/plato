use std::sync::LazyLock;

use crate::color;
use crate::settings::{ThemeMode, ThemeSchedule};
use chrono::{DateTime, Local, Timelike};

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
        ThemeMode::Auto | ThemeMode::Scheduled => {}
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
pub fn update_from_schedule(schedule: &ThemeSchedule, current_time: &DateTime<Local>) {
    if *THEME_MODE.lock().unwrap() != ThemeMode::Scheduled || !schedule.enabled {
        return;
    }

    let time = current_time.time();
    let now_minutes = (time.hour() as u16) * 60 + (time.minute() as u16);
    let start_minutes = schedule.dark_start.as_minutes();
    let end_minutes = schedule.dark_end.as_minutes();

    let is_dark = if start_minutes <= end_minutes {
        now_minutes >= start_minutes && now_minutes < end_minutes
    } else {
        now_minutes >= start_minutes || now_minutes < end_minutes
    };

    *DARK_MODE.lock().unwrap() = is_dark;
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
