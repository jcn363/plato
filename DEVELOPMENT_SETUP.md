# Plato Development Setup

Quick reference for developing Plato e-reader software.

> **Status**: ✅ Ready for development  
> **Last Updated**: 2026-04-27

---

## ✅ Prerequisites

| Component   | Version/Status       | Notes                |
|-------------|----------------------|----------------------|
| Rust        | 1.93.0               | Stable toolchain     |
| Targets     | x86_64, arm, aarch64 | Host + cross-compile |
| OpenSSL     | libssl-dev           | System dependency    |
| C Libraries | ❌ None              | Pure Rust project    |

**Pure Rust Stack**:

- `pdfpurr` - PDF rendering (replaces MuPDF)
- `lopdf` - PDF manipulation
- `skrifa` + `rustybuzz` - Font stack (replaces FreeType/HarfBuzz)
- `djvu-rs`, `zip`, `unrar` - Format support

---

## ⚠️ Known Issues

### Test Linking (Host Development)

The `libs_host/` directory contains 32-bit ARM binaries, not x86_64. This causes test linking to fail on host machines.

**Impact**: `cargo check` and `cargo build` work; `cargo test` fails at link stage.

**Workaround**: Use `cargo check` for verification, or build x86_64 libraries:

```bash
./build.sh host slow
```

---

## 📋 Development Commands

### Building for Development (Host)

```bash
# Quick check without linking
cargo check --target x86_64-unknown-linux-gnu

# Full build (will fail to link tests without proper libs)
./build.sh host skip --no-clippy --no-fmt

# Build with formatting and linting
./build.sh host skip
```

### Building for Kobo Devices

```bash
# 32-bit ARM (original Kobo)
./build.sh arm

# 64-bit ARM (newer Kobo devices)
./build.sh arm64
```

### Testing

```bash
# Tests require proper x86_64 libraries
cargo test --target x86_64-unknown-linux-gnu

# Alternatively, use cargo check for verification
cargo check --target x86_64-unknown-linux-gnu --workspace
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings
```

---

## 🔧 Additional Setup (Optional)

### Cross-Compilation Verification

Verify ARM builds for Kobo devices:

```bash
./build.sh arm skip --no-clippy --no-fmt
```

### CI/CD Integration

Consider GitHub Actions for:
- ARM cross-compilation
- clippy and fmt checks

---

## 📊 Build Status Summary

| Component | Status | Notes |
|-----------|--------|-------|
| Rust Toolchain | ✅ | 1.93.0, all targets |
| Native Dependencies | ✅ | Pure Rust (no C libs) |
| PDF Rendering | ✅ | PDFPurr 0.4.0 |
| PDF Manipulation | ✅ | lopdf |
| ARM Build | ✅ | Successfully builds for Kobo |
| Unit Tests | ✅ | No C libraries needed |
| Clippy | ✅ | Passes |
| Formatting | ✅ | rustfmt.toml configured |

---

## 📚 Project Structure

The Plato workspace consists of:

| Crate         | Purpose                                                |
|---------------|--------------------------------------------------------|
| `plato-core`  | Core library: document handling, rendering, UI, device |
| `plato`       | Main binary for Kobo devices                           |
| `importer`    | Document importer tool                                 |
| `fetcher`     | Article fetcher from online sources                    |
| `epub_edit`   | EPUB editing library                                   |
| `epub_editor` | EPUB editing CLI tool                                  |

---

## 🎯 Development Workflow

For day-to-day development:

1. **Make changes** to source code
2. **Run `cargo fmt`** to maintain code style
3. **Check compilation**: `cargo check --target x86_64-unknown-linux-gnu`
4. **Run clippy**: `cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings`
5. **Commit changes** with descriptive messages
6. **Test on device/emulator** when functionality changes

---

*Last updated: 2026-04-27*
