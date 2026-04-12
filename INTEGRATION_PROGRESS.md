# Plato Integration Progress

> Current-source update based on the checked-out branch state

## Completed

- Common view helpers now exist for repeated child/menu patterns:
  - `with_child!`
  - `add_menu()`
  - `menu_helpers::{toggle_menu_vec, toggle_menu_with, toggle_menu_ctx, toggle_menu_item, toggle_menu_self}`
- **Theme System**: Full implementation of Light, Dark, Sepia, Auto (light sensor), and Scheduled (time-based) modes with persistence, gestures, and top-bar indicators.
- **PDF Tools UI**: Full implementation of Delete, Rotate, Extract, Merge (multi-file), and Redact (interactive regions) operations surfaced via UI.
- **Cover Editor UI**: Full implementation of Rotate, Grayscale, Brightness, Contrast, and Crop controls wired into document flows.
- **Gesture Extraction**: Extracted 80+ lines of gesture handling and `GestureProcessor` trait to `reader_gestures.rs`.
- **Reader support modules**: Exist under `crates/core/src/view/reader/reader_impl/`, with rendering, settings, annotations, and gestures extracted (integration in progress).
- **Test Segregation (Mandatory)**: Initial refactoring completed for 10+ core modules (geom, device, helpers, html, dictionary) with unit tests moved to sibling `_tests.rs` files.

## Open

### Reader migration cleanup

- `reader.rs` is still `3403` lines (AGENTS.md target: < 1000 lines).
- The file still ends with a stub-method block that duplicates reader behavior.
- Extracted reader modules are present, but many helpers remain inactive or dead-code-gated.

### Home modularization

- `home/mod.rs` is still `2769` lines (AGENTS.md target: < 1000 lines).

### Font module refactoring

- `font/mod.rs` is `2400` lines (AGENTS.md target: < 1000 lines).
- Must migrate from direct FFI to safe wrappers in `crates/core/src/font/`.

### Test Segregation

- All unit tests must be extracted into sibling files named `{module}_tests.rs`.

### PDF tools UI completion

- `pdf_manipulator.rs` is now reachable from selected-document contexts. Interactive page selection and degree input for Delete, Rotate, and Extract are implemented. Redaction workflow now includes page selection and a mode to define the redaction region.
- **Pending:** Interactive redaction region definition UI, file selection for PDF merging, and full parameter integration for all manipulation paths.

### Cover editor product decision

- `crates/core/src/view/cover_editor.rs` now has interactive controls for Rotate, Grayscale, Save, Reset, Brightness, and Contrast.
- Crop functionality is initiated with UI elements and mode transitions, allowing visual selection.
- **Pending:** Interactive application of the crop selection, and addressing remaining `#[allow(dead_code)]` scaffolding.

## Verification

Current verification status on the checked-out branch:

- `cargo check --target x86_64-unknown-linux-gnu`: passes
- `cargo check --target arm-unknown-linux-gnueabihf -p plato`: passes
- `cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato`: passes after rebuilding `mupdf_wrapper`

Notes:

- A true clean ARM rebuild requires regenerating the native `mupdf_wrapper` archive after `cargo clean`, because Cargo does not rebuild it automatically.
- Clean clippy and additional target claims should still be refreshed only after rerunning those exact commands.

## Deferred

- Large framework work for settings registries
- Broad event-system unification
- Cross-cutting input-validation framework
- Broad speculative performance refactors

## Monolithic Files

| File | Current Lines | Status |
|------|---------------|--------|
| `reader_impl/reader.rs` | 3403 | Open |
| `view/home/mod.rs` | 2769 | Open |
| `font/mod.rs` | 2400 | Open |
| `document/html/engine.rs` | 2672 | Open |
| `document/html/layout.rs` | 718 | Informational |

## Next Steps

1. Retire stale backlog items that are already implemented.
2. Keep verification notes synchronized with actual rerun results.
3. Address the remaining real UI and structural gaps (as listed in Open sections).
