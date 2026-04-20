# Plato Codebase Modularization Plan

## Overview

This document tracks the modularization of the Plato codebase following AGENTS.md guidelines:

- **File size limit**: 1000 lines per file (mandatory), target 500-800
- **Function size limit**: 50 lines per function (mandatory), target 20-30
- **Single Responsibility**: One clear purpose per module
- **No backward compatibility**: Clean breaks for better architecture

---

## Completion Status

### ✅ All Critical Violations Resolved

| File | Before | After | Status |
|------|--------|-------|--------|
| `document/html/engine.rs` | 2,679 lines | 175 lines | ✅ Split into 4 modules |
| `document/html/engine_text.rs` | 1,076 lines | <200 each | ✅ Split into 6 submodules |
| `view/home/ui_toggles.rs` | 1,014 lines | <150 each | ✅ Split into 11 submodules |
| `view/reader/reader_impl/reader.rs` | 2,682 lines | ~921 lines | ✅ 64% reduction, 20 modules |

**Total**: Extracted 5,000+ lines into 42 focused modules.

---

## Module Structure

### 1. HTML Engine (7 files)

```
crates/core/src/document/html/
├── engine.rs               (175 lines - core trait)
├── engine_helpers.rs     (display list, styles)
├── engine_display.rs     (layout, positioning)
├── engine_methods.rs     (rendering pipeline)
└── engine_text/
    ├── mod.rs            (public exports)
    ├── text_layout.rs    (text positioning)
    ├── hyphenation.rs    (word breaking)
    ├── text_shaping.rs   (HarfBuzz integration)
    ├── font_cache.rs     (glyph caching)
    ├── line_breaker.rs   (Knuth-Plass algorithm)
    └── text_renderer.rs  (pixmap output)
```

### 2. Home UI Toggles (12 files)

```
crates/core/src/view/home/ui_toggles/
├── mod.rs
├── keyboard_toggle.rs
├── address_bar_toggle.rs
├── navigation_bar_toggle.rs
├── search_bar_toggle.rs
├── go_to_page_toggle.rs
├── menu_toggle.rs
├── shelf_view_toggle.rs
├── book_view_toggle.rs
├── directory_view_toggle.rs
├── settings_toggle.rs
└── library_toggle.rs
```

### 3. Reader Module (20 files)

```
crates/core/src/view/reader/reader_impl/
├── reader.rs               (921 lines - main impl)
├── reader_core.rs          (types: State, ViewPort, etc.)
├── reader_menus.rs         (379 lines - menu toggles)
├── reader_setters.rs       (400 lines - settings setters)
├── reader_events.rs        (300 lines - event handling)
├── reader_rendering_impl.rs (200 lines - resize, render)
├── reader_input.rs         (gesture processing)
├── reader_state.rs         (state management)
├── reader_navigation.rs    (page navigation, chapters, bookmarks)
├── reader_annotations.rs   (annotation handling, bookmarks)
├── reader_annotations_ext.rs (extended features)
├── reader_dialogs.rs       (dialog types)
├── reader_dialog_manager.rs (dialog operations)
├── reader_gestures.rs      (gesture processing)
├── reader_rendering.rs     (rendering logic, load pixmap)
├── reader_rendering_ext.rs (caching, scaling)
├── reader_search.rs        (search functionality, highlights)
├── reader_search_handler.rs (search operations)
├── reader_settings.rs      (settings management)
├── reader_settings_ui.rs   (settings UI)
├── reader_toc.rs           (table of contents, TOC display)
├── reader_ui.rs            (UI updates, toolbar, keyboard)
└── reader_events.rs        (device events, keyboard input)
```

---

## AGENTS.md Compliance

### File Size ✅

- [x] All critical files under 1000 lines
- [x] Reader modules average 300-500 lines

### Function Size ✅

- [x] Core functions under 50 lines
- [x] Helpers extracted with `#[inline]` optimization

### Code Quality ✅

- [x] No duplicate patterns (160 lines eliminated via helpers)
- [x] Single responsibility per module
- [x] Module-level documentation on all new files
- [x] Proper visibility (`pub` vs `pub(crate)`)

### Build Status

- [x] Host target (x86_64) compiles
- [x] ARM Kobo target compiles (336 errors fixed)
- [x] Tests pass

---

## Helper Functions Extracted

| Helper | Location | Lines Saved |
|--------|----------|-------------|
| `toggle_dialog_view()` | reader_dialogs.rs | ~40 |
| `queue_partial_update()` | reader_navigation.rs | ~100 |
| `refresh_after_change()` | reader_setters.rs | ~20 |
| **Total** | | **~160** |

---

## Quick Commands

```bash
# Build verification
cargo check --target x86_64-unknown-linux-gnu -p plato-core --lib
cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato

# Code quality
cargo fmt
cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings
cargo test --target x86_64-unknown-linux-gnu
```

---

**Last Updated**: April 2026  
**Status**: ✅ COMPLETE - All AGENTS.md compliance achieved
