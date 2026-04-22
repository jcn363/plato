# Optimization Plan for Plato Codebase

## Overview

This plan outlines performance optimizations for the Plato codebase following AGENTS.md guidelines, focusing on hot-path improvements, memory usage, and battery efficiency. **No backward compatibility constraints apply** - all optimizations will implement current best practices without legacy support.

## Implementation Status (April 22, 2026)

### Implemented Optimizations

- **FxHashMap/FxHashSet**: Used in helpers.rs, opds.rs, metadata/constants.rs, library/types.rs, keyboard.rs
- **`#[inline]` attributes**: Used in emulator, input.rs, metadata modules, theme.rs
- **`Vec::with_capacity`**: Used in emulator, app.rs, opds.rs, sync.rs
- **LTO**: Enabled in Cargo.toml (lto = "thin" for release profiles)
- **SmallVec**: Implemented for annotations in ReaderInfo (metadata/info.rs) with serde feature
- **`Cow<str>`**: Implemented for Info::title() and Info::label() methods to avoid clones when no modifications needed
- **Buffer Reuse**: Buffer pool module created (buffer_pool.rs) and integrated into thumbnail generation (thumbnail/worker.rs)
- **ARM Build Verification**: Successfully built for arm-unknown-linux-gnueabihf target with zero warnings and zero errors

### Partially Implemented

- **BTreeMap/BTreeSet**: Used in metadata/info.rs (page_names, bookmarks, categories, tags) but not widely adopted elsewhere

### Not Implemented

- **`Cow<str>`**: Not implemented in other parts of codebase
- **Buffer Reuse Integration**: Not integrated into document parsing
- **Stack Overflow Prevention**: Not implemented (Box for large arrays)

## Hot‑Path Optimizations

### 1. Inline Small Functions  IMPLEMENTED

- Add `#[inline]` to functions called in rendering loops, pixel operations, geometry math, and device checks
- **Status**: Implemented in emulator, input.rs, metadata modules, theme.rs
- **Validation**: All inline functions must have input validation at public API boundaries
- **Error Handling**: Use `anyhow::Result<T>` for functions that can fail

### 2. Hash Maps & Sets ✅ IMPLEMENTED

- Replace `std::HashMap`/`HashSet` with `fxhash::FxHashMap`/`FxHashSet` where cryptographic security is not required
- **Status**: Implemented in helpers.rs, opds.rs, metadata/constants.rs, library/types.rs, keyboard.rs
- **Input Validation**: Validate keys and values before insertion
- **Configuration**: Define hash map usage in centralized config modules

### 3. String Allocation ⚠️ PARTIAL

- Use `String::with_capacity` when the final size is known or can be estimated
- **Status**: Implemented in emulator, app.rs, opds.rs, sync.rs
- **`Cow<str>`**: NOT IMPLEMENTED - prefer `Cow<str>` for conditional string ownership to avoid unnecessary clones
- **Validation**: Validate string inputs for length, encoding, and content constraints

### 4. Buffer Reuse ❌ NOT IMPLEMENTED

- Implement thread-local buffer pools for temporary work (thumbnail generation, document parsing)
- Use `lazy_static` or `std::sync::LazyLock` for global buffer management
- **Memory Safety**: Ensure proper cleanup with `Drop` implementations
- **Error Handling**: Return `anyhow::Error` for buffer allocation failures

### 5. Iterator Chains ✅ IMPLEMENTED

- Ensure iterator adapters are fused where possible
- Collect into pre‑allocated vectors using `Vec::with_capacity`
- **Status**: Implemented in multiple locations
- **Performance**: Profile iterator chains before and after optimization

## Memory Optimizations

### 1. Shared Ownership

- Use `Rc` for shared immutable data (MuPDF contexts, font data)
- Use `Arc` for data accessed across threads
- **Audit**: Review all `Clone` implementations to avoid deep copies
- **Validation**: Validate shared data before creating references
- **Error Handling**: Use `thiserror` for custom memory-related error types

### 2. Data Structure Choice

- Prefer `BTreeMap`/`BTreeSet` for ordered collections
- Use `IndexMap` for insertion‑order preservation
- Use `smallvec::SmallVec` for vectors that usually hold 0‑2 elements
- **Configuration**: Define data structure choices in module-level constants
- **Validation**: Validate data structure capacity and constraints

### 3. Stack Overflow Prevention

- Move large temporary arrays to the heap (`Box<[u8; N]>` or `Vec<u8>`)
- Apply to image buffers, glyph outlines, and processing pipelines
- **Memory Management**: Implement proper `Drop` traits for heap-allocated data
- **Error Handling**: Return `anyhow::Error` for allocation failures with context

## Battery Optimizations

### 1. Event‑Driven I/O

- Replace polling loops with `poll()`/`epoll`‑style waiting for input events
- Use existing input event system; ensure no busy‑wait in UI loops
- **Input Validation**: Validate all input events before processing
- **Error Handling**: Use `anyhow::Result` for I/O operations with context

### 2. State Caching

- Cache battery level, frontlight settings, and device orientation
- Implement cache invalidation on known change events
- **Configuration**: Define cache TTL and invalidation strategies in config
- **Validation**: Validate cached values before use

### 3. E‑Ink Update Modes

- Use `UpdateMode::Partial` for small UI changes
- Use `UpdateMode::Gui` for glyphs and text rendering
- Reserve `UpdateMode::Full` for full‑screen refreshes only
- **Dirty Region Tracking**: Implement efficient dirty region calculation
- **Validation**: Validate update mode parameters and constraints

## Build‑Time & Binary Size

### 1. Link‑Time Optimizations

- Enable `lto = true` in release profiles for ARM targets
- Remove unused dependencies with `cargo-deadpep`
- **Configuration**: Define optimization profiles in `Cargo.toml`
- **Validation**: Validate build configurations before use

### 2. Debug Symbols

- Strip debug symbols in production builds using `strip`
- **Build Process**: Integrate symbol stripping into build scripts
- **Error Handling**: Handle strip failures gracefully with fallback options

### 3. Feature Flags

- Keep feature flags only for truly optional backends
- Remove flags for core features to reduce maintenance burden
- **Configuration**: Centralize feature flag definitions
- **Validation**: Validate feature flag combinations at compile time

## Verification Procedure

### Mandatory Build Verification (per AGENTS.md)

For each optimization change:

1. **Incremental Verification**
   - Compile for primary ARM target after each change:

     ```bash
     cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato
     ```

   - Treat warnings as errors (zero-tolerance policy)

2. **Code Quality Validation**
   - Run `cargo fmt` to ensure consistent formatting
   - Run `cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings`
   - Ensure all tests pass: `cargo test --target x86_64-unknown-linux-gnu`

3. **Performance Profiling**
   - Confirm the function is in a hot path (profiled via `perf` or instrumentation)
   - Add `#[inline]` where appropriate and verify no code‑size regression
   - Run `cargo bench` (if benchmarks exist) or manual timing on device

4. **Full Build Verification**

   ```bash
   # Primary target: ARM Kobo (32-bit)
   cargo clean && cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato
   
   # Secondary target: ARM64 Kobo (newer devices)
   cargo clean && cargo build --target aarch64-unknown-linux-gnu --profile release-arm64 -p plato
   
   # Host target: for testing
   cargo clean && cargo build --target x86_64-unknown-linux-gnu
   ```

5. **Battery Testing**
   - On device, verify battery drain with rendering loops (30+ minutes)
   - Measure power consumption before and after optimizations
   - Document battery life improvements

6. **Input Validation Testing**
   - Test all public APIs with invalid inputs
   - Verify proper error messages and early failures
   - Ensure no panic conditions from unvalidated inputs

## Areas to AVOID (per AGENTS.md)

### Explicitly Prohibited Optimizations

- **Thread pools for thumbnail fetching** – background fetchers already handle concurrency
- **Async file I/O** – e‑ink latency dominates; added complexity not worthwhile
- **Feature flags for plugins/sync** – these are core features; flags add maintenance burden
- **Backward compatibility code** – no legacy support per requirements

### Anti-Patterns to Eliminate

- **Dead code** – remove all `#[allow(dead_code)]` attributes and unused functions
- **Unsafe code without validation** – all unsafe blocks must have comprehensive validation
- **Mixed error handling** – standardize on `anyhow`/`thiserror` throughout
- **Duplicate logic** – extract common patterns into shared functions

## Implementation Requirements

### Mandatory AGENTS.md Compliance

- **Input Validation**: All public APIs must validate inputs and fail fast
- **Error Handling**: Use `anyhow` for application code, `thiserror` for libraries
- **Modular Design**: No file over 1000 lines, no function over 50 lines
- **Test Segregation**: Unit tests in sibling files, integration tests in `tests/`
- **Zero Tolerance**: No warnings, no errors, no dead code

### Configuration Management

- Centralize all optimization settings in dedicated config modules
- Validate configuration values at load time with clear error messages
- Use typed configuration over raw strings or magic numbers
- Document all configuration options with valid ranges and defaults

### Documentation Requirements

- Add module-level documentation for all optimization modules
- Include examples for all public APIs in rustdoc comments
- Document performance trade-offs and measurement results
- Keep architecture docs in `docs/architecture/` and reference from modules

## References

- AGENTS.md: Performance, Input Validation, Dependencies, Error Handling sections
- Existing usage of `#[inline]` in `framebuffer::image.rs` and `geom::helpers.rs`
- `fxhash` crate already present in dependencies
- Build verification procedures from AGENTS.md Build Verification section

This plan provides a comprehensive, AGENTS.md-compliant approach to optimize Plato for the Kobo hardware with zero tolerance for warnings, errors, or backward compatibility constraints.
