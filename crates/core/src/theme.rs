use std::sync::LazyLock;

use crate::color;
use crate::settings::{ThemeMode, ThemeSchedule};
use chrono::{DateTime, Local, Timelike};

/// Detect system dark mode for desktop platforms
fn detect_system_dark_mode() -> bool {
    // Check for explicit system dark mode environment variable
    if let Ok(value) = std::env::var("SYSTEM_DARK_MODE") {
        return value == "1" || value == "true";
    }

    // Platform-specific detection
    #[cfg(target_os = "linux")]
    {
        detect_linux_dark_mode()
    }

    #[cfg(target_os = "macos")]
    {
        detect_macos_dark_mode()
    }

    #[cfg(target_os = "windows")]
    {
        detect_windows_dark_mode()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        // Fallback to light mode for unsupported platforms
        false
    }
}

/// Detect dark mode on Linux systems
#[cfg(target_os = "linux")]
fn detect_linux_dark_mode() -> bool {
    // Check GNOME/GTK settings
    if let Ok(value) = std::env::var("GTK_THEME") {
        if value.contains("dark") {
            return true;
        }
    }

    // Check for KDE settings
    if let Ok(value) = std::env::var("KDE_COLOR_SCHEME") {
        if value.contains("dark") {
            return true;
        }
    }

    // Check for XDG desktop portal settings
    if let Ok(value) = std::env::var("XDG_CURRENT_DESKTOP") {
        if value.to_lowercase().contains("kde") || value.to_lowercase().contains("gnome") {
            // Try to read from gsettings or kconfig
            if let Ok(output) = std::process::Command::new("gsettings")
                .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
                .output()
            {
                let theme = String::from_utf8_lossy(&output.stdout);
                if theme.to_lowercase().contains("dark") {
                    return true;
                }
            }
        }
    }

    false
}

/// Detect dark mode on macOS systems
#[cfg(target_os = "macos")]
fn detect_macos_dark_mode() -> bool {
    if let Ok(output) = std::process::Command::new("defaults")
        .args(&["read", "-g", "AppleInterfaceStyle"])
        .output()
    {
        let style = String::from_utf8_lossy(&output.stdout);
        style.trim() == "Dark"
    } else {
        false
    }
}

/// Detect dark mode on Windows systems
#[cfg(target_os = "windows")]
fn detect_windows_dark_mode() -> bool {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    // Try to read from Windows Registry
    if let Ok(output) = std::process::Command::new("reg")
        .args(&[
            "query",
            "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        // AppsUseLightTheme = 0x0 means dark mode is enabled
        if output_str.contains("0x0") {
            return true;
        }
    }

    false
}

static DARK_MODE: LazyLock<std::sync::Mutex<bool>> = LazyLock::new(|| std::sync::Mutex::new(false));
static THEME_MODE: LazyLock<std::sync::Mutex<ThemeMode>> =
    LazyLock::new(|| std::sync::Mutex::new(ThemeMode::System));
static AUTO_THRESHOLD: LazyLock<std::sync::Mutex<u16>> =
    LazyLock::new(|| std::sync::Mutex::new(100));

#[inline]
pub fn is_dark_mode() -> bool {
    *DARK_MODE.lock().expect("DARK_MODE lock poisoned")
}

#[inline]
pub fn is_sepia_mode() -> bool {
    *THEME_MODE.lock().expect("THEME_MODE lock poisoned") == ThemeMode::Sepia
}

#[inline]
pub fn theme_mode() -> ThemeMode {
    *THEME_MODE.lock().expect("THEME_MODE lock poisoned")
}

#[inline]
pub fn set_dark_mode(enabled: bool) {
    *DARK_MODE.lock().expect("DARK_MODE lock poisoned") = enabled;
}

#[inline]
pub fn set_theme_mode(mode: ThemeMode) {
    *THEME_MODE.lock().expect("THEME_MODE lock poisoned") = mode;
    match mode {
        ThemeMode::Light | ThemeMode::Sepia => {
            *DARK_MODE.lock().expect("DARK_MODE lock poisoned") = false;
        }
        ThemeMode::Dark => {
            *DARK_MODE.lock().expect("DARK_MODE lock poisoned") = true;
        }
        ThemeMode::System => {
            *DARK_MODE.lock().expect("DARK_MODE lock poisoned") = detect_system_dark_mode();
        }
        ThemeMode::Auto | ThemeMode::Scheduled => {}
    }
}

#[inline]
pub fn set_auto_threshold(threshold: u16) {
    *AUTO_THRESHOLD.lock().expect("AUTO_THRESHOLD lock poisoned") = threshold;
}

#[inline]
pub fn auto_threshold() -> u16 {
    *AUTO_THRESHOLD.lock().expect("AUTO_THRESHOLD lock poisoned")
}

#[inline]
pub fn update_from_light_sensor(light_level: u16) {
    let mode = *THEME_MODE.lock().expect("THEME_MODE lock poisoned");
    if mode == ThemeMode::Auto {
        let threshold = *AUTO_THRESHOLD.lock().expect("AUTO_THRESHOLD lock poisoned");
        let dark = light_level < threshold;
        *DARK_MODE.lock().expect("DARK_MODE lock poisoned") = dark;
    } else if mode == ThemeMode::System {
        // Re-detect system theme in case it changed
        *DARK_MODE.lock().expect("DARK_MODE lock poisoned") = detect_system_dark_mode();
    }
}

#[inline]
pub fn update_from_schedule(schedule: &ThemeSchedule, current_time: &DateTime<Local>) {
    if *THEME_MODE.lock().expect("THEME_MODE lock poisoned") != ThemeMode::Scheduled
        || !schedule.enabled
    {
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

    *DARK_MODE.lock().expect("DARK_MODE lock poisoned") = is_dark;
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
