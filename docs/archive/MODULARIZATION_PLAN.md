# Plato Codebase Modularization Plan

## Overview

This document tracks the modularization of the Plato codebase following AGENTS.md guidelines:

- **File size limit**: 1000 lines per file (mandatory), target 500-800
- **Function size limit**: 50 lines per function (mandatory), target 20-30
- **Single Responsibility**: One clear purpose per module
- **No backward compatibility**: Clean breaks for better architecture

---

## Completion Status (April 22, 2026)

### ✅ All Critical Violations Resolved

| File                                | Before      | After                      | Status                       |
|-------------------------------------|-------------|----------------------------|------------------------------|
| `document/html/`                    | 2,679 lines | Multiple files < 22KB each | ✅ Split into 20 files       |
| `view/home/ui_toggles.rs`           | 1,014 lines | 13 files < 14KB each       | ✅ Split into directory      |
| `view/reader/reader_impl/reader.rs` | 2,682 lines | 25.9KB (~900 lines)        | ✅ 64% reduction, 16 modules |

**Total**: Extracted 5,000+ lines into 50+ focused modules.

---

## Module Structure

### 1. HTML Engine (20 files)

```text
crates/core/src/document/html/
├── engine.rs               (15.2KB - core trait)
├── engine_image.rs        (8.0KB - image handling)
├── engine_methods.rs      (16.9KB - rendering pipeline)
├── engine_render.rs       (17.6KB - rendering)
├── engine_table.rs        (3.5KB - table layout)
├── engine_text_gather.rs  (10.7KB - text gathering)
├── engine_text_hyphenate.rs (13.5KB - hyphenation)
├── engine_text_items.rs   (18.9KB - text items)
├── css.rs                 (15.5KB - CSS parsing)
├── dom.rs                 (15.7KB - DOM handling)
├── layout.rs              (17.4KB - layout engine)
├── parse.rs               (21.8KB - XML parsing)
├── style.rs               (11.1KB - styling)
├── xml.rs                 (10.2KB - XML utilities)
├── mod.rs                 (17.5KB - public exports)
├── engine_text/           (directory - text processing)
└── Multiple test files
```

### 2. Home UI Toggles (13 files)

```text
crates/core/src/view/home/ui_toggles/
├── mod.rs                  (485B - public exports)
├── keyboard_toggle.rs      (5.5KB)
├── address_bar_toggle.rs   (4.9KB)
├── navigation_bar_toggle.rs (4.9KB)
├── search_bar_toggle.rs    (4.8KB)
├── go_to_page_toggle.rs    (5.2KB)
├── menu_toggle.rs          (9.4KB)
├── shelf_view_toggle.rs    (4.7KB)
├── book_view_toggle.rs     (8.2KB)
├── directory_view_toggle.rs (8.1KB)
├── settings_toggle.rs     (6.4KB)
├── library_toggle.rs       (13.7KB)
└── utils.rs                (2.8KB)
```

### 3. Reader Module (16 files)

```text
crates/core/src/view/reader/reader_impl/
├── mod.rs                  (1.9KB - public exports)
├── reader.rs               (25.9KB - main impl)
├── reader_core.rs          (3.7KB - types: State, ViewPort, etc.)
├── reader_menus.rs         (10.3KB - menu toggles)
├── reader_setters.rs       (12.2KB - settings setters)
├── reader_events.rs        (1.1KB - event handling)
├── reader_rendering_impl.rs (8.5KB - resize, render)
├── reader_gestures.rs      (8.4KB - gesture processing)
├── reader_navigation.rs    (7.5KB - page navigation, chapters, bookmarks)
├── reader_annotations.rs   (5.1KB - annotation handling, bookmarks)
├── reader_dialogs.rs       (3.8KB - dialog types)
├── reader_rendering.rs     (6.6KB - rendering logic, load pixmap)
├── reader_search.rs        (9.9KB - search functionality, highlights)
├── reader_settings.rs      (22.0KB - settings management)
├── reader_toc.rs           (7.4KB - table of contents, TOC display)
└── reader_ui.rs            (4.0KB - UI updates, toolbar, keyboard)
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

| Helper                   | Location             | Lines Saved |
|--------------------------|----------------------|-------------|
| `toggle_dialog_view()`   | reader_dialogs.rs    | ~40         |
| `queue_partial_update()` | reader_navigation.rs | ~100        |
| `refresh_after_change()` | reader_setters.rs    | ~20         |
| **Total**                |                      | **~160**    |

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
