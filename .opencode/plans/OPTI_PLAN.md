# Optimization Plan for Plato Codebase

## Overview
This plan outlines performance optimizations for the Plato codebase following AGENTS.md guidelines, focusing on hot-path improvements, memory usage, and battery efficiency without breaking backward compatibility.

## Hot‑Path Optimizations
1. **Inline Small Functions**
   - Add `#[inline]` to functions called in rendering loops, pixel operations, geometry math, and device checks.
   - Target: `framebuffer::Pixmap::get_pixel/set_pixel`, `geom::lerp`, `surface_area`, `nearest_segment_point`.
2. **Hash Maps & Sets**
   - Replace `std::HashMap`/`HashSet` with `fxhash::FxHashMap`/`FxHashSet` where cryptographic security is not required (caches, look‑ups).
3. **String Allocation**
   - Use `String::with_capacity` when the final size is known or can be estimated (e.g., building file paths, JSON strings).
   - Prefer `Cow<str>` for conditional ownership to avoid unnecessary clones.
4. **Buffer Reuse**
   - Reuse allocation buffers for temporary work (e.g., thumbnail generation, document parsing) via thread‑local pools or `lazy_static` buffers.
5. **Iterator Chains**
   - Ensure iterator adapters are fused where possible; collect into pre‑allocated vectors.

## Memory Optimizations
1. **Shared Ownership**
   - Use `Rc` for shared immutable data (MuPDF contexts, font data) and `Arc` for data accessed across threads.
   - Audit `Clone` implementations to avoid deep copies.
2. **Data Structure Choice**
   - Prefer `BTreeMap`/`BTreeSet` for ordered collections; `IndexMap` for insertion‑order preservation.
   - Use `smallvec::SmallVec` for vectors that usually hold 0‑2 elements to avoid heap allocation.
3. **Avoid Stack Overflow**
   - Move large temporary arrays to the heap (`Box<[u8; N]>` or `Vec<u8>`) in hot paths (image buffers, glyph outlines).

## Battery Optimizations
1. **Event‑Driven I/O**
   - Replace polling loops with `poll()`/`epoll`‑style waiting for input vsensors.
   - Use existing input event system; ensure no busy‑wait in UI loops.
2. **State Caching**
   - Cache battery level, frontlight settings, and device orientation to reduce redundant sysfs reads.
   - Invalidate caches on known change events.
3. **E‑Ink Update Modes**
   - Use `UpdateMode::Partial` for small UI changes, `UpdateMode::Gui` for glyphs, and reserve `UpdateMode::Full` for full‑screen refreshes.
   - Avoid unnecessary full flashes by tracking dirty regions.

## Build‑Time & Binary Size
1. **Link‑Time Optimizations**
   - Enable `lto = true` in release profiles for ARM targets.
   - Remove unused dependencies with `cargo-deadpep`.
2. **Debug Symbols**
   - Strip debug symbols in production builds (`strip` on the final binary).
3. **Feature Flags**
   - Keep feature flags only for truly optional backends (e.g., alternative document parsers) to avoid compiling dead code.

## Verification Procedure
For each change:
1. Confirm the function is in a hot path (profiled via `perf` or instrumentation on device).
2. Add `#[inline]` where appropriate and verify no code‑size regression.
3. Run `cargo bench` (if benchmarks exist) or manual timing on device.
4. Ensure `cargo check --target x86_64-unknown-linux-gnu` and `cargo clippy -- -D warnings` pass.
5. Run the test suite on host target to catch regressions.
6. On device, verify battery drain with a simple script (e.g., loop rendering a page for 30 min and measure %).

## Areas to AVOID (per AGENTS.md)
- Thread pools for thumbnail fetching – background fetchers already handle concurrency.
- Async file I/O – e‑ink latency dominates; added complexity not worthwhile.
- Feature flags for plugins/sync – these are core features; flags add maintenance burden.

## References
- AGENTS.md: Performance, Input Validation, Dependencies sections.
- Existing usage of `#[inline]` in `framebuffer::image.rs` and `geom::helpers.rs`.
- `fxhash` crate already present in dependencies.

This plan provides a targeted, measurable approach to optimize Plato for the Kobo hardware while respecting the project’s constraints.