# Plato Theme-Aware System

## Overview

The Plato e-reader now supports a comprehensive theme system with light/dark/sepia/auto modes, light sensor integration, gesture controls, and persistent settings.

## Features Implemented

### 1. Theme Modes
- **Light Mode**: White background (#FFFFFF), black text
- **Dark Mode**: Dark gray background (#222222), light gray text
- **Sepia Mode**: Warm beige background (#F4E4BC), dark brown text (#5C5C5C)
- **Auto Mode**: Automatic switching based on ambient light sensor

### 2. Auto Theme (Light Sensor)
- Uses device's ambient light sensor to automatically switch themes
- Configurable threshold (50-200, default 100)
- Polls light level on battery check events (~30 seconds)
- Lower threshold = darker environment needed to trigger dark mode

### 3. Theme Toggle Gesture
- **Two-finger swipe from left edge**: Switch to dark mode
- **Two-finger swipe from right edge**: Switch to sepia mode
- Only works when in Auto mode (exits auto to manual selection)

### 4. Theme Indicator
- Icon in top bar shows current mode (sun/moon/sepia/auto icon)
- Tap icon to cycle through modes (same as settings toggle)

### 5. Settings UI
- Toggle cycles: Off (Light) → On (Dark) → Sepia → Auto
- Auto Threshold button: Adjust sensitivity (50-200, step 50)

### 6. Scheduled Theme Mode
- Automatically switch themes based on time of day
- Configurable sunset/sunrise times or fixed times
- Integrated with settings for custom scheduling
- Persistence across reboots and USB reconnects

### 7. Persistence
- All theme settings saved to Settings.toml
- Survives app restart and USB reconnect
- Settings auto-saved on suspend/exit

## Architecture

### Core Theme Module (`crates/core/src/theme.rs`)

```rust
// Theme state management
pub fn is_dark_mode() -> bool          // Current dark/light state
pub fn is_sepia_mode() -> bool         // Current sepia state
pub fn set_dark_mode(enabled: bool)    // Set dark/light manually
pub fn theme_mode() -> ThemeMode        // Current mode (Light/Dark/Sepia/Auto/Scheduled)
pub fn set_theme_mode(mode: ThemeMode) // Set mode
pub fn auto_threshold() -> u16         // Get auto threshold
pub fn set_auto_threshold(threshold: u16) // Set auto threshold
pub fn update_from_light_sensor(level: u16) // Update from light sensor
pub fn update_from_schedule(schedule: &ThemeSchedule, current_time: &DateTime<Local>) // Update from schedule

// Color helpers (used by views)
pub fn background(dark: bool) -> Color
pub fn foreground(dark: bool) -> Color
pub fn sepia_background() -> Color
pub fn sepia_foreground() -> Color
```

### Settings Module (`crates/core/src/settings/theme.rs`)

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
    Sepia,
    Auto,
    Scheduled,
}

pub struct ThemeSchedule {
    pub enabled: bool,
    pub sunrise: (u8, u8),
    pub sunset: (u8, u8),
}

pub struct ThemeSettings {
    pub mode: ThemeMode,
    pub auto_threshold: u16,  // Light level threshold for auto mode
    pub schedule: ThemeSchedule,
}
```

### Color Module (`crates/core/src/color.rs`)

```rust
// Sepia theme colors
pub const SEPIA_BACKGROUND: Color = GRAYF4  // 0xF4
pub const SEPIA_FOREGROUND: Color = GRAY5C  // 0x5C
```

### Key Integration Points

| Location | Purpose |
|----------|---------|
| `app.rs:130` | Initialize theme on startup |
| `app.rs:703` | Re-sync theme on USB reconnect |
| `app.rs:824` | Poll light sensor for auto theme |
| `app.rs:832` | Update theme from schedule |
| `app.rs:977` | Handle two-finger swipe gesture |
| `display.rs:311` | Handle settings toggle |
| `top_bar.rs` | Render theme indicator icon |

## Files Modified

### Core Theme
- `crates/core/src/theme.rs` - Theme state management, color helpers, schedule logic
- `crates/core/src/settings/theme.rs` - ThemeMode enum, ThemeSettings and ThemeSchedule structs
- `crates/core/src/color.rs` - Theme-aware color functions (pre-existing)

### Views
- `crates/core/src/view/settings/display.rs` - Settings UI for modes and schedule
- `crates/core/src/view/top_bar.rs` - Theme indicator
- `crates/core/src/view/entries.rs` - EntryId additions

### App Integration
- `crates/plato/src/app.rs` - Startup, USB reconnect, gesture handling, schedule polling

### Icons
- `icons/theme-light.svg` - Sun icon for light mode
- `icons/theme-dark.svg` - Moon icon for dark mode
- `icons/theme-auto.svg` - Auto icon for auto mode

## Usage

### From Settings
1. Go to Settings → Display
2. Tap "Dark Mode" toggle to cycle: Off → On → Sepia → Auto → Scheduled
3. Tap "Auto Threshold" to adjust sensitivity
4. Tap "Schedule" to configure sunset/sunrise times

### From Gesture
1. In Auto or Scheduled mode, use two-finger swipe from screen edges
2. Left edge → Dark mode
3. Right edge → Sepia mode

### From Top Bar
1. Tap theme icon (left of menu icon)
2. Cycles through modes

## Build Verification

```bash
# ARM build (Kobo 32-bit)
cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato

# ARM64 build (newer Kobo devices)
cargo build --target aarch64-unknown-linux-gnu --profile release-arm64

# Host build (development)
cargo build --target x86_64-unknown-linux-gnu
```

- ✅ ARM Build: Zero warnings
- ✅ Cargo fmt: Passes

## Theme System Summary

All theme features implemented in this release:

| Feature | Status |
|---------|--------|
| Light/Dark themes | ✅ Complete |
| Sepia theme | ✅ Complete |
| Auto theme (light sensor) | ✅ Complete |
| Scheduled theme | ✅ Complete |
| Settings UI | ✅ Complete |
| Persistence | ✅ Complete |
| Gesture toggle | ✅ Complete |
| Theme indicator in top bar | ✅ Complete |
| Inverted book covers in dark/sepia | ✅ Complete |

## Future Enhancements

Potential improvements for future releases:
- Smooth theme transition animations
- Per-document theme preferences
- Custom color theme support