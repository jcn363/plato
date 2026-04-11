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

The repository also contains a standalone `epub_editor/` CLI tool that is built by `build.sh`, but it is excluded from the Cargo workspace.

Documentation:

- [Installation and configuration guide](doc/GUIDE.md)
- [User manual](doc/MANUAL.md)
- [Build instructions](doc/BUILD.md)
- [Not implemented features](doc/NOT_IMPLEMENTED.md)
- [OCR and TTS notes](doc/OCR_TTS.md)

## Supported firmwares

Any 4.*X*.*Y* firmware, with *X* ≥ 6, will do.

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
- **PDF** - Added auto-crop margins feature for scanned documents, PDF/A detection, annotation reading and export
- **Rendering** - Added minimum font size support for better readability
- **ePUB** - Enhanced HTML engine with improved font handling
- **CSS** - Full CSS support including border, background, text-transform, text-decoration, tab-size
- **ARM** - Added NEON SIMD and VFP4 optimizations for 32-bit Kobo devices
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

# Create distribution bundle
./dist.sh

# Run the desktop emulator (requires SDL2)
./run-emulator.sh

# Install the importer helper
./install-importer.sh

# Run tests (requires host target)
cargo test --target x86_64-unknown-linux-gnu
```

## Performance Optimizations

Recent performance improvements include:

- Added NEON SIMD and VFP4 optimizations for 32-bit Kobo devices
- Inlined hot-path functions for pixel operations, geometry calculations, and device capabilities
- Migrated to `std::sync::LazyLock` for better performance with constants and regex patterns
- Optimized memory usage with reduced MuPDF context cache and grayscale thumbnails
- Added progressive document loading with LRU caching for large PDFs

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
