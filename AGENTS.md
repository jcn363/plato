# AGENTS.md

This file provides guidance for AI coding agents working in the Plato codebase.

## Project Overview

Plato is a document reader for Kobo e-readers, written in Rust. It's a Cargo workspace with the following crates:

- **crates/core** (`plato-core`) — Core library with document handling, rendering, UI views, device interaction
- **crates/plato** — Main binary for Kobo devices
- **crates/emulator** — SDL2-based desktop emulator for development
- **crates/importer** — Document importer tool
- **crates/fetcher** — Article fetcher from online sources
- **crates/epub_edit** — EPUB editing library

## Build & Run Commands

The default build target is **arm-unknown-linux-gnueabihf** (32-bit ARM for Kobo). See `.cargo/config.toml`.

```bash
# Build for 32-bit ARM (original Kobo devices) — DEFAULT
cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato

# Build for 64-bit ARM (newer Kobo devices: Libra 2, Sage, Clara 2E, etc.)
cargo build --target aarch64-unknown-linux-gnu --profile release-arm64

# Build for host (development/testing)
cargo build --target x86_64-unknown-linux-gnu

# Full build with native dependencies (downloads libs + MuPDF)
./build.sh

# Create distribution bundle
./dist.sh

# Run the desktop emulator (requires SDL2)
./run-emulator.sh
```

## Testing

Since the default target is ARM, all test commands on the host require `--target x86_64-unknown-linux-gnu`:

```bash
# Run all tests
cargo test --target x86_64-unknown-linux-gnu

# Run tests for a specific crate
cargo test -p plato-core --target x86_64-unknown-linux-gnu

# Run a single test by name
cargo test -p plato-core test_device_canonical_rotation --target x86_64-unknown-linux-gnu

# Run tests in a specific module
cargo test -p plato-core geom::tests --target x86_64-unknown-linux-gnu

# Run tests matching a pattern
cargo test overlaping --target x86_64-unknown-linux-gnu
```

Tests use standard Rust `#[cfg(test)]` / `#[test]` attributes. See the [Test Segregation](#test-segregation) section for placement rules.

## Code Style

### Formatting

- Add `rustfmt.toml` to the project root to enforce consistent code style across all contributors and CI pipelines
- Use default `rustfmt` settings as a baseline, then customize via `rustfmt.toml`
- Run `cargo fmt` before committing
- Run `cargo clippy` to catch common issues

### Imports

- Group imports: std library first, then external crates, then local `crate::` imports
- Use explicit imports rather than glob (`use std::fmt` not `use std::fmt::*`)
- Re-export commonly used types from `lib.rs` (see `crates/core/src/lib.rs:30-40`)

```rust
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, format_err, Error};
use serde::{Deserialize, Serialize};

use crate::helpers::load_json;
use crate::metadata::Info;
```

### Naming

- **snake_case**: functions, methods, variables, modules, constants
- **PascalCase**: types, structs, enums, traits
- **SCREAMING_SNAKE_CASE**: true constants (`const DEFAULT_FONT_SIZE`)
- Prefix unused parameters or dead code markers with `_`

### Types & Structs

- Derive common traits: `#[derive(Debug, Clone)]` for structs, add `Copy, Eq, PartialEq` when appropriate
- Use `#[serde(rename_all = "camelCase")]` or `kebab-case` for serialization
- Use `#[serde(skip_serializing_if = "...")]` to omit empty/default fields
- Prefer `pub` fields on structs over getters for internal types
- Use the builder pattern for complex configurations — provide a `Builder` struct or fluent setter methods (`.foo(..).bar(..).build()`) for structs with many optional fields
- Ensure proper resource cleanup in error cases — implement `Drop` for types that own resources (file handles, FFI pointers, network connections)
- Monitor memory usage on resource-constrained devices — use `Box` for large data structures to avoid stack overflow and enable heap allocation

### Input Validation

**Mandatory rule:** Validate all inputs, especially at public API boundaries.

- Add input validation for all public APIs — never trust external data
- Use the `validator` crate for complex validation scenarios (email formats, string length, numeric ranges, regex patterns)
- Validate early and fail fast — reject invalid inputs before any side effects occur
- Provide clear, actionable error messages that tell the caller exactly what was invalid and why
- Validate configuration values, user input, file contents, and network responses at their entry points

### Error Handling

**Mandatory rule:** Standardize on a single error handling approach. Use `anyhow` for application-level error handling (binaries, top-level logic) and `thiserror` for library-level error types. Never mix both in the same module for the same concern.

- Use `anyhow::Error` as the primary error type throughout
- Use `bail!` for early returns with errors
- Use `format_err!` to create ad-hoc errors
- Use `.with_context(|| "...")` to add context to errors — always provide meaningful context that includes what operation failed and relevant identifiers (file paths, IDs, etc.)
- Use `thiserror` for defining custom error types in libraries
- Avoid `unwrap()` — prefer `?`, `unwrap_or`, `unwrap_or_default`, or explicit `match`
- For lock poisoning, use `.expect("lock_name lock poisoned")` instead of `.unwrap()`

```rust
use anyhow::{bail, format_err, Context, Error};

pub fn load_json<T, P: AsRef<Path>>(path: P) -> Result<T, Error>
where
    for<'a> T: Deserialize<'a>,
{
    let file = File::open(path.as_ref())
        .with_context(|| format!("can't open file {}", path.as_ref().display()))?;
    // ...
}
```

### Performance

- Use `#[inline]` on hot-path small functions (pixel operations, geometry math, device checks)
- Use `FxHashMap`/`FxHashSet` from `fxhash` instead of std `HashMap` for non-cryptographic use
- Pre-allocate buffers with `String::with_capacity` when size is known
- Prefer `Cow<str>` for conditional string ownership

### DRY (Don't Repeat Yourself)

**Mandatory rule:** Never duplicate code. If you find yourself writing the same logic in more than one place, extract it into a shared function, trait, or module.

- If two or more functions contain the same sequence of operations, extract the common logic into a helper function
- If multiple files repeat the same initialization pattern (e.g., MuPDF context creation), create a single shared factory function
- If the same `match` arm or `if` branch pattern appears in multiple locations, refactor into a method on the relevant type
- Use generics, traits, or macros to eliminate structural duplication (e.g., similar `WalkDir` iteration with identical filter logic)
- Constants repeated across files belong in a shared `consts` module

**When extracting, prefer the smallest cohesive unit:** a closure for local reuse, a private function for file-local reuse, a `pub(crate)` function for cross-module reuse within the crate.

### Modular Design

**Mandatory rule:** Keep files and functions focused and reasonably sized. Break up monolithic code.

- No source file should exceed **1000 lines** — split into submodules when approaching this limit
- No function should exceed **50 lines** — extract inner logic into helpers when approaching this limit
- Break down large functions into smaller, focused ones — each function should do one thing well
- Each module should have a single clear responsibility (e.g., rendering, parsing, I/O, UI)
- When a `mod.rs` file grows large, extract related logic into sibling files (e.g., `home/mod.rs` + `home/shelf.rs` + `home/book.rs`)
- Split large modules into smaller, more focused ones when they handle multiple distinct concerns
- Separate data structures, business logic, and I/O into distinct modules
- Use `pub(crate)` visibility to share helpers within a crate without exposing them publicly

**Signs a file needs splitting:** multiple unrelated struct+impl blocks, mixed concerns (e.g., parsing + rendering + I/O), or any single file over 800 lines.

### Modular Architecture

**Mandatory rule:** Design for clear separation of concerns and testability.

- Add interfaces/traits for major components to improve testability — define traits for services, repositories, and external integrations
- Mock trait implementations in tests rather than relying on concrete types
- Each layer should depend only on abstractions (traits), not concrete implementations
- Group related functionality behind well-defined module boundaries with minimal public surface area

### Module Hierarchy

**Mandatory rule:** Structure modules logically, avoid circular dependencies, and document purposes.

- Group related functionality by domain (e.g., `document/pdf`, `document/epub`, `view/reader`)
- Avoid circular dependencies between modules — if two modules reference each other, extract shared types to a third module
- Document each module's purpose at the top of its `mod.rs` file

### Architecture Documentation

- Add high-level architecture diagrams and document design decisions and trade-offs
- Document the rationale behind major structural choices (e.g., why a trait-based abstraction was chosen over concrete types)
- Keep architecture docs in `docs/architecture/` and reference them from module-level documentation

### Single Source of Truth

**Mandatory rule:** Every piece of knowledge or logic must have one authoritative location. Never scatter the same concept across multiple places.

- If a value can change, define it once and reference it everywhere (e.g., `const` or `lazy_static!` instead of inline literals)
- If a type has multiple representations (e.g., string names, IDs), store the mapping in one place and derive the rest
- When extracting constants, define them in the module that owns the concept, then `pub` or `pub(crate)` export them
- Avoid shadowing or overriding the same data in multiple layers — if a setting is in `Context`, don't also cache it locally without a clear invalidation strategy
- When refactoring a duplicated pattern, consolidate into the *canonical* location and remove the copies

### Configuration Management

**Mandatory rule:** Centralize configuration management and validate all configuration values.

- Group related configuration in dedicated structs or modules — avoid scattering config across unrelated files
- Add validation for configuration values at load time — reject invalid values early with clear error messages
- Use typed configuration over raw strings or magic numbers — define enums for known sets of valid values
- Document all configuration options, their valid ranges, and default values
- Validate configuration values against constraints (e.g., font size ranges, color values, timeout limits) before use

### Test Segregation

**Mandatory rule:** Strictly separate test code from production code to avoid contamination and overhead.

- **Unit tests** must be in the same directory as production code using sibling test files (e.g., `loop.rs` and `loop_tests.rs`)
- Test files should include a `mod loop;` (or `use super::*;`) to access the production code they test
- **Integration tests** go in `tests/` directory at the workspace or crate root
- Test-only helpers, fixtures, and utilities must live in test files or separate test-only crates
- Never gate production behavior on `cfg(test)` — the compiled binary should be identical whether tests exist or not
- Avoid test-specific dependencies leaking into the main dependency tree; use `[dev-dependencies]` in `Cargo.toml`
- Each test file should be named `{module}_tests.rs` and placed alongside its corresponding production module
- Group related tests using modules — organize tests by feature or component for clarity
- Add integration tests that exercise multiple components together to verify end-to-end behavior

### General Patterns

- Use `lazy_static!` for global statics that require runtime initialization
- Use `bitflags!` for flag enums
- Prefer `BTreeMap`/`BTreeSet` for ordered collections; `IndexMap` for insertion-ordered maps
- Keep `mod` declarations alphabetical; use `pub mod` for public API, plain `mod` for internal

### Dependency Management

**Mandatory rule:** Regularly audit dependencies for security and maintainability.

- Use `cargo-audit` to check for known vulnerabilities — run it before releases and periodically during development
- Audit and update dependencies regularly — don't let them drift far behind
- Use workspace inheritance for shared dependency versions — define versions in the root `Cargo.toml` `[workspace.dependencies]` section
- Pin major versions and avoid wildcard dependencies

### Async Patterns

- Document `Send` and `Sync` bounds for async code — ensure types that cross thread boundaries implement the correct traits
- Add deadlock detection for code using multiple locks — use `tracing` spans to track lock acquisition order
- Use `tracing` for better async debugging — replace `log` with `tracing` for structured, context-aware logging
- Prefer `tokio` or `async-std` runtime primitives over raw `Future` manipulation

### API Documentation

- Add examples for all public APIs in rustdoc comments — use `/// # Examples` blocks with runnable code
- Document safety requirements for `unsafe` functions and methods
- Use `///` for public API documentation and `//` for internal notes
- Keep examples minimal but complete — they should compile and run without additional setup

## Automation

**Mandatory rule:** Use scripts for building, testing, linting, formatting, and deployment to reduce errors and speed up cycles.

- Always run `cargo fmt` and `cargo clippy` before considering a task complete
- Use `cargo test` to verify changes compile and pass tests — run it proactively, not just when asked
- Prefer `cargo check` over `cargo build` during development for faster feedback
- When modifying multiple files, batch changes and run a single validation pass at the end
- For this project, the emulator requires native libs — use `./run-emulator.sh` which sets `RUSTFLAGS` correctly
- Cross-compilation targets ARM by default (see `.cargo/config.toml`) — use `--target x86_64-unknown-linux-gnu` for host builds

## Error Handling Process

**Mandatory rule:** Address errors in small increments, commit frequently, and review for accuracy.

- Fix one category of error at a time (e.g., all `unwrap()` in one file, then all in the next)
- Run `cargo check` or `cargo test` after each small batch of changes to catch regressions early
- Commit working changes frequently with clear messages describing what was fixed
- Review changes for: **grammatical** accuracy (comments, docs), **factual** accuracy (API usage, types), **logical** correctness (control flow, edge cases)
- When a fix introduces new errors, stop and understand the dependency chain before continuing
- Never leave the codebase in a broken state — if a refactor is too large, revert and split into smaller steps

### Error Resolution Sequence

When facing multiple compilation errors, resolve in this order:

1. **Dependency issues** — fix version conflicts and ensure compatibility (`Cargo.toml`)
2. **Import resolution** — validate all module imports and path configurations (`use` statements)
3. **Type mismatches** — harmonize type definitions and error handling patterns
4. **Missing implementations** — add missing methods, traits, types
5. **Validate compilation and testing** — ensure all tests pass and functionality is preserved

## Task Discipline

**Mandatory rule:** Stay focused, validate incrementally, and prefer composition.

- **One task at a time** — avoid concurrent operations to maintain focus and reliability
- **Decompose incrementally** — break complex tasks into manageable steps with frequent validation
- **Prefer composition over inheritance** — build flexible systems through component composition and traits

## Build Verification

**Mandatory rule:** Achieve zero warnings and zero errors on every build target.

### Systematic Build Process

1. **Incremental verification** — After each code change, compile for the primary target (ARM Kobo) immediately
2. **Zero-tolerance policy** — Treat warnings as errors; never introduce new warnings into the codebase
3. **Full build verification** — Before considering any task complete, run a clean build:
   ```bash
   # Primary target: ARM Kobo (32-bit)
   cargo clean && cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato

   # Secondary target: ARM64 Kobo (newer devices)
   cargo clean && cargo build --target aarch64-unknown-linux-gnu --profile release-arm64 -p plato

   # Host target: for testing
   cargo clean && cargo build --target x86_64-unknown-linux-gnu
   ```
4. **Clippy validation** — Run clippy on host target after significant changes:
   ```bash
   cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings
   ```

### Task Decomposition

- **One concern per change** — Isolate refactoring from functional changes in separate commits
- **Smallest viable diff** — Prefer several focused commits over one large, mixed commit
- **Verify before proceeding** — Compile successfully after each atomic change before moving to the next

### Code Quality Principles

- **Rewrite over patch** — When a file has accumulated significant technical debt (dead code, deprecated patterns, unclear structure), rewrite it completely rather than patching
- **Rust idioms only** — Every line must follow current Rust best practices; avoid deprecated patterns
- **Root cause analysis** — When encountering a bug or issue, identify and fix the root cause; do not apply surface-level workarounds
- **Eliminate dead code** — Remove unused functions, imports, fields, and modules immediately; never leave dead code for later
- **No backward compatibility** — Do not add code to support old APIs, deprecated patterns, or legacy behavior unless explicitly required
- **Project containment** — All created or used files and directories must be located inside the project root directory (`/home/user/Desktop/plato`); never create or access files outside the project workspace

### Context Management

- **Flush after each task** — After completing a focused task (fix, refactor, feature), ensure the context is clean:
  - All builds pass
  - All tests pass
  - No warnings or errors
  - Code is formatted (`cargo fmt`)
  - Clippy passes
- **Avoid state accumulation** — Do not layer changes on top of unverified state; verify each step before proceeding

## Architecture Notes

Each crate should have a single responsibility, explicit documentation in its `Cargo.toml`, and specified dependencies.

- The `Context` struct (`crates/core/src/context.rs`) holds runtime state: framebuffer, settings, library, fonts, input history
- Views implement the `View` trait and handle `Event`s; rendering goes through `RenderQueue`
- Device-specific code uses `CURRENT_DEVICE` lazy static with environment variables `PRODUCT` and `MODEL_NUMBER`
- MuPDF bindings live in `crates/core/src/document/mupdf_sys.rs` with a C wrapper in `mupdf_wrapper/`
- Safe MuPDF wrappers are in `crates/core/src/document/mupdf.rs` — use these instead of direct FFI calls
- Safe FreeType wrappers are in `crates/core/src/font/freetype.rs` — use these instead of direct FFI calls
- Safe HarfBuzz wrappers are in `crates/core/src/font/harfbuzz.rs` — use these instead of direct FFI calls
- **NEW code must use safe wrappers** — all user code should import from `crate::document::mupdf`, `crate::font::freetype`, `crate::font::harfbuzz` instead of `mupdf_sys`, `freetype_sys`, `harfbuzz_sys`
- Legacy code in `font/mod.rs` still uses direct FFI — all FFI calls are covered by safe wrappers, migration requires architectural restructuring (replacing `FontLibrary`/`FontOpener`/`Font` with composed safe wrapper types)
- `pdf.rs` and `pdf_manipulator.rs` have been migrated to use safe wrappers
- All safe wrappers include `#[inline]` for hot-path optimization and `Drop` implementations for RAII resource cleanup
- `MuPdfContext` uses `Rc` internally for shared ownership across multiple documents
- `Outline`, `Link`, `Annotation` wrappers return owned values from `next()`/`down()` with proper RAII cleanup
- Text iteration uses `Iterator` trait: `TextPage::blocks()` → `TextBlockIter`, `TextBlock::lines()` → `TextLineIter`, `TextLine::chars()` → `TextCharIter`
- `Face::face_ptr()` returns raw `*mut FtFace` for HarfBuzz integration
- The pre-compiled `libs/libmupdf.so` is incomplete — it lacks many PDF manipulation, annotation, and redaction symbols
- `mupdf_wrapper/mupdf_wrapper.c` provides 20+ custom FFI functions (e.g., `fz_pdf_count_pages`, `fz_save_document`, `fz_first_annot`, `fz_apply_redactions`) that bridge the gap
- The wrapper is built as `libmupdf_wrapper.a` and linked via `crates/core/build.rs` for ARM/ARM64 targets
- When adding new MuPDF FFI functions, implement them in `mupdf_wrapper.c` using the `WRAP` macro or explicit `fz_try`/`fz_catch` blocks
- Rebuild the wrapper after modifying `mupdf_wrapper.c`: `cd mupdf_wrapper && TARGET_OS=Kobo CC=arm-linux-gnueabihf-gcc AR=arm-linux-gnueabihf-ar ./build.sh`
- Use `new_mupdf_context()` from `mupdf_sys` to create MuPDF contexts (DRY helper for FFI init)
- Use `MuPdfContext` from `mupdf.rs` for safe context management with RAII cleanup
- Use `log_error!`, `log_warn!`, `log_info!` macros from `crate::helpers` instead of raw `eprintln!`
- The pre-compiled `libs/libmupdf.so` is incomplete — it lacks many PDF manipulation, annotation, and redaction symbols
- `mupdf_wrapper/mupdf_wrapper.c` provides 20+ custom FFI functions (e.g., `fz_pdf_count_pages`, `fz_save_document`, `fz_first_annot`, `fz_apply_redactions`) that bridge the gap
- The wrapper is built as `libmupdf_wrapper.a` and linked via `crates/core/build.rs` for ARM/ARM64 targets
- When adding new MuPDF FFI functions, implement them in `mupdf_wrapper.c` using the `WRAP` macro or explicit `fz_try`/`fz_catch` blocks
- Rebuild the wrapper after modifying `mupdf_wrapper.c`: `cd mupdf_wrapper && TARGET_OS=Kobo CC=arm-linux-gnueabihf-gcc AR=arm-linux-gnueabihf-ar ./build.sh`
- Use `new_mupdf_context()` from `mupdf_sys` to create MuPDF contexts (DRY helper for FFI init)
- Use `MuPdfContext` from `mupdf.rs` for safe context management with RAII cleanup
- Use `log_error!`, `log_warn!`, `log_info!` macros from `crate::helpers` instead of raw `eprintln!`

## Kobo Elipsa Specifications

- **Screen**: 10.3" E Ink Carta 1200 display
  - 227 PPI
  - 1404 x 1872 resolution
  - Dark Mode available
- **Front light**: ComfortLight – single colour with adjustable brightness
- **Size**: 193 x 228 x 8 mm
- **Weight**: 383 grams (SleepCover adds 345 grams)
- **Storage**: Internal flash eMMC 32 GB
- **RAM**: LPDDR4 1 GB
- **Processor**: Allwinner B300 SoC Quad Core @ 1.8GHz, an ARMv7 (32-bit)
- **GPU**: Mali400 MP2
- **eInk Controller**: B300+SY7636
- **Hall Sensor**: TLE4913
- **Accelerometer**: KX122
- **WiFi / Bluetooth Chipset**: Realtek RTL8821CS
- **Button**: Power on/off
- **Stylus**: Kobo Stylus (compatible with MPP — Microsoft Pen Protocol)
- **Colors**: Space Deep Blue
- **Customizability**: TypeGenius — 12 fonts, 50+ font styles, exclusive weight/sharpness settings
- **Supported formats**: 16 natively (EPUB, EPUB3, KEPUB, FlePub, PDF, MOBI, JPEG, GIF, PNG, BMP, TIFF, TXT, HTML, RTF, CBZ, CBR)
- **Connectivity**: Wi-Fi 802.11 ac/b/g/n, Type-C USB, Bluetooth 4.2
- **Battery**: 2x1200 mAh (2400 mAh total), weeks of battery life
- **Languages**: English, French, German, Spanish, Italian, Catalan, Portuguese, Dutch, Danish, Swedish, Finnish, Norwegian, Turkish, Japanese, Traditional Chinese
- **Content**: Kobo eBookstore (6M+ titles), OverDrive, Dropbox, Adobe Digital Editions, Pocket, sideloading

## Could the Kobo Elipsa benefit from parallel programming?

The Kobo Elipsa can benefit from parallel programming for workloads that are CPU-bound, parallelizable, and not latency-sensitive. Practical considerations:

### When parallelism helps

- **Page rendering / compositing:** Rasterizing complex pages (PDFs with many objects, or large bitmaps) and blending layers (text, annotations, UI) can be split into tiles or scanline bands processed in parallel.
- **PDF/EPUB layout & reflow:** Laying out complex pages or reflowing large documents can parallelize per-chunk or per-page work.
- **Image decoding & scaling:** Decode multiple images or tiles concurrently (especially if using multi-frame or tiled images).
- **OCR / handwriting recognition (if local):** Model inference or feature extraction can be parallelized across CPU cores (or vectorized).
- **Background tasks:** Indexing library, thumbnail generation, or file format conversions can run on background threads without blocking the UI.
- **I/O pipelining:** Overlap flash reads, decompression, and rendering on separate threads to improve throughput.

### When it’s not helpful or is counterproductive

- **Simple UI interactions:** Eink refresh latency and the need to block for full or partial refresh often dominate perceived responsiveness more than CPU. Spawning threads won’t reduce eink update time.
- **Small, short-lived tasks:** Threading overhead and context switches can increase total runtime on a low‑power CPU.
- **Memory pressure:** The Elipsa has limited RAM; parallel tasks that increase peak memory usage (multiple large render buffers) risk paging or OOM.
- **Power consumption:** More active cores raise energy use and can reduce battery life; mobile workloads must trade speed for battery.

### Implementation notes and best practices

- Use a thread pool sized to available cores (usually 2–4 on such devices), not one thread per task.
- Partition work into coarse-grained tiles or page-level jobs to amortize thread overhead.
- Limit peak memory by streaming and reusing buffers. Process tiles sequentially when memory is tight.
- Prioritize interactive threads (UI, touch/pen input) and run heavy work at lower priority or in background.
- Prefer SIMD/vectorized libraries (NEON) for image & text rendering where possible; this can yield big gains without adding threads.
- Measure: benchmark real device workloads (PDF rendering, annotation response) to confirm gains.
- Use existing optimized libraries (MuPDF, Harfbuzz, Skia with context-appropriate builds) which already include multithreading or SIMD optimizations.

### Bottom line

Parallel programming can improve throughput for rendering, decoding, layout, and background processing on the Kobo Elipsa, but benefits are bounded by eink refresh latency, limited RAM, power constraints, and threading overhead. Focus on coarse-grained parallel tasks, buffer reuse, and SIMD acceleration, and always validate on the actual device.
