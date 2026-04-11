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
- Reader support modules exist in `crates/core/src/view/reader/reader_impl/`

## Open

### Reader

- File: `crates/core/src/view/reader/reader_impl/reader.rs`
- Current size: `3403` lines
- Active issue: file still ends with a duplicate stub-method block
- Follow-up: complete or unwind the partial reader split

### Home

- File: `crates/core/src/view/home/mod.rs`
- Current size: `2769` lines
- Active issue: the active home implementation is still oversized
- Follow-up: split by active responsibility, not just line count

### PDF Tools

- File: `crates/core/src/view/pdf_manipulator.rs`
- Active issue: now reachable for selected PDFs, but still carries dead-code action flow and hard-coded manipulation defaults
- Follow-up: connect a real user workflow or reduce the surfaced scope

### Cover Editor

- File: `crates/core/src/view/cover_editor.rs`
- Active issue: now reachable for selected EPUBs, but the editing UI is still partial and dead-code-gated
- Follow-up: complete the UI or intentionally narrow the feature

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
