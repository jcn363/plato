# Gemini CLI Context: Plato

This file provides foundational mandates and instructional context for Gemini CLI when working in the Plato codebase.

## Project Overview

**Plato** is an optimized document reader for Kobo e-readers, written in Rust. It supports a wide range of formats including EPUB, PDF, CBZ, FB2, MOBI, and HTML. The project is structured as a Cargo workspace with specialized crates for core logic, device-specific binaries, emulation, and helper tools.

### Architecture & Workspace Layout

- **`crates/core` (`plato-core`)**: The engine of the application. Handles document parsing, rendering, UI views, device interaction, and settings.
- **`crates/plato`**: The main binary specifically for Kobo devices.
- **`crates/emulator`**: An SDL2-based desktop emulator for cross-platform development and testing.
- **`crates/importer`**: The `plato-import` tool for processing documents.
- **`crates/fetcher`**: The `article_fetcher` binary for retrieving online content.
- **`crates/epub_edit`**: A library for EPUB editing functionality used within the UI.
- **`mupdf_wrapper/`**: A C-based FFI wrapper for MuPDF that provides critical PDF manipulation functions not found in standard pre-compiled libraries.
- **`epub_editor/`**: A standalone CLI tool for EPUB manipulation (excluded from the Cargo workspace).

## Building and Running

The primary target is **ARMv7** (`arm-unknown-linux-gnueabihf`) for Kobo devices.

### Key Commands

- **Full Build**: `./build.sh` (Downloads libraries, builds the MuPDF wrapper, and compiles all crates).
- **Kobo Binary (32-bit ARM)**: `cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato`
- **Kobo Binary (64-bit ARM64)**: `cargo build --target aarch64-unknown-linux-gnu --profile release-arm64 -p plato`
- **Host Binary (Development)**: `cargo build --target x86_64-unknown-linux-gnu -p plato`
- **Desktop Emulator**: `./run-emulator.sh` (Requires SDL2).
- **Distribution**: `./dist.sh` (Creates a distribution bundle).

### Testing

All tests executed on a host machine **MUST** specify the host target to avoid cross-compilation errors:

- **Run all tests**: `cargo test --target x86_64-unknown-linux-gnu`
- **Crate-specific tests**: `cargo test -p plato-core --target x86_64-unknown-linux-gnu`
- **Module-specific tests**: `cargo test -p plato-core geom::tests --target x86_64-unknown-linux-gnu`

## Engineering Standards

### Core Mandates

- **Zero Warning Policy**: All changes must achieve zero warnings and zero errors on all supported targets (ARM, ARM64, and x86_64).
- **Surgical Updates**: Apply minimal, targeted changes. Verify after every atomic change.
- **Validation**: Mandatory input validation at all public API boundaries. Provide clear, actionable error messages.

### Coding Guidelines

- **Error Handling**: 
  - Use `anyhow` for application-level logic and `thiserror` for library-level types. 
  - Standardize on `anyhow::Error` globally.
  - **NEVER** use `unwrap()`; prefer `?`, `context()`, or `expect()` (only for lock poisoning).
- **Resource Management**: Implement `Drop` for all types owning resources (FFI pointers, file handles) to ensure RAII-based cleanup.
- **Safe Wrappers**: Always use safe Rust wrappers (e.g., `crate::document::mupdf`) rather than direct FFI calls (`mupdf_sys`).
- **Performance**: 
  - Mark small, hot-path functions (pixel ops, geometry, device checks) with `#[inline]`.
  - Use `std::sync::LazyLock` for constants and regex patterns.
  - Prefer `rustc_hash::FxHashMap/FxHashSet` for non-cryptographic maps.
- **Formatting**: Adhere to `rustfmt.toml`. Always run `cargo fmt` and `cargo clippy` before completion.

### Modularization & Architecture

- **File Limits**: No source file should exceed **1000 lines**. Split into submodules if this limit is approached.
- **Function Limits**: No function should exceed **50 lines**. Extract logic into focused helper functions.
- **Test Segregation**: 
  - Unit tests belong in sibling files named `{module}_tests.rs` in the same directory as the production code.
  - Integration tests belong in the `tests/` directory at the crate root.
- **DRY Principle**: Consolidate repeated patterns into authoritative, shared modules (e.g., a `consts` module for shared literals).

## Strategic Orchestration

For complex tasks (batch refactoring, large search/replace, or fixing multiple lint errors), invoke the `generalist` sub-agent to maintain efficiency and keep the primary session history lean.
