# Plato Integration Progress

> Current-source update based on the checked-out branch state

## Completed

- Common view helpers now exist for repeated child/menu patterns:
  - `with_child!`
  - `add_menu()`
  - `menu_helpers::{toggle_menu_vec, toggle_menu_with, toggle_menu_ctx, toggle_menu_item, toggle_menu_self}`
- Reader support modules exist under `crates/core/src/view/reader/reader_impl/`, so the codebase has already started a split away from one monolithic reader file.
- PDF manipulation can now be launched for a selected PDF, and cover editing can now be launched for a selected EPUB, but both features remain incomplete from a product/UI perspective.

## Open

### Reader migration cleanup

- `reader.rs` is still `3403` lines.
- The file still ends with a stub-method block.
- Extracted reader modules are present, but many helpers remain inactive or dead-code-gated.

### Home modularization

- `home/mod.rs` is still `2769` lines.

### PDF tools UI completion

- `pdf_manipulator.rs` is now reachable from selected-document flows, but still contains parked file-selection/action-selection code.
- The manipulation flow is still not fully parameterized through the UI.

### Cover editor product decision

- The editor contains real image-editing helpers.
- The view is now reachable from selected-document flow for EPUBs, but is still partially parked behind `#[allow(dead_code)]` and lacks a complete editing UI.

## Verification

Current verification status on the checked-out branch:

- `cargo check --target x86_64-unknown-linux-gnu` passes
- `cargo check --target arm-unknown-linux-gnueabihf -p plato` passes
- `cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato` passes after rebuilding `mupdf_wrapper`

Notes:

- A true clean ARM rebuild requires regenerating the native `mupdf_wrapper` archive after `cargo clean`, because Cargo does not rebuild it automatically.
- Clean clippy and additional target claims should still be refreshed only after rerunning those exact commands.

## Deferred

- Large framework work for settings registries
- Broad event-system unification
- Generic input-validation architecture
- Non-measured performance refactors

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
3. Address the remaining real UI and structural gaps:
   - reader stub cleanup
   - home modularization
   - PDF tools workflow
   - cover editor scope/completion
