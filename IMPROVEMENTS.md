# Plato Codebase Improvement Opportunities

## Current Status

| Metric | Status |
|--------|--------|
| **Build** | ✅ Clean (x86_64, ARM32, ARM64) |
| **Clippy** | ✅ Clean (no warnings) |
| **Unwrap/expect** | ⚠️ ~30 in production code (Regex builds, FFI, tests) |
| **Tests** | ⚠️ Require native libs (mupdf, gumbo) |
| **License** | ✅ MIT on all crates, deny.toml configured |
| **Dependencies** | ✅ Mostly current (see [Dependency Management](#dependency-management)) |

## Completed Improvements

### Documentation
- ✅ Module-level docs: `reader_impl/`, `document/html/`, `font/`, `gesture.rs`
- ✅ Function-level docs: `context.rs` (# Errors/# Panics), `pdf.rs`, `geom/*.rs` (examples)
- ✅ Section comments: `reader.rs` (9 sections), `font/mod.rs` (8 sections)

### Code Quality
- ✅ Format string improvements (build.rs, epub_edit)
- ✅ Raw string literal cleanup (epub_edit)
- ✅ `lazy_static` → `LazyLock` migration (13 instances across 9 files)
- ✅ Result documentation (# Errors sections)
- ✅ Option usage simplification (map().unwrap_or() → direct patterns)
- ✅ `#[must_use]` attributes on geometry/color methods
- ✅ DeviceFlags bitflag for Context struct booleans
- ✅ Pre-allocation (`with_capacity()`) across 20+ locations

### Dependencies
- ✅ `nix` 0.30.1 → 0.31.2
- ✅ `zip` 7.0.0 → 8.5.0
- ✅ `rand_core` 0.9.x → 0.10, `rand_xoshiro` 0.7.0 → 0.8.0
- ✅ `quick-xml` 0.37.0 → 0.39.2
- ✅ `indexmap` 2.13.0 → 2.13.1
- ✅ `chrono` 0.4.42 → 0.4.44
- ✅ `fxhash` → `rustc-hash` 2.1.2 (RUSTSEC-2025-0057 resolved)
- ✅ MIT license added to all 6 crates
- ✅ `deny.toml` for cargo-deny
- ✅ Workspace dependency alignment (`[workspace.dependencies]`)

### Architecture
- ✅ Safe FFI wrappers: `mupdf.rs`, `freetype.rs`, `harfbuzz.rs` with RAII/Drop
- ✅ `pdf.rs` and `pdf_manipulator.rs` migrated to safe wrappers
- ✅ AArch64 (ARM64) support for newer Kobo devices
- ✅ Build system: `mupdf_wrapper.c` expanded with 20+ custom FFI functions
- ✅ Build fix: `context.online` → `flags.remove(DeviceFlags::ONLINE)` in emulator

## Remaining Items

### Cannot Fix — Device-Dependent `lazy_static` (5 remaining)
These depend on `CURRENT_DEVICE` runtime configuration and cannot use `LazyLock`:
- `device.rs:474` — `CURRENT_DEVICE` (env vars)
- `frontlight/natural.rs:38` — `FRONTLIGHT_DIRS` (device model)
- `font/mod.rs:88` — `MD_TITLE` (device dims/DPI)
- `font/md_title.rs:5` — `MD_TITLE` (device dims/DPI)
- `view/icon.rs:18` — `ICONS_PIXMAPS` (device DPI)

### Deferred — Not Worth the Complexity
- **Object pooling** — Render chunks cached via LRU; geometry objects are Copy/Clone stack types; E-ink refresh latency dominates
- **Gesture algorithm optimization** — Already O(1) throughout
- **Text layout optimization** — Uses Knuth-Plass (TeX standard) via `paragraph-breaker`
- **Image scaling optimization** — MuPDF bilinear + Lanczos3 already appropriate
- **FreeType/HarfBuzz separation** — Already cleanly separated at FFI and safe-wrapper levels
- **Bitmap font modules** — No bitmap font usage exists; all outline fonts

### Future Opportunities

#### Test Coverage
- Property-based testing (proptest/quickcheck) for geometry, PDF layout, font metrics, gestures
- Integration tests for document loading, UI transitions, input handling, settings persistence
- *Blocker:* Tests require native libs (mupdf, gumbo) not available on host

#### File Splitting (evaluated, not feasible)
- `reader.rs` (4168 LOC) — tightly coupled state machine; extraction not practical
- `font/mod.rs` (2783 LOC) — FreetypeError coupling prevents clean extraction
- `html/engine.rs` (2678 LOC) — rendering pipeline requires full context

#### Dependency Updates (deferred)
- `reqwest` 0.12.28 → 0.13.x — Breaking TLS feature changes, requires ARM testing
- `toml` 0.9.x → 1.x — Major API changes (crate renamed to `toml-edit` for editing)
- `bincode` 1.3.3 — RUSTSEC-2025-0141 via kl-hyphenate; low risk (not network-facing)

#### Potential Features
- Dark mode transitions (smooth fade vs instant)
- Reading statistics export
- Cloud bookmark sync
- Custom gesture configuration UI
- Annotation export to Markdown
- Stylus notes/sketches (Elipsa)

## Dependency Management

### Security Advisories

| Package | Advisory | Status | Risk |
|---------|----------|--------|------|
| ~~`fxhash`~~ | RUSTSEC-2025-0057 | ✅ Replaced with `rustc-hash` | Resolved |
| `bincode` (via kl-hyphenate) | RUSTSEC-2025-0141 | ⚠️ Unmaintained | Low (offline use) |

### Version Alignment

| Package | plato-core | fetcher | Status |
|---------|-----------|---------|--------|
| `reqwest` | 0.12.28 | 0.13.2 | ⚠️ Unaligned (TLS breaking changes) |

## Device-Specific Optimizations

| Optimization | Status | Details |
|-------------|--------|---------|
| Display refresh batching | ✅ Implemented | `MAX_UPDATE_DELAY` (600ms) deduplicates updates |
| Font cache eviction | N/A | No explicit font cache; fonts loaded on demand |
| Filesystem sync | N/A | Standard `std::fs` without explicit `fsync()` |

## Codebase Overview

| Crate | Purpose | Files | LOC |
|-------|---------|-------|-----|
| plato-core | Core library | 188 | ~53k |
| plato | Kobo binary | — | — |
| emulator | SDL2 desktop | — | — |
| importer | Document import | — | — |
| fetcher | Article fetcher | — | — |
| epub_edit | EPUB editing | — | — |

**Total dependencies:** ~423 (including transitive)
