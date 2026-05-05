# AGENTS.md - Plato Development Guide

Quick facts for AI agents working on this Rust e-reader firmware project.

## Project Overview

- **Type**: Rust document reader for Kobo e-readers
- **Architecture**: Cargo workspace with ~25 crates
- **Default target**: ARM 32-bit (`arm-unknown-linux-gnueabihf`)
- **Key constraint**: Runs on embedded devices with 256MB RAM

## Essential Commands

```bash
# Build (default ARM target)
cargo build --profile release-arm -p plato

# Build for host development (x86_64)
cargo build --target x86_64-unknown-linux-gnu -p plato

# Build for 64-bit ARM devices (Libra 2, Sage, Clara 2E, etc.)
cargo build --target aarch64-unknown-linux-gnu --profile release-arm64 -p plato

# Lint & typecheck (must pass before commit)
cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings

# Format
cargo fmt

# Test (always specify target on this project)
cargo test --target x86_64-unknown-linux-gnu
```

## Key Files & Locations

| Path | Purpose |
|------|---------|
| `crates/core/src/lib.rs` | Core library entry, re-exports |
| `crates/core/src/error.rs` | `PlatoError` enum definition |
| `crates/core/src/settings/` | All settings structs |
| `.cargo/config.toml` | Build target defaults |
| `build.sh` | Full build script (supports --no-clean, --no-clippy, --no-fmt) |

## Critical Conventions

1. **Always use `--target x86_64-unknown-linux-gnu`** for host commands (tests, clippy, check)
2. **No `.unwrap()`** - use `?`, `bail!`, or explicit error handling
3. **Use `PlatoResult<T>`** from `crates/core/src/error.rs` - not `anyhow` for core library code
4. **Add `#[inline]`** to frequently-called small functions (pixel ops, geometry math)
5. **Use `FxHashMap`** from `rustc-hash` for non-cryptographic hashing
6. **Memory**: Use `Box` for large data, `String::with_capacity()` for predictable sizes
7. **Derive traits**: Always `Debug, Clone` at minimum; add `Copy, Eq, PartialEq` when appropriate
8. **Tests**: Put in same file with `#[cfg(test)]` module, not separate files

## Common Pitfalls

- **Forget target flag**: Most `cargo` commands need `--target x86_64-unknown-linux-gnu`
- **Missing reqwest features**: Google Drive sync requires `reqwest` with `query` feature
- **Dead code warnings**: Many structs in `pocket.rs`/`instapaper.rs` intentionally unused - add `#[allow(dead_code)]` at module level
- **Memory on embedded**: Don't allocate large arrays on stack; use `Box` or `Vec`

## Pure Rust Stack

All C dependencies replaced with Rust equivalents:
- **PDF**: `pdfpurr` (replaces MuPDF)
- **Fonts**: `skrifa` + `rustybuzz` (replaces FreeType + HarfBuzz)
- **Compression**: `bzip2`, `djvu-rs` (replaces C libraries)

## Error Handling

Use `PlatoError` in `crates/core/src/error.rs`. Add variants with `#[from]` for automatic conversion:
```rust
#[error("failed to open {path}: {source}")]
pub struct MyError {
    pub path: PathBuf,
    #[from]
    pub source: std::io::Error,
}
```

## Style Checklist (before commit)

- [ ] `cargo fmt` runs clean
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo check --target x86_64-unknown-linux-gnu` passes
- [ ] No `.unwrap()` calls
- [ ] No unused imports
- [ ] Tests pass: `cargo test --target x86_64-unknown-linux-gnu`