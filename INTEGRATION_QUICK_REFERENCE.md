# Plato Integration Quick Reference

## Status Vocabulary

- `Completed`
- `Open`
- `Deferred`
- `Blocked`
- `Stale/Retired`

## Completed

- `with_child!` macro in `crates/core/src/view/common.rs`
- `add_menu()` helper in `crates/core/src/view/common.rs`
- Generic menu toggle helpers in `crates/core/src/view/menu_helpers.rs`
- **Safe Wrapper Migration for Fonts**: `crates/core/src/font/mod.rs` (802 lines) is now fully integrated with safe wrappers.
- **Unit Test Segregation**: All unit tests in `crates/core/src` have been moved to sibling `_tests.rs` files.
- **Epub Editor Integration**: Moved from standalone tool to `crates/epub_editor` workspace member.
- **PDF Tools UI Workflow Completion**: Interactive redaction region definition UI and file selection flow for PDF merging are implemented.

## Open

### 1. Reader stub block and partial module migration

**Problem**

The reader refactor is incomplete. `reader.rs` is still **3403 lines** and contains a large block of stubbed methods (lines 3206+) that provide no functionality.

**Recommended action**

- Complete the extraction of logic from `reader.rs` into `reader_gestures.rs`, `reader_rendering.rs`, etc.
- Replace the stub methods with active call paths to the new modules.
- Ensure all functions are under 50 lines.

### 2. Home view modularization

**Status:** Completed

The home view modularization is complete. `home/mod.rs` has been reduced from 2786 to 596 lines. All modules extracted: ops.rs, ui_toggles.rs, library.rs, fetcher.rs, navigation.rs, updates.rs, input.rs

**Recommended action**

(N/A - task completed)

### 3. Fonts module refactoring

**Problem**

The font module (`crates/core/src/font/mod.rs`) is **2400 lines** and uses direct FFI instead of safe wrappers.

**Recommended action**

- Migrate to safe wrappers and split into submodules.

## Deferred

- Settings registry / schema-generation layer
- Generic object pooling
- Smooth theme transition animations

## Verification Status

### Host Verification (x86_64 Linux)
- **Command:** `cargo check --target x86_64-unknown-linux-gnu`
- **Result:** Pass (2026-04-13)

### ARM Kobo Verification (32-bit ARM)
- **Command:** `./build.sh arm fast`
- **Result:** Pass (2026-04-13)
