# Plato Theme-Aware Conversion Plan

## Current Status
✅ **FULLY IMPLEMENTED** - All core theme infrastructure is complete with **zero warnings**

### Completed Implementation:

**Core Theme Infrastructure:**
- ✅ Global dark mode state in `crate::theme` (`is_dark_mode()`, `set_dark_mode()`)
- ✅ Theme-aware color helpers in `crate::theme`:
  - `background(dark: bool) -> Color` - returns BLACK in dark mode, WHITE in light
  - `foreground(dark: bool) -> Color` - returns WHITE in dark mode, BLACK in light
- ✅ Theme-aware color helpers in `crate::color`:
  - `background(dark) -> Color`
  - `foreground(dark) -> Color`
  - `text_normal(dark) -> [Color; 3]`
  - `text_bump_small(dark) -> [Color; 3]`
  - `separator(dark)`
  - `keyboard_bg(dark)`
  - `text_inverted_hard(dark) -> [Color; 3]`
  - `text_inverted_soft(dark) -> [Color; 3]`
  - `text_bump_large(dark) -> [Color; 3]`
  - `separator_strong(dark)`
  - `reading_progress(dark)`
  - `progress_full(dark)`
  - `progress_empty(dark)`
  - `progress_value(dark)`
  - `battery_fill(dark)`

**View Component Updates (All Completed):**
- ✅ All view components updated to use theme-aware colors
- ✅ All unused imports removed
- ✅ All variable naming fixed

**Persistence Integration:**
- ✅ Initial app startup: Theme initialized from `settings.dark_mode` (`app.rs:130`)
- ✅ Settings toggle: Theme updated when user changes dark mode (`display.rs:313`)
- ✅ USB connect: Theme re-synced when settings reloaded (`app.rs:700`)
- ✅ Settings auto-save: Dark mode preserved on suspend/exit

## Files Modified

### Theme Module (`crates/core/src/theme.rs`)
- Added `background(dark: bool) -> Color` helper
- Added `foreground(dark: bool) -> Color` helper

### Color Module (`crates/core/src/color.rs`)
- Already had all theme-aware helpers implemented

### App Integration (`crates/plato/src/app.rs`)
- Initialize theme state from settings on startup
- Re-sync theme state when settings reloaded on USB connect

### Settings View (`crates/core/src/view/settings/display.rs`)
- Update theme state when user toggles dark mode

### View Components Updated (30+ files)
- Updated imports and function calls to use theme-aware colors

## Build Verification

- ✅ **ARM Build (32-bit)**: Passes with zero warnings
- ✅ **Cargo fmt**: Runs successfully
- ⚠️ **ARM64 Build**: Requires cross-compilation toolchain (aarch64-linux-gnu-gcc)
- ⚠️ **Host Build**: Requires native libraries (mupdf_wrapper)

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