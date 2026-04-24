# Contributing to Plato

Thank you for your interest in contributing to Plato! This guide will help you get started quickly and follow our project standards.

## 📚 Quick Start

**New contributors? Start here:**

1. **Clone & Setup**

   ```bash
   git clone https://github.com/bgamari/plato.git
   cd plato
   ./build.sh  # Downloads dependencies and builds for host
   ```

2. **Make Your First Change**

   ```bash
   # Edit code, then validate
   cargo fmt
   cargo clippy
   cargo test --target x86_64-unknown-linux-gnu
   ```

3. **Submit Your Work**

   ```bash
   git add .
   git commit -m "feat: add your feature"
   git push origin your-branch-name
   ```

## 📖 Essential Resources

- [README.md](README.md) - Project overview and supported devices
- [doc/BUILD.md](doc/BUILD.md) - Detailed setup and build instructions
- [AGENTS.md](AGENTS.md) - Repository-specific coding guidance
- [Cargo.toml](Cargo.toml) - Project structure and dependencies

## 🏗️ Development Setup

### Project Structure

```text
plato/
├── crates/
│   ├── core/           # Shared engine and main source code
│   ├── plato/          # Device binary (main application)
│   ├── importer/       # plato-import binary
│   ├── fetcher/        # article_fetcher binary
│   └── epub_edit/       # EPUB editing library
├── epub_editor/        # Standalone CLI tool (separate from workspace)
├── doc/               # Documentation
├── fonts/             # Font files
└── css/               # Stylesheets
```

### Prerequisites

**Required:**

- Rust toolchain (stable)
- Git
- Basic build tools: `wget`, `curl`, `pkg-config`, `unzip`, `jq`, `patchelf`

**For cross-compilation:**

- ARM toolchain (for Kobo device builds)

### Build Commands

**Development (Host):**

```bash
# Quick build for testing
cargo build --target x86_64-unknown-linux-gnu -p plato

# Build with optimizations
cargo build --release --target x86_64-unknown-linux-gnu -p plato
```

**Production (ARM Devices):**

```bash
# 32-bit ARM (original Kobo devices) - DEFAULT
cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato

# 64-bit ARM (newer Kobo: Libra 2, Sage, Clara 2E)
cargo build --target aarch64-unknown-linux-gnu --profile release-arm64 -p plato
```

**Full Build System:**

```bash
# Complete build with all dependencies
./build.sh

# Build with options (skip clean, specific target)
./build.sh --no-clean arm skip

# Create distribution package
./dist.sh
```

### Testing

**Important:** Default target is ARM, so host testing requires `--target x86_64-unknown-linux-gnu`.

**Common Test Commands:**

```bash
# Run all tests
cargo test --target x86_64-unknown-linux-gnu

# Test specific crate
cargo test -p plato-core --target x86_64-unknown-linux-gnu

# Run single test
cargo test -p plato-core test_device_canonical_rotation --target x86_64-unknown-linux-gnu

# Test module
cargo test -p plato-core geom::tests --target x86_64-unknown-linux-gnu

# Pattern matching
cargo test overlapping --target x86_64-unknown-linux-gnu
```

**Development Workflow:**

```bash
# During development - faster checks
cargo check --target x86_64-unknown-linux-gnu
cargo clippy --target x86_64-unknown-linux-gnu

# Before committing - full validation
cargo fmt
cargo test --target x86_64-unknown-linux-gnu
```

## 🎨 Code Style & Standards

### Formatting & Quality

**Before every commit:**

```bash
cargo fmt      # Format code
cargo clippy  # Check for issues
cargo test    # Run tests
```

### Code Organization

**Import Structure:**

```rust
// Standard library first
use std::collections::HashMap;
use std::path::Path;

// External crates
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

// Local modules
use crate::document::Document;
use crate::view::View;
```

**Naming Conventions:**

- `snake_case` - functions, methods, variables, modules
- `PascalCase` - types, structs, enums, traits  
- `SCREAMING_SNAKE_CASE` - true constants
- Prefix unused items with `_`

**Struct Best Practices:**

```rust
#[derive(Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: String,           // Prefer pub fields for internal types
    #[serde(skip_serializing_if = "String::is_empty")]
    pub title: String,
    content: Box<str>,        // Use Box for large data on constrained devices
}
```

### Error Handling

**Standard approach - use anyhow:**

```rust
use anyhow::{bail, Context, Result};

pub fn load_document(path: &Path) -> Result<Document> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    if content.is_empty() {
        bail!("Document cannot be empty");
    }
    
    parse_document(&content)
}
```

### Performance Guidelines

**Hot path optimizations:**

```rust
#[inline]
pub fn pixel_to_device(px: f32) -> i32 {
    (px * DEVICE_SCALE).round() as i32
}

// Use pre-allocated buffers
let mut buffer = String::with_capacity(estimated_size);

// Prefer efficient collections
use rustc_hash::FxHashMap;  // Instead of std::HashMap
use std::collections::BTreeMap;  // For ordered data
```

### Architecture Principles

**DRY - Don't Repeat Yourself:**

- Extract common logic into helper functions and modules
- Use shared factory functions for repeated patterns
- Eliminate structural duplication with generics and traits

**Modular Design:**

- **File limit:** 1000 lines max - split into submodules
- **Function limit:** 50 lines max - extract helpers
- **Single responsibility:** Each module has one clear purpose
- **Visibility:** Use `pub(crate)` for internal sharing

**Module Organization:**

```text
crates/
├── core/
│   ├── document/     # Document handling (pdf, epub, etc.)
│   ├── view/         # UI views and interaction
│   ├── device/       # Device-specific code
│   └── geom/         # Geometry and math
└── plato/            # Main application
```

**Testing Standards:**

```rust
// Unit tests - same directory
#[cfg(test)]
mod document_tests {
    use super::*;
    
    #[test]
    fn test_document_loading() { /* ... */ }
}

// Integration tests - tests/ directory
// tests/document_integration.rs
```

### Configuration & Validation

**Typed Configuration:**

```rust
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_font_size")]
    pub font_size: u8,
    
    #[validate(range(min = 1, max = 100))]
    pub max_pages: usize,
}

impl Default for Config {
    fn default() -> Self { /* ... */ }
}
```

## 🔄 Development Workflow

### Pre-Commit Checklist

**Always run before committing:**

```bash
cargo fmt                    # Format code
cargo clippy -- -D warnings  # Treat warnings as errors
cargo test --target x86_64-unknown-linux-gnu  # Run tests
cargo audit                  # Check for security issues
```

### Incremental Development

**Work in small, verifiable steps:**

1. Make one focused change
2. `cargo check` to verify compilation
3. `cargo test` to verify behavior
4. Commit with clear message
5. Repeat

**Error Resolution Order:**

1. Fix dependency conflicts
2. Resolve import issues
3. Fix type mismatches
4. Add missing implementations
5. Validate full build

### Build Standards

**Zero-tolerance policy:**

- No warnings allowed (`cargo clippy -- -D warnings`)
- All tests must pass
- All targets must compile
- Code must be formatted

**Build targets:**

```bash
# Primary development target
cargo build --target x86_64-unknown-linux-gnu

# Production targets (before PR)
cargo build --target arm-unknown-linux-gnueabihf
cargo build --target aarch64-unknown-linux-gnu
```

## 🐛 Troubleshooting

### Common Build Issues

**Cross-compilation errors:**

```bash
# Install ARM target
rustup target add arm-unknown-linux-gnueabihf
rustup target add aarch64-unknown-linux-gnu

# Check toolchain
rustup show
```

**Dependency issues:**

```bash
# Clean and rebuild
cargo clean
cargo update
./build.sh --clean
```

**Test failures on host:**

```bash
# Always specify host target for testing
cargo test --target x86_64-unknown-linux-gnu

# Run specific failing test with output
cargo test --target x86_64-unknown-linux-gnu -- --nocapture test_name
```

### Performance Issues

**Slow compilation:**

- Use `cargo check` during development
- Consider `cargo build` only when needed
- Check for unnecessary dependencies in `Cargo.toml`

**Memory issues on device:**

- Use `Box<T>` for large data structures
- Avoid deep clones in hot paths
- Monitor memory usage with profiling tools

### Getting Help

**Resources:**

- Check [doc/BUILD.md](doc/BUILD.md) for detailed setup
- Review [AGENTS.md](AGENTS.md) for coding standards
- Search existing GitHub issues
- Ask questions in GitHub discussions

**Debug information:**

```bash
# Enable debug logging
RUST_LOG=debug cargo run --target x86_64-unknown-linux-gnu

# Build with debug info
cargo build --target x86_64-unknown-linux-gnu --features debug
```

## 📝 Documentation Standards

### Module Documentation

**Every `mod.rs` should start with:**

```rust
//! Document handling for Plato.
//! 
//! This module provides parsing and rendering support for various document formats
//! including PDF, EPUB, and text files. It coordinates with the view system to
//! display content on Kobo e-ink displays.
//! 
//! # Examples
//! 
//! ```
//! use plato_core::document::Document;
//! let doc = Document::open("book.epub")?;
//! let page = doc.page(0)?;
//! ```
//!
//! # Safety
//! 
//! This module uses FFI bindings to MuPDF and must ensure proper memory management.
```

### API Documentation

**Public APIs require complete docs:**

```rust
/// Loads a document from the given path.
/// 
/// # Arguments
/// 
/// * `path` - Path to the document file
/// 
/// # Returns
/// 
/// A `Result` containing the loaded `Document` or an error
/// 
/// # Errors
/// 
/// Returns an error if:
/// - The file doesn't exist
/// - The file format is unsupported
/// - The file is corrupted
/// 
/// # Examples
/// 
/// ```
/// let doc = Document::open("book.pdf")?;
/// println!("Loaded {} pages", doc.page_count());
/// ```
pub fn open<P: AsRef<Path>>(path: P) -> Result<Document>
```

## 🚀 Submitting Changes

### Commit Messages

**Format:** `<type>: <description>`

**Types:**

- `feat` - New feature
- `fix` - Bug fix  
- `docs` - Documentation
- `style` - Formatting (no code change)
- `refactor` - Code refactoring
- `test` - Adding/updating tests
- `chore` - Maintenance tasks

**Examples:**

```bash
feat: add EPUB font size configuration
fix: resolve PDF rendering crash on large documents
docs: update build instructions for ARM64
```

### Pull Request Guidelines

**Before submitting:**

- [ ] All tests pass: `cargo test --target x86_64-unknown-linux-gnu`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code is formatted: `cargo fmt`
- [ ] Documentation is updated
- [ ] PR description explains the change
- [ ] Tests cover new functionality

**PR best practices:**

- Keep changes focused and reviewable
- Link to relevant issues: `Fixes #123`
- Use draft PRs for work-in-progress
- Respond to feedback promptly

### Issue Reporting

**Bug reports should include:**

- Device model and firmware version
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs or screenshots

**Feature requests should include:**

- Use case and motivation
- Proposed implementation approach
- Potential alternatives considered

## 📄 License

By contributing to Plato, you agree that your contributions will be licensed under the same [AGPL-3.0](LICENSE-AGPLv3) license as the project.

---

**Thank you for contributing to Plato! 🎉**

If you need help, don't hesitate to ask in GitHub discussions or issues.
