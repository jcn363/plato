# Plato Integration Quick Reference

## Status Vocabulary

- `Completed`
- `Open`
- `Deferred`
- `Blocked`
- `Stale/Retired`

## Completed

- `with_child!` exists in `crates/core/src/view/common.rs`
- `add_menu()` exists in `crates/core/src/view/common.rs`
- Menu toggle helpers exist in `crates/core/src/view/menu_helpers.rs`
- Reader support modules exist in `crates/core/src/view/reader/reader_impl/`, indicating progress on splitting the monolithic reader file.

## Open

### Reader

- File: `crates/core/src/view/reader/reader_impl/reader.rs`
- Current size: `3403` lines (Target: < 1000)
- Active issue: file still ends with a duplicate stub-method block
- Follow-up: complete or unwind the partial reader split

### Home

- File: `crates/core/src/view/home/mod.rs`
- Current size: `2769` lines (Target: < 1000)
- Active issue: the active home implementation is still oversized
- Follow-up: split by active responsibility, not just line count

### Fonts

- File: `crates/core/src/font/mod.rs`
- Current size: `2400` lines (Target: < 1000)
- Active issue: uses direct FFI instead of safe wrappers
- Follow-up: migrate to safe wrappers and split into submodules

### Test Segregation

- Active issue: unit tests are currently embedded in production code
- Follow-up: extract all tests into sibling `{module}_tests.rs` files

### PDF Tools

- File: `crates/core/src/view/pdf_manipulator.rs`
- **Status:** Partially completed. Interactive page selection and degree input for Delete, Rotate, and Extract are implemented. Redaction workflow includes page selection and a mode to define the region.
- **Active issue:** Interactive redaction region definition UI is pending implementation. File selection for PDF merging is missing. Some manipulation paths still depend on hard-coded defaults rather than fully integrated user inputs.
- **Follow-up:** Implement interactive redaction region definition, file selection for merge, and integrate user inputs.

### Cover Editor

- File: `crates/core/src/view/cover_editor.rs`
- **Status:** Substantially completed. Interactive controls for Rotate, Grayscale, Save, Reset, Brightness, and Contrast are implemented. Crop functionality is initiated with UI elements and mode transitions, allowing visual selection.
- **Active issue:** Interactive cropping region application is pending. Some helper functions might still be guarded by `#[allow(dead_code)]`.
- **Follow-up:** Implement interactive crop application and address remaining dead code.

## Verification

- `cargo check --target x86_64-unknown-linux-gnu`: passes
- `cargo check --target arm-unknown-linux-gnueabihf -p plato`: passes
- `cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato`: passes

Clean-build note:

- After `cargo clean`, Kobo release builds also require rebuilding `mupdf_wrapper`, because Cargo does not regenerate that native archive by itself.

## Deferred

- settings registry work
- event-system unification
- cross-cutting input-validation framework
- broad speculative performance refactors

## Stale/Retired

These should not be listed as missing opportunities anymore:

- `with_child!` macro
- `add_menu()` helper
- generic menu toggle helpers

## Large Files

| File | Lines |
|------|------:|
| `reader_impl/reader.rs` | 3403 |
| `view/home/mod.rs` | 2769 |
| `font/mod.rs` | 2400 |
| `document/html/engine.rs` | 2672 |
| `document/html/layout.rs` | 718 |
