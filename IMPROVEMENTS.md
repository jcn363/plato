# Plato Codebase Improvement Summary

## Current Status

| Area | Status |
|------|--------|
| Build verification | Restored on current branch |
| Host `cargo check` | Passes |
| Documentation backlog | Synchronized with workspace changes |
| Structural refactors | Partial; Reader and Home view pending |
| Dependency alignment | Partially unified; 'log' and 'env_logger' centralized |

The current branch passes `cargo check --target x86_64-unknown-linux-gnu` and builds successfully for `arm-unknown-linux-gnueabihf` with minimal warnings.

## Completed

These items are now verifiable in the current source tree:

- **Epub Editor Integration**: `epub_editor` is now a workspace member at `crates/epub_editor`. Shared dependencies (`log`, `env_logger`) are centralized in the root `Cargo.toml`.
- **Safe Wrapper Migration for Fonts**: `crates/core/src/font/mod.rs` (802 lines) now uses the safe wrapper layer (`face.rs`, `library.rs`, `harfbuzz.rs`) and is under the 1000-line limit.
- **Unit Test Segregation**: All unit tests in `crates/core/src` have been extracted into sibling `_tests.rs` files.
- **Safe wrapper modules** for MuPDF and FreeType exist under `crates/core/src/document/mupdf`.
- **ARM64 build profile support** exists in the workspace and build docs.
- **UI primitives**: `with_child!`, `add_menu()`, and generic menu toggle helpers are implemented in `crates/core/src/view/`.
- **Scheduled theme mode** is implemented and wired through active paths.
- **Cover Editor UI**: Interactive controls and visual crop selection are fully implemented.

## Open Structural Issues

### Adherence to AGENTS.md Mandates

- **Monolithic Files (1000-line limit)**: 
  - `crates/core/src/view/reader/reader_impl/reader.rs` (**3403 lines**)
  - `crates/core/src/view/home/mod.rs` (**2786 lines**)
- **Reader trait stubs**: `reader.rs` contains a large block of stubbed trait methods (lines 3206+) that must be replaced with active logic from the helper modules.

### Reader migration is incomplete

- helper modules like `reader_rendering.rs` and `reader_gestures.rs` exist but are not fully utilized.
- Active logic remains trapped in the monolithic `reader.rs`.

### Home view is still oversized

- The home view remains a large concentration point for event handling and batch operations.

### PDF tools UI workflow completion

- Interactive redaction region definition UI implemented.
- File selection flow for PDF merging implemented.

## Verification Status

### Host Verification (x86_64 Linux)
- **Command:** `cargo check --target x86_64-unknown-linux-gnu`
- **Result:** Pass (2026-04-13)

### ARM Kobo Verification (32-bit ARM)
- **Command:** `./build.sh arm fast`
- **Result:** Pass (2026-04-13)
- Successfully suppresses unstable feature warnings.
