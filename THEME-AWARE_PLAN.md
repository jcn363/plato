# Plato Theme-Aware Conversion Plan

## Current Status
✅ **FULLY IMPLEMENTED** - All core theme infrastructure is complete with **zero warnings**

### Completed Implementation:

**Core Theme Infrastructure:**
- ✅ Global dark mode state in `crate::theme` (`is_dark_mode()`, `set_dark_mode()`)
- ✅ Theme mode with auto/light/dark options (`theme_mode()`, `set_theme_mode()`)
- ✅ Auto threshold for light sensor (`auto_threshold()`, `set_auto_threshold()`)
- ✅ Light sensor integration (`update_from_light_sensor()`)
- ✅ Theme-aware color helpers in `crate::theme`:
  - `background(dark: bool) -> Color` - returns BLACK in dark mode, WHITE in light
  - `foreground(dark: bool) -> Color` - returns WHITE in dark mode, BLACK in light
- ✅ Theme-aware color helpers in `crate::color`:
  - `background(dark) -> Color`
  - `foreground(dark) -> Color`
  - All other theme-aware color helpers

**Settings Integration:**
- ✅ ThemeSettings struct with mode and auto_threshold fields
- ✅ ThemeMode enum: Light, Dark, Auto
- ✅ Settings UI: Toggle cycles through Off → On → Auto
- ✅ Auto Threshold: Button to adjust sensitivity (50-200, step 50)

**Persistence Integration:**
- ✅ Initial app startup: Theme initialized from settings
- ✅ Settings toggle: Theme updated when user changes mode
- ✅ USB connect: Theme re-synced when settings reloaded
- ✅ Settings auto-save: All theme preferences preserved

**Auto Theme Feature:**
- ✅ Light sensor integration on CheckBattery events
- ✅ Automatic dark mode based on ambient light level
- ✅ Configurable threshold (default 100, range 50-200)

## Files Modified

### Theme Module (`crates/core/src/theme.rs`)
- Added `theme_mode()`, `set_theme_mode()` for mode state
- Added `auto_threshold()`, `set_auto_threshold()` for sensitivity
- Added `update_from_light_sensor()` for auto theme

### Settings Module (`crates/core/src/settings/`)
- Added `theme.rs` with ThemeMode enum and ThemeSettings struct
- Updated `mod.rs` to include theme settings in Settings struct

### Settings Display (`crates/core/src/view/settings/display.rs`)
- Added Theme Mode toggle (cycles: Off/On/Auto)
- Added Auto Threshold button
- Updated event handlers for new UI

### App Integration (`crates/plato/src/app.rs`)
- Initialize theme state from settings on startup
- Re-sync theme state when settings reloaded on USB connect
- Check light sensor for auto theme on battery check events

### Entry IDs (`crates/core/src/view/entries.rs`)
- Added SetAutoThemeThreshold entry

## Build Verification

- ✅ **ARM Build (32-bit)**: Passes with zero warnings
- ✅ **Cargo fmt**: Runs successfully

## Implementation Notes

### Theme Helper Functions in `theme.rs`
```rust
#[inline]
pub fn background(dark: bool) -> crate::color::Color {
    if dark { crate::color::BLACK } else { crate::color::WHITE }
}

#[inline]
pub fn foreground(dark: bool) -> crate::color::Color {
    if dark { crate::color::WHITE } else { crate::color::BLACK }
}
```

### Usage in Views
```rust
use crate::theme;
use crate::color::{background, foreground};

// In render method:
fb.draw_rectangle(&self.rect, background(theme::is_dark_mode()));
font.render(fb, foreground(theme::is_dark_mode()), &plan, pt);
```

### Persistence Flow
1. **App Startup**: `theme::set_dark_mode(settings.dark_mode)` at `app.rs:130`
2. **User Toggle**: `theme::set_dark_mode(context.settings.dark_mode)` at `display.rs:313`
3. **Settings Saved**: Automatic on suspend/exit via `save_toml()`
4. **USB Reconnect**: Re-sync via `theme::set_dark_mode(dark_mode)` at `app.rs:700`

## Timeline

- ✅ **Phase 1-5**: COMPLETE - Theme system fully implemented with persistence