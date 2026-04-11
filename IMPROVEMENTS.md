# Plato Codebase Improvement Summary

## Current Status

| Area | Status |
|------|--------|
| Build verification | Restored on current branch |
| Host `cargo check` | Passes |
| Documentation backlog | Needs source-driven cleanup |
| Structural refactors | Partial and still in progress |
| Dependency alignment | Mixed; not fully unified |

The current branch now passes `cargo check --target x86_64-unknown-linux-gnu`. Recent fixes restored the theme/settings integration by re-exporting the schedule types through `crate::settings`, aligning `update_from_schedule` with `DateTime<Local>`, and wiring scheduled theme mode through the active settings and app paths.

## Completed

These items are still verifiable in the current source tree:

- Safe wrapper modules for MuPDF, FreeType, and HarfBuzz exist under `crates/core/src/document/mupdf` and `crates/core/src/font`.
- `pdf.rs` and `pdf_manipulator.rs` use the safe wrapper layer rather than calling raw MuPDF bindings directly.
- ARM64 build profile support exists in the workspace and build docs.
- `with_child!` is implemented in `crates/core/src/view/common.rs`.
- `add_menu()` is implemented in `crates/core/src/view/common.rs`.
- Generic menu toggle helpers exist in `crates/core/src/view/menu_helpers.rs`.
- Reader helper modules exist in `crates/core/src/view/reader/reader_impl/`, but the migration is only partial.
- The plugin system is implemented and wired enough to load scripts, infer triggers, and enforce `allow_network`.
- External storage settings and auto-import logic are implemented and referenced from the home view.
- Scheduled theme mode is implemented in the active code paths:
  - `crates/core/src/settings/theme.rs`
  - `crates/core/src/theme.rs`
  - `crates/core/src/view/settings/display.rs`
  - `crates/core/src/view/top_bar.rs`
  - `crates/plato/src/app.rs`
- The orphan duplicate `Home` definition in `crates/core/src/view/home/home.rs` has been removed, so `batch_mode` no longer has a second stale definition outside the active module tree.

## Open Structural Issues

### Adherence to AGENTS.md Mandates

The following structural issues currently violate the mandatory rules in `AGENTS.md`:

- **Monolithic Files (1000-line limit)**: Several files significantly exceed the 1000-line limit and must be split into submodules.
- **Test Segregation**: Initial extraction of unit tests to sibling files completed for `device.rs`, `dictionary/mod.rs`, and `dictionary/indexing.rs`. Remaining embedded tests still need to be migrated.
- **Safe Wrapper Migration**: `font/mod.rs` still uses direct FFI calls and must be refactored to use the safe wrappers in `crates/core/src/font/`.

### Reader migration is incomplete

- `crates/core/src/view/reader/reader_impl/reader.rs` is still `3403` lines (Target: < 1000 lines).
- The file still ends with a large stub-method block that duplicates real reader behavior behind placeholder render calls.
- Helper modules like `reader_rendering.rs`, `reader_settings.rs`, and `reader_gestures.rs` exist, but many extracted functions are still marked `#[allow(dead_code)]` and are not the active implementation path.

Recommended next action:
- Decide whether the reader split will continue or be rolled back.
- If it continues, move active logic out of the stub block and remove dead extracted helpers as they are replaced.
- Ensure all functions in the new modules are under 50 lines.

Acceptance condition:
- No duplicate reader interface layer remains in `reader.rs`.
- Extracted reader modules contain active code paths rather than parked helpers.
- File is under 1000 lines.

### Home view is still oversized and has duplicated state

- `crates/core/src/view/home/mod.rs` is still `2769` lines (Target: < 1000 lines).
- The home view remains a large concentration point for event handling, menus, and batch operations.

Recommended next action:
- Split by active responsibility rather than by arbitrary file size.

Acceptance condition:
- Home responsibilities are partitioned by behavior, not duplicated across wrapper/core types.
- File is under 1000 lines.

### Safe Wrapper Migration for Fonts

- `font/mod.rs` is `2400` lines and still uses direct FFI calls (e.g., `FtFace`).

Recommended next action:
- Replace `FontLibrary`, `FontOpener`, and `Font` with implementations that use the safe wrappers in `crates/core/src/font/`.
- Split the monolithic `font/mod.rs` into smaller modules.

Acceptance condition:
- Zero direct FFI calls in `font/mod.rs`.
- File is under 1000 lines.

### PDF tools UI is only partially surfaced

- `crates/core/src/view/pdf_manipulator.rs` can now be launched for a selected PDF from the reader title menu and the home book menu.
- The view still has dead-code manipulation modes and incomplete parameter collection.
- Redaction and manipulation paths still depend on hard-coded defaults rather than interactive file/action input.

Recommended next action:
- Either integrate PDF tools with a real file-selection flow or explicitly scope the view down to currently-supported operations.

Acceptance condition:
- Every visible PDF tools action is reachable from the UI and uses user-selected inputs.

### Cover editor is implemented below the UI surface but still partial

- `crates/core/src/view/cover_editor.rs` contains real editing helpers for crop, rotate, brightness, contrast, grayscale, and save.
- The view can now be launched for a selected EPUB from the home book menu.
- The view is still broadly guarded by `#[allow(dead_code)]`.
- There is no interactive toolbar or surfaced editing workflow matching the helper capabilities.

Recommended next action:
- Choose one direction: complete the UI flow or reduce the feature to a simpler, intentionally limited cover replacement tool.

Acceptance condition:
- The user-facing feature set matches the implemented code path and no longer depends on parked dead-code scaffolding.

## Deferred by Design

These are not good near-term priorities based on the current code:

- Object pooling for views or geometry values.
- Gesture algorithm optimization beyond the current constant-time path.
- Large "settings registry" or UI-generation framework work.
- Broad "event handler unification" without a concrete defect driving it.
- Input validation framework extraction across all view inputs.
- Bitmap font specialization work.
- Smooth theme transition animations.
- Per-document theme preferences.

These may be revisited later, but they should not appear as active backlog items unless a current bug or implementation effort depends on them.

## Verification Status

- Host `cargo check --target x86_64-unknown-linux-gnu` currently passes on the checked-out branch.
- Clean clippy or cross-target claims should still be made only after rerunning those commands explicitly.

## Dependencies

### Current Notes

- `plato-core` uses `reqwest 0.12`.
- `fetcher` uses `reqwest 0.13.1`.
- This is not "aligned via workspace" and should be documented as a deliberate split or a pending cleanup.
- `fxhash` replacement with `rustc-hash` remains true in the current source.
- The workspace still contains documented security/dependency hygiene work, but claims should reflect the actual manifests, not earlier review snapshots.

## Stale or Retired Claims

The following should no longer be tracked as open opportunities:

- Missing `with_child!` macro.
- Missing `add_menu()` helper.
- Missing generic menu toggle helpers.
- Blanket claims that plugin or external storage settings are "unclear" when source integration exists.

These are now historical review notes, not current backlog items.
