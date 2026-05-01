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
- `djvu-rs`, `zip`, `rar` - Format support (pure Rust)

---

## ⚠️ Known Issues

None currently. All targets compile and tests pass.

**Desktop Development**: The software framebuffer backend enables full development on Linux desktops without requiring a physical framebuffer device (/dev/fb0).

## 📋 Development Commands

### Building for Development (Host)

```bash
# Quick check without linking
cargo check --target x86_64-unknown-linux-gnu

# Full build for desktop development
cargo build --target x86_64-unknown-linux-gnu -p plato

# Run on desktop (uses software framebuffer)
./target/x86_64-unknown-linux-gnu/debug/plato

# Run with debug framebuffer output
PLATO_DEBUG_FB=tmp/framebuffer.png ./target/x86_64-unknown-linux-gnu/debug/plato

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

## Build Status

| Component           | Status  | Notes                        |
|---------------------|---------|------------------------------|
| Rust Toolchain      | ✅      | 1.93.0, all targets          |
| Native Dependencies | ✅      | Pure Rust (no C libs)        |
| Desktop x86_64      | ✅      | Software framebuffer backend |
| ARM Build           | ✅      | Successfully builds for Kobo |
| Unit Tests          | ✅      | 273 tests, all passing       |
| Clippy              | ✅      | Zero warnings                |
| Formatting          | ✅      | rustfmt.toml configured      |

---

## 📚 Project Structure

| Crate         | Purpose                                                |
|---------------|--------------------------------------------------------|
| `plato-core`  | Core library: document handling, rendering, UI, device |
| `plato`       | Main binary for Kobo devices                           |
| `importer`    | Document importer tool                                 |
| `fetcher`     | Article fetcher from online sources                    |
| `epub_edit`   | EPUB editing library                                   |
| `epub_editor` | EPUB editing CLI tool                                  |

---

## 🎯 Quick Workflow

1. Make changes → `cargo fmt` → `cargo check --target x86_64-unknown-linux-gnu`
2. `cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings`
3. Commit with descriptive messages
