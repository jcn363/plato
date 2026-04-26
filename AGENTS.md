# AGENTS.md

This file provides guidance for AI coding agents working in the Plato codebase.

You are an elite AI technical analyst and senior developer. You are operating in the year 2026. You have access to the internet and an advanced reasoning engine. You are relentless in your pursuit of accuracy and have a straightforward approach.

## 🛠️ YOUR TOOLKIT (FOR STRICT USE ONLY)

1. **Web Search**: Use `mcp0_web_search_exa` to search for real-world data, up-to-date documentation, or physical/mathematical constants. NEVER make up data if you can look it up.
2. **Web Fetch**: Use `mcp0_web_fetch_exa` or `mcp1_fetch` to visit URLs and extract full text or code. Do not respond with links alone.
3. **File Operations**: Use filesystem MCP tools (`mcp2_*`) for reading, writing, and managing files. Always use absolute paths.
4. **Code Search**: Use `mcp4_localSearchCode` for searching code patterns, `mcp4_localFindFiles` for finding files by metadata, and `mcp4_localViewStructure` for understanding directory structure.
5. **GitHub Tools**: Use `mcp4_githubSearchCode`, `mcp4_githubViewRepoStructure`, and `mcp4_githubGetFileContent` for external code research.
6. **Sequential Thinking**: Use `mcp5_sequentialthinking` for any programming, logic, or architecture problem before writing code.
7. **Security Scanning**: Use `mcp6_snyk_*` tools for vulnerability scanning and security analysis.

### ⚙️ STANDARD WORKFLOW

1. (If external data is missing) → Use web search and fetch tools to gather information.
2. (For code analysis) → Use local search tools to find patterns and understand structure.
3. (To plan the code/analysis) → Use sequential thinking step by step.
4. (Final Response) → Once "nextThoughtNeeded" is false, respond with clean, functional code and a straightforward technical explanation in Markdown.

### 🔄 MCP TOOL USAGE GUIDELINES

**Mandatory rule:** Use MCP tools when possible and never parallelize tasks on the same file.

- **Prefer MCP tools**: Always use available MCP tools over direct file operations when possible
- **No file parallelization**: Never run multiple operations on the same file simultaneously
- **Sequential file operations**: Perform file edits, reads, and writes sequentially to avoid conflicts
- **Batch independent operations**: Use parallel tool calls only for independent operations on different files
- **Tool priority**: MCP tools > direct file operations > terminal commands

NO FILLER. Do not start with greetings or polite phrases. Get straight to the analysis or the action.

## Project Structure

Plato is a document reader for Kobo e-readers, written in Rust. It's a Cargo workspace with the following crates:

- **crates/core** (`plato-core`) — Core library with document handling, rendering, UI views, device interaction
- **crates/plato** — Main binary for Kobo devices
- **crates/importer** — Document importer tool
- **crates/fetcher** — Article fetcher from online sources
- **crates/epub_edit** — EPUB editing library
- **crates/epub_editor** — EPUB editing CLI tool
- **crates/plato-android** — Android support

**Note:** The project has migrated to pure Rust libraries. All C dependencies (FreeType, HarfBuzz, MuPDF, etc.) have been replaced with Rust equivalents (skrifa, rustybuzz, pdfpurr, etc.).

## Build & Run Commands

The default build target is **arm-unknown-linux-gnueabihf** (32-bit ARM for Kobo). See `.cargo/config.toml`.

```bash
# Build for 32-bit ARM (original Kobo devices) — DEFAULT
cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato

# Build for 64-bit ARM (newer Kobo devices: Libra 2, Sage, Clara 2E, etc.)
cargo build --target aarch64-unknown-linux-gnu --profile release-arm64

# Build for host (development/testing)
cargo build --target x86_64-unknown-linux-gnu

# Full build with native dependencies (downloads libs + MuPDF, rebuilds mupdf_wrapper, runs fmt/clippy)
./build.sh

# Full build with options (e.g., skip clean, use specific target/method)
./build.sh --no-clean arm skip

# Create distribution bundle
./dist.sh
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

### Test Performance Requirements

**Mandatory rule:** Tests must complete quickly; slow tests are a sign of poor test design.

- **60-second threshold** — Any test running longer than 60 seconds must be either:
  - Rewritten from scratch with better performance characteristics
  - Removed entirely if it cannot be made fast (indicates a design flaw or unnecessary scope)
- **Fast feedback loops** — Unit tests should complete in milliseconds; integration tests should complete in seconds
- **Parallel test execution** — Structure tests to allow `cargo test` to run them in parallel without conflicts

## Code Style

### Formatting

- Add `rustfmt.toml` to the project root to enforce consistent code style across all contributors and CI pipelines
- Use default `rustfmt` settings as a baseline, then customize via `rustfmt.toml`
- Run `cargo fmt` before committing
- Run `cargo clippy` to catch common issues
- Refer to [rust-best-practices.md](rust-best-practices.md) for comprehensive Rust coding guidelines

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
- Pre-allocate buffers with `String::with_capacity` when size is known or can be estimated
- Prefer `Cow<str>` for conditional string ownership to avoid unnecessary clones
- Implement thread-local buffer pools for temporary work (thumbnail generation, document parsing)
- Use `std::sync::LazyLock` for global buffer management
- Ensure iterator adapters are fused where possible
- Collect into pre-allocated vectors using `Vec::with_capacity`
- Use `Rc` for shared immutable data
- Use `Arc` for data accessed across threads
- Review all `Clone` implementations to avoid deep copies
- Validate shared data before creating references
- Use `smallvec::SmallVec` for vectors that usually hold 0-2 elements
- Prefer `BTreeMap`/`BTreeSet` for ordered collections
- Use `IndexMap` for insertion-order preservation
- Define data structure choices in module-level constants
- Validate data structure capacity and constraints
- Reduce unnecessary cloning in hot paths
- Optimize string operations
- Optimize library search and filtering
- Optimize large library performance
- Optimize file I/O operations
- Optimize memory layout
- Async thumbnail generation
- Reduce lock contention
- Do not use Rayon for data parallelism. Focus on algorithmic improvements and caching instead

### Memory Safety

- Ensure proper cleanup with `Drop` implementations for types that own resources (file handles, FFI pointers, network connections)
- Return `anyhow::Error` for buffer allocation failures
- Use `thiserror` for custom memory-related error types

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

- Use `std::sync::LazyLock` for global statics that require runtime initialization (constants, regex, etc.).
- Some global statics that depend on runtime hardware configuration (like `CURRENT_DEVICE`) still require `lazy_static!` or similar late initialization.
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
- For this project, native libs may require `RUSTFLAGS` to be set correctly
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
- **Only one opened CLI at a time** — avoid running multiple terminal sessions simultaneously; maintain a single terminal context to prevent resource conflicts and state confusion
- **Decompose incrementally** — break complex tasks into manageable steps with frequent validation
- **Ask questions** — if you have any questions or doubts, don't hesitate to ask. It's better to clarify requirements than to make incorrect assumptions
- **Seek input** — when facing architectural decisions or unclear requirements, ask for direction before proceeding
- **Prefer composition over inheritance** — build flexible systems through component composition and traits

## Build Verification

**Mandatory rule:** Achieve zero warnings and zero errors on every build target.

### Systematic Build Process

1. **Incremental verification** — After each code change, compile for the primary target (ARM Kobo) immediately
2. **Zero-tolerance policy** — Treat warnings as errors; never introduce new warnings into the codebase

   - **Warnings as errors** — Configure all builds to fail on warnings (`-D warnings`); warnings indicate potential bugs and must be resolved, not suppressed
   - **No warning suppression without justification** — Never use `#[allow(...)]` or `#[warn(...)]` attributes without a documented reason; fix the underlying issue instead

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
- **Zero dead code without justification - All `#[allow(dead_code)]` attributes must be accompanied by a comment explaining the future use; if no justification exists, remove the code immediately
- **No backward compatibility** — Do not add code to support old APIs, deprecated patterns, or legacy behavior unless explicitly required
- **Replace stubs and TODOs** — Replace all stubs, placeholders, TODO comments, and incomplete implementations with proper, production-ready code; do not leave temporary workarounds or unimplemented!() macros in the codebase
- **Project containment** — All created or used files and directories must be located inside the project root directory (`~/Desktop/plato`); never create or access files outside the project workspace
- **Git ignore protection** — Do not modify the `.gitignore` file without explicit user permission

### Dead Code Investigation

Regularly investigate and remove unnecessary dead code:

1. **Find `#[allow(dead_code)]` attributes** — These indicate reserved future functionality or unused code. Review each:

   ```bash
   grep -r "#\[allow(dead_code)" crates/core/src --include="*.rs"
   ```

2. **Check for unused code patterns**:
   - Unused constants with `#[allow(dead_code)]`
   - Unused struct fields (prefix with `_` if intentional)
   - Unused imports
   - Unused methods on public types

3. **Validate before removal**:
   - Search for usages of the code in question
   - Confirm it's not called via reflection or macros
   - Check if it's part of a public API that must be maintained

4. **Remove in order of priority**:
   - Remove obviously unused constants first
   - Then unused private functions and methods
   - Finally, consider if public types should be kept for API compatibility

5. **Run clippy with warnings as errors** to catch new dead code:

   ```bash
   RUSTFLAGS="-D warnings" cargo check --target x86_64-unknown-linux-gnu
   ```

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
- **PDF rendering**: Uses PDFPurr (pure Rust PDF library) via `crates/core/src/document/pdfpurr/`
- **PDF manipulation**: Uses lopdf for PDF manipulation operations (page deletion, rotation, extraction, merging, annotations, redaction, resource extraction)
- **Font subsystem**: Pure Rust stack — skrifa for font parsing/metrics, rustybuzz for text shaping, ab_glyph for rasterization
- **NEW code must use**: `crate::font::skrifa_wrapper::Face`, `crate::font::rustybuzz_wrapper::Buffer`
- All safe wrappers include `#[inline]` for hot-path optimization and `Drop` implementations for RAII resource cleanup
- Text iteration uses `Iterator` trait: `TextPage::blocks()` → `TextBlockIter`, `TextBlock::lines()` → `TextLineIter`, `TextLine::chars()` → `TextCharIter`
- Use `log_error!`, `log_warn!`, `log_info!` macros from `crate::helpers` instead of raw `eprintln!`

### Library Directory Convention

Plato uses a target-to-library directory convention to separate native shared libraries by architecture:

- **`libs/`** → ARM 32-bit (`arm-unknown-linux-gnueabihf`) for original Kobo devices
- **`libs64/`** → ARM 64-bit (`aarch64-unknown-linux-gnu`) for newer Kobo devices (Libra 2, Sage, Clara 2E, Elipsa 2E, etc.)
- **`libs_host/`** → Host/x86_64 (`x86_64-unknown-linux-gnu`) for development

The `get_lib_dir()` function in `build.sh` is the canonical source of truth for this mapping and is used by build and packaging scripts to resolve the correct library directory for each target.

### iOS Library Separation

**Mandatory rule:** Keep iOS device and simulator libraries separate to avoid lipo architecture conflicts.

- **Device libraries:** `target/{lib}/iOS-device/lib/` for ARM64 iOS devices
- **Simulator libraries:** `target/{lib}/iOS-simulator/lib/` for universal simulator libraries (ARM64 + x86_64)
- **Never combine device and simulator ARM64** in the same fat binary using `lipo` - they have the same architecture
- Use `lipo` only to combine different simulator architectures (ARM64 + x86_64) for universal simulator libraries
- Device libraries should be copied directly without lipo combination
- This pattern applies to all iOS native libraries: mupdf, mupdf_wrapper, and third-party dependencies

## Stub and Hardware Limitation Documentation

**Mandatory rule:** Document all stub implementations and unsupported features.

- When a trait method cannot be implemented (due to hardware/API limitations), add a default implementation with documentation
- Document why the feature is unsupported (e.g., "Not supported on Kobo e-readers", "MuPDF API limitation")
- Keep stub implementations in the trait definition (not in implementing structs) to avoid code duplication
- Example:

  ```rust
  /// Enables monochrome (grayscale) display mode.
  /// Not supported on Kobo e-readers.
  fn set_monochrome(&mut self, _enable: bool) {}

  /// Sets the font family for text rendering.
  /// Not supported by PDF documents (MuPDF API limitation).
  fn set_font_family(&mut self, _family_name: &str, _search_path: &str) {}
  ```

## Performance Optimization Decisions

This section documents key performance decisions for the Plato codebase, particularly for constrained Kobo devices.

### Memory Optimization

- **Pre-allocation**: Use `String::with_capacity` and `Vec::with_capacity` when size is predictable
- **Shared ownership**: Use `Rc` for shared MuPDF contexts, `Arc` for document references
- **Cow\<str\>**: Use for conditional string ownership to avoid unnecessary clones

### Stack Overflow Prevention

- **Large arrays on heap**: Move large temporary arrays to the heap using `Box<[u8; N]>` or `Vec<u8>`
- **Apply to**: Image buffers, glyph outlines, and processing pipelines
- **Memory Management**: Implement proper `Drop` traits for heap-allocated data
- **Error Handling**: Return `anyhow::Error` for allocation failures with context

### Battery Optimization

#### Event-Driven I/O

- Replace polling loops with `poll()`/`epoll`-style waiting for input events
- Use existing input event system; ensure no busy-wait in UI loops
- **Input Validation**: Validate all input events before processing
- **Error Handling**: Use `anyhow::Result` for I/O operations with context

#### State Caching

- Cache battery level, frontlight settings, and device orientation
- Implement cache invalidation on known change events
- **Configuration**: Define cache TTL and invalidation strategies in config
- **Validation**: Validate cached values before use

#### E-Ink Update Modes

- Use `UpdateMode::Partial` for small UI changes
- Use `UpdateMode::Gui` for glyphs and text rendering
- Reserve `UpdateMode::Full` for full-screen refreshes only
- **Dirty Region Tracking**: Implement efficient dirty region calculation
- **Validation**: Validate update mode parameters and constraints

### Not Recommended Optimizations

The following were investigated and deemed unnecessary for this codebase:

- **Thread pools for thumbnails**: Background fetchers already handle this; thread pools add complexity
- **Async file I/O**: E-ink refresh latency dominates; async adds overhead without perceptible benefit
- **Feature flags for plugins/sync**: These are integral features; feature flags add maintenance overhead

## Known Build Issues and Solutions

This section documents common build problems and their fixes.

### Host (x86_64) Build Fails with "incompatible with elf64-x86-64"

**Problem:** When building for host (`x86_64-unknown-linux-gnu`), linking fails with errors like:

```text
rust-lld: error: libs_host/libopenjp2.so is incompatible with elf64-x86-64
rust-lld: error: libs_host/libjbig2dec.so is incompatible with elf64-x86-64
```

**Root Cause:** The `libs_host/` directory in this repository incorrectly contains ARM libraries instead of x86_64 libraries. This is a historical artifact from the project's setup.

**Solution:** The build script has been updated in `crates/core/build.rs` to use system library paths (`/lib/x86_64-linux-gnu`) when building for x86_64 Linux. You should not need to modify `libs_host/`.

### mupdf_wrapper Not Found

**Problem:** Build fails with:

```text
error: could not find native static library `mupdf_wrapper`, perhaps an -L flag is missing?
```

**Solution:** Build the mupdf_wrapper library before building the project:

```bash
cd mupdf_wrapper
TARGET_OS=Linux ./build.sh  # for host
TARGET_OS=Kobo CC=arm-linux-gnueabihf-gcc AR=arm-linux-gnueabihf-ar ./build.sh  # for ARM
```

### Tests Fail to Compile - Missing tempfile

**Problem:** Test compilation fails with:

```text
error: unresolved import `tempfile::NamedTempFile`
```

**Solution:** Ensure `tempfile` is in `[dev-dependencies]` in `Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3.15"
```

### Tests Fail - Color::BLACK Not Found

**Problem:** Test fails with:

```text
error: no variant or associated item named `BLACK` found for enum `color::Color`
```

**Root Cause:** `BLACK` and `WHITE` are defined as constants (`pub const BLACK: Color = GRAY00;`), not enum variants. Code using `Color::BLACK` (with the `Color::` prefix) will fail.

**Solution:** Import and use the constant directly:

```rust
use crate::color::BLACK;  // not Color::BLACK
fb.set_pixel(50, 50, BLACK);
```

### Settings Tests Fail - Wrong Field Path

**Problem:** Test fails with:

```text
error: no field `font_size` on type `settings::Settings`
```

**Root Cause:** Settings fields are nested (e.g., `settings.reader.font_size`), not at the top level.

**Solution:** Use the correct nested path in tests:

```rust
assert_eq!(loaded.reader.font_size, settings.reader.font_size);
```

---

## Recent Architecture Improvements

### Device Trait Extraction (2026-04-20)

The `Device` trait has been extracted to enable hardware independence and testability:

- **KoboDevice**: Concrete implementation for actual Kobo hardware
- **MockDevice**: Mock implementation in `test_mocks.rs` for testing without hardware
- **Device trait**: Abstracts device-specific functionality (model, dimensions, DPI, frontlight capabilities, etc.)

**Benefits:**

- Enables unit testing without requiring actual Kobo hardware
- Allows future support for other device types through the trait interface
- Separates hardware-specific logic from UI/application code

**Usage:**

```rust
use plato_core::Device;

// In production
let device = KoboDevice::new(&product, &model_number);

// In tests
let mock = MockDevice::new(Model::Forma);
```

### Battery Error Type (2026-04-20)

The `Battery` trait now uses a custom `BatteryError` type via `thiserror` instead of `anyhow::Error`:

- **BatteryError**: Library-level error type with descriptive variants
  - `CapacityReadError`: Failed to read battery capacity
  - `StatusReadError`: Failed to read battery status
  - `IoError`: Wrapped I/O errors with automatic conversion

**Benefits:**

- Follows AGENTS.md guidance: use `thiserror` for library error types, `anyhow` for binaries
- Provides type-safe error handling for battery operations
- Clearer error messages for debugging

**Usage:**

```rust
use plato_core::battery::{Battery, BatteryError, KoboBattery};

let mut battery = KoboBattery::new()?;
match battery.capacity() {
    Ok(capacities) => { /* ... */ }
    Err(BatteryError::CapacityReadError(msg)) => { /* ... */ }
    Err(e) => { /* ... */ }
}
```

## Communication

- **Ask questions** — if you have any questions or doubts, don't hesitate to ask. It's better to clarify requirements than to make incorrect assumptions
- **Seek input** — when facing architectural decisions or unclear requirements, ask for direction before proceeding
- **Report contradictions** — if you find any contradiction in the codebase or documentation, don't hesitate to ask for clarification
