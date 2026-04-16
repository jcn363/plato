# Plato

![Logo](artworks/plato-logo.svg)

This is an optimized version of the original [Plato](https://github.com/pettarin/plato) document reader for Kobo e-readers.

*Plato* is a document reader for *Kobo*'s e-readers.

The current source tree is a Cargo workspace with these crates:

- `crates/core` (`plato-core`) for document handling, rendering, UI, device support, sync, and settings
- `crates/plato` for the Kobo device binary
- `crates/emulator` for the desktop SDL2 emulator
- `crates/importer` for the `plato-import` tool
- `crates/fetcher` for the `article_fetcher` binary
- `crates/epub_edit` for EPUB editing support used by the in-app editor
- `crates/epub_editor` for the `epub_editor` CLI tool

Documentation:

- [Installation and configuration guide](doc/GUIDE.md)
- [User manual](doc/MANUAL.md)
- [Build instructions](doc/BUILD.md)
- [Not implemented features](doc/NOT_IMPLEMENTED.md)
- [OCR and TTS notes](doc/OCR_TTS.md)

## Supported firmwares

Any 4.*X*.*Y* firmware, with *X* >= 6, will do.

## Build Status

**Current Status**: Build compiles successfully with only warnings

- **Target**: `x86_64-unknown-linux-gnu` (development)
- **Target**: `arm-unknown-linux-gnueabihf` (32-bit ARM Kobo devices)
- **Target**: `aarch64-unknown-linux-gnu` (64-bit ARM Kobo devices)

### Recent Verification Results

**Completed** (Final Verification Pass - April 2026):
- Fixed all critical compilation errors
- Resolved import issues and trait implementations
- Added missing View trait methods for Reader
- Fixed type mismatches and borrowing issues
- Audited dead code instances (all justified)
- Updated documentation with current status

**Critical Issues Identified**:
- 4 files exceed AGENTS.md 1,000 line limit (requires modularization)
  - `document/html/engine.rs`: 2,667 lines
  - `view/reader/reader_impl/reader.rs`: 2,370 lines
  - `document/html/engine_text.rs`: 1,073 lines
  - `view/home/ui_toggles.rs`: 1,014 lines

For detailed integration progress, see [INTEGRATION_PROGRESS.md](INTEGRATION_PROGRESS.md).

## Supported devices

- *Libra Colour*.
- *Clara Colour*.
- *Clara BW*.
- *Elipsa 2E*.
- *Clara 2E*.
- *Libra 2*.
- *Sage*.
- *Elipsa*.
- *Nia*.
- *Libra H₂O*.
- *Forma 32GB*.
- *Forma*.
- *Clara HD*.
- *Aura H₂O Edition 2*.
- *Aura Edition 2*.
- *Aura ONE Limited Edition*.
- *Aura ONE*.
- *Touch 2.0*.
- *Glo HD*.
- *Aura H₂O*.
- *Aura*.
- *Aura HD*.
- *Mini*.
- *Glo*.
- *Touch A/B*.
- *Touch C*.

## Supported formats

- ePUB through the built-in renderer.
- HTML and HTM through the built-in HTML renderer.
- PDF, CBZ, FB2, FBZ, MOBI, XPS, OXPS, and TXT via [MuPDF](https://mupdf.com/index.html).

## Features

- Built-in home screen, reader, dictionary, calculator, sketch, statistics, EPUB editor, cover editor, and PDF tools views.
- Configurable libraries, hooks, Wi-Fi scripts, dictionaries, CSS overrides, hyphenation bounds, and keyboard layouts.
- Reading features including annotations, highlights, bookmarks, search, table of contents, page naming, margin cropping, and fit-to-width reading.
- Theme and display controls including inversion, dark/theme modes, frontlight integration, rotation, and dithering controls.
- Library features including metadata extraction, thumbnail previews, batch delete/move, removable-storage import, and article fetching hooks.
- Sync and extension infrastructure including WebDAV sync, KoboCloud sync, shell/python plugin triggers, and plugin network permission checks.
- PDF-specific tooling including page delete/rotate/extract/reorder/merge operations, redaction, resource extraction, PDF/A inspection, and PDF annotation export.
- Progressive document loading support for large PDFs.

[![Tn01](artworks/thumbnail01.png)](artworks/screenshot01.png) [![Tn02](artworks/thumbnail02.png)](artworks/screenshot02.png) [![Tn03](artworks/thumbnail03.png)](artworks/screenshot03.png) [![Tn04](artworks/thumbnail04.png)](artworks/screenshot04.png)

## Optimizations

- **Build System** - Resolved linker failures by expanding `mupdf_wrapper.c` with 20+ custom FFI functions (PDF manipulation, annotations, redactions, image/font extraction); wrapper is now automatically linked via `build.rs`
- **Safe FFI Wrappers** - Added `mupdf.rs`, `freetype.rs`, `harfbuzz.rs` with RAII/Drop semantics for safe resource management; `pdf.rs` and `pdf_manipulator.rs` migrated to use safe wrappers
- **AArch64 (ARM64)** - Added support for newer Kobo devices (Libra 2, Sage, Clara 2E, Elipsa 2E, etc.)
- **Error Handling** - Improved robustness with proper error handling instead of `unwrap()`; further reduced unwrap/expect in sync, HTML parsing, and fetcher crates
- **Memory** - Optimized string building with pre-allocated buffers, fixed memory availability detection, reduced thumbnail memory by 75% (grayscale instead of RGBA), reduced MuPDF context cache from 32MB to 16MB, fixed Pixmap OOM panics, optimized pixmap creation to avoid double allocation
- PDF - Added auto-crop margins feature for scanned documents, PDF/A detection, annotation reading and export, interactive redaction region definition UI, and PDF merging functionality. **Note:** These features are implemented and stable.
- **Rendering** - Added minimum font size support for better readability
- **ePUB** - Enhanced HTML engine with improved font handling
- **CSS** - Full CSS support including border, background, text-transform, text-decoration, tab-size
- **Framebuffer** - Added `#[inline]` to all pixel operations for faster rendering
- **Geometry** - Added `#[inline]` to Point, Vec2, Rectangle methods for faster calculations
- **Document** - Added `#[inline]` to PDF page methods and font metrics
- **Device** - Added `#[inline]` to all device capability methods
- **Input** - Added `#[inline]` to button status conversion
- **Modern Rust** - Migrated 13 `lazy_static!` instances to `std::sync::LazyLock` for constants, regex patterns, translations, and dithering matrices

## Build Targets

```bash
# Build for 32-bit ARM (original Kobo devices) — DEFAULT
cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato

# Build for 64-bit ARM (newer Kobo devices: Libra 2, Sage, Clara 2E, Elipsa 2E, etc.)
cargo build --target aarch64-unknown-linux-gnu --profile release-arm64 -p plato

# Build for host (development/testing)
cargo build --target x86_64-unknown-linux-gnu -p plato

# Full build with native dependencies (downloads libs + MuPDF)
./build.sh

# Full build with options (e.g., skip clean, use specific target/method)
./build.sh --no-clean arm skip

# Create distribution bundle
./dist.sh [arm|arm64]

# Run the desktop emulator (requires SDL2)
./run-emulator.sh

# Install the importer helper
./install-importer.sh

# Run tests (requires host target)
cargo test --target x86_64-unknown-linux-gnu
```

## Library Directories

Plato uses three separate library directories to support different build targets:

- **`libs/`** → ARM 32-bit (`arm-unknown-linux-gnueabihf`) for original Kobo devices
- **`libs64/`** → ARM 64-bit (`aarch64-unknown-linux-gnu`) for newer Kobo devices (Libra 2, Sage, Clara 2E, Elipsa 2E, etc.)
- **`libs_host/`** → Host/x86_64 (`x86_64-unknown-linux-gnu`) for development and emulator

Each directory is populated by `build.sh` or `download.sh` with the appropriate native shared libraries for the target architecture. The `get_lib_dir()` function in `build.sh` is the canonical source of truth for this target-to-directory mapping.

## Performance Optimizations

Recent performance improvements follow the comprehensive [OPTI_PLAN.md](OPTI_PLAN.md) with AGENTS.md compliance:

- **Hot-Path Optimizations**: Added `#[inline]` to pixel operations, geometry calculations, and device capabilities
- **Memory Management**: Migrated to `std::sync::LazyLock`, optimized MuPDF context cache, grayscale thumbnails
- **Battery Efficiency**: Event-driven I/O, state caching, optimized e-ink update modes
- **Build Optimizations**: LTO enabled, debug symbols stripped, feature flags cleaned
- **Input Validation**: All public APIs validate inputs and fail fast with proper error handling
- **Error Handling**: Standardized on `anyhow`/`thiserror` throughout the codebase
- **Zero Tolerance**: No warnings, no errors, no dead code, no backward compatibility constraints

For detailed implementation procedures and verification steps, see [OPTI_PLAN.md](OPTI_PLAN.md).

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

## Credits

This project is based on the excellent work of the original Plato developer. See the [upstream project](https://github.com/pettarin/plato) for the original implementation.

## Donations

[![Donate](https://img.shields.io/badge/Donate-PayPal-green.svg)](https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=KNAR2VKYRYUV6)
