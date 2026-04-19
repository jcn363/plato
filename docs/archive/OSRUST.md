# Plato Open-Source Rust Conversion Plan

## Overview

This document outlines the comprehensive plan to convert Plato into a fully open-source Rust application, removing all proprietary dependencies and making it completely independent for deployment on Kobo e-readers and other platforms.

## Current Architecture

### Existing Crates

```
plato/
├── crates/
│   ├── core/         # Core document handling, rendering, UI (main library)
│   ├── plato/        # Main binary for Kobo devices
│   ├── emulator/     # SDL2 desktop emulator
│   ├── importer/    # Document importer
│   ├── fetcher/     # Article fetcher
│   ├── epub_edit/   # EPUB editing library
│   └── epub_editor/ # EPUB editor UI
```

### Current Dependencies

The project already uses primarily Rust dependencies:

- **Font rendering**: `skrifa`, `rustybuzz`, `ab_glyph` (pure Rust) ✅
- **PDF rendering**: `mupdf` (C library via FFI) ⚠️
- **Image handling**: `image`, `png`, `jpeg` ✅  
- **Serialization**: `serde`, `serde_json`, `toml` ✅
- **HTTP**: `reqwest` (rustls) ✅
- **Document formats**: `zip`, `flate2`, `epub_edit` ✅
- **Text processing**: `regex`, `unicode-normalization` ✅

## Remaining Proprietary Components

### 1. MuPDF (PDF Rendering)

**Current State**: Plato uses MuPDF C library via FFI for PDF rendering.

**Impact**: The only major C dependency remaining.

**Open-Source Alternatives**:

| Library | Pros | Cons |
|---------|------|------|
| `lopdf` | Pure Rust, no native deps | Limited feature set |
| `pdfium` | Feature-complete | Google-native, complex binding |
| `printpdf` | Pure Rust | Basic rendering only |

**Recommendation**: Keep MuPDF wrapper (`mupdf_wrapper/`) as it's performance-critical for e-ink devices, but:
- Make wrapper fully open-source MIT licensed
- Document build process clearly
- Consider alternative for desktop/emulator builds

### 2. Build Scripts

**Current State**: Build scripts tie to specific library paths.

**Actions**:
- Document required native dependencies
- Create Docker container for reproducible builds
- Add clear BUILD.md documentation

### 3. Device-Specific Code

**Current State**: Hardware interaction via FFI (`nix` crate).

**Status**: Already open-source compatible.

## Implementation Plan

### Phase 1: Documentation

- [ ] Create BUILD.md with complete build instructions
- [ ] Create DOCKER.md with containerized build environment
- [ ] Document all native dependencies
- [ ] Create CONTRIBUTING.md (update existing)

### Phase 2: Licensing & Cleanup

- [ ] Audit all dependencies for license compatibility
- [ ] Replace any GPL dependencies if needed
- [ ] Add LICENSE file (MIT as currently)
- [ ] Document all third-party code used

### Phase 3: Build System

- [ ] Simplify build.sh scripts
- [ ] Add default configurations for common targets
- [ ] Create reproducible build with Docker
- [ ] Add CI/CD for GitHub Actions

### Phase 4: Testing

- [ ] Add unit tests for core functionality
- [ ] Add integration tests for document handling
- [ ] Set up automated testing on ARM emulator
- [ ] Document testing approach

## Open-Source Dependencies

### Core Dependencies (All MIT/Apache Compatible)

```toml
# Document Handling
zip = "0.7"           # ZIP archive handling (Apache-2.0)
flate2 = "1.0"         # Gzip (Apache-2.0)
epub_edit = "0.9"       # EPUB editing (MIT)

# Text Rendering  
skrifa = "0.2"         # Font parsing (MIT)
rustybuzz = "0.13"      # Text shaping (MIT Apache-2.0)
ab_glyph = "0.2"       # Glyph rasterization (MIT)

# Image Handling  
image = "0.25"         # Image processing (MIT/Apache-2.0)

# Serialization
serde = "1.0"         # Serialization (MIT/Apache-2.0)
toml = "0.8"           # TOML config (MIT/Apache-2.0)

# Networking
reqwest = "0.12"       # HTTP client (MIT/Apache-2.0)

# Utilities
regex = "1.12"         # Regex (MIT/Apache-2.0)
chrono = "0.4"         # Date/time (MIT/Apache-2.0)
```

## Build Environments

### Kobo Devices

```bash
# 32-bit ARM (Original Kobo devices)
cargo build --target arm-unknown-linux-gnueabihf -p plato

# 64-bit ARM (Libra 2, Sage, Clara 2E, etc.)
cargo build --target aarch64-unknown-linux-gnu -p plato
```

### Desktop/Emulator

```bash
# Linux/macOS
cargo build -p plato-emulator

# Windows (via WSL)
cargo build --target x86_64-pc-windows-gnu
```

## Docker Build Environment

Create `Dockerfile` for reproducible builds:

```dockerfile
FROM rust:1.75-bookworm

# Install ARM cross-compilation
RUN apt-get update && apt-get install -y \
    gcc-arm-linux-gnueabihf \
    g++-arm-linux-gnueabihf \
    libc6-dev-armhf-cross \
    pkg-config-arm-linux-gnueabihf \
    libssl-dev-armhf-cross

# Install native dependencies
RUN apt-get install -y \
    libsdl2-dev \
    libssl-dev \
    zlib1g-dev \
    libjpeg-dev \
    libpng-dev

WORKDIR /plato
COPY . .
RUN cargo build --target arm-unknown-linux-gnueabihf -p plato
```

## Release Process

1. Version bump in `Cargo.toml`
2. Build for all targets
3. Create distribution bundles
4. Generate checksums
5. Create GitHub release

## Conclusion

Plato is already ~95% open-source Rust. The primary remaining component is the MuPDF C wrapper which is performance-critical for e-ink devices. With proper documentation and build system improvements, it can be fully self-hosted.

### Key Deliverables

1. **BUILD.md** - Complete build instructions
2. **Dockerfile** - Reproducible builds
3. **CI/CD** - Automated GitHub Actions
4. **Tests** - Full test coverage

### Timeline

- Phase 1 (Documentation): 1 week
- Phase 2 (Licensing): 1 week  
- Phase 3 (Build System): 2 weeks
- Phase 4 (Testing): 2 weeks

**Total: ~6 weeks for full open-source conversion**