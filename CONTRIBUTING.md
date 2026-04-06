# Contributing to Plato

Thank you for your interest in contributing to Plato! This document outlines the guidelines and best practices for contributing to this project.

## Development Setup

### Prerequisites
- Rust toolchain (stable)
- Git
- For ARM builds: Cross-compilation toolchain
- For emulator: SDL2 development libraries

### Building
```bash
# Build for 32-bit ARM (original Kobo devices) - DEFAULT
cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato

# Build for 64-bit ARM (newer Kobo devices: Libra 2, Sage, Clara 2E, etc.)
cargo build --target aarch64-unknown-linux-gnu --profile release-arm64

# Build for host (development/testing)
cargo build --target x86_64-unknown-linux-gnu

# Full build with native dependencies (downloads libs + MuPDF)
./build.sh

# Create distribution bundle
./dist.sh

# Run the desktop emulator (requires SDL2)
./run-emulator.sh
```

### Testing
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

## Code Style Guidelines

### Formatting
- Run `cargo fmt` before committing
- Run `cargo clippy` to catch common issues
- The project uses `rustfmt.toml` for consistent formatting

### Imports
- Group imports: std library first, then external crates, then local `crate::` imports
- Use explicit imports rather than glob (`use std::fmt` not `use std::fmt::*`)
- Re-export commonly used types from `lib.rs`

### Naming
- **snake_case**: functions, methods, variables, modules, constants
- **PascalCase**: types, structs, enums, traits
- **SCREAMING_SNAKE_CASE**: true constants (`const DEFAULT_FONT_SIZE`)
- Prefix unused parameters or dead code markers with `_`

### Types & Structs
- Derive common traits: `#[derive(Debug, Clone)]` for structs, add `Copy, Eq, PartialEq` when appropriate
- Use `#[serde(rename_all = "camelCase")]` or `kebab-case` for serialization
- Use `#[serde(skip_serializing_if = "...")]` to omit empty/default fields
- Prefer `pub` fields on structs over getters for internal types
- Use the builder pattern for complex configurations
- Ensure proper resource cleanup in error cases — implement `Drop` for types that own resources
- Monitor memory usage on resource-constrained devices — use `Box` for large data structures

### Input Validation
**Mandatory:** Validate all inputs, especially at public API boundaries.
- Validate early and fail fast — reject invalid inputs before any side effects occur
- Provide clear, actionable error messages
- Use the `validator` crate for complex validation scenarios
- Validate configuration values, user input, file contents, and network responses

### Error Handling
**Mandatory:** Standardize on a single error handling approach.
- Use `anyhow::Error` as the primary error type throughout
- Use `bail!` for early returns with errors
- Use `format_err!` to create ad-hoc errors
- Use `.with_context(|| "...")` to add context to errors
- Use `thiserror` for defining custom error types in libraries
- Avoid `unwrap()` — prefer `?`, `unwrap_or`, `unwrap_or_default`, or explicit `match`
- For lock poisoning, use `.expect("lock_name lock poisoned")` instead of `.unwrap()`

### Performance
- Use `#[inline]` on hot-path small functions (pixel operations, geometry math, device checks)
- Use `FxHashMap`/`FxHashSet` from `fxhash` instead of std `HashMap` for non-cryptographic use
- Pre-allocate buffers with `String::with_capacity` when size is known
- Prefer `Cow<str>` for conditional string ownership

### DRY (Don't Repeat Yourself)
**Mandatory:** Never duplicate code.
- Extract common logic into helper functions, traits, or modules
- Create shared factory functions for repeated initialization patterns
- Refactor repeated match/if patterns into methods on relevant types
- Use generics, traits, or macros to eliminate structural duplication
- Constants repeated across files belong in a shared `consts` module

### Modular Design
**Mandatory:** Keep files and functions focused and reasonably sized.
- No source file should exceed **1000 lines** — split into submodules when approaching this limit
- No function should exceed **50 lines** — extract inner logic into helpers when approaching this limit
- Each module should have a single clear responsibility
- Split large modules into smaller, more focused ones when handling multiple distinct concerns
- Separate data structures, business logic, and I/O into distinct modules
- Use `pub(crate)` visibility to share helpers within a crate without exposing them publicly

### Module Hierarchy
**Mandatory:** Structure modules logically, avoid circular dependencies, and document purposes.
- Group related functionality by domain (e.g., `document/pdf`, `document/epub`, `view/reader`)
- Avoid circular dependencies between modules
- Document each module's purpose at the top of its `mod.rs` file

### Single Source of Truth
**Mandatory:** Every piece of knowledge or logic must have one authoritative location.
- Define values that can change once and reference them everywhere
- Store mappings in one place and derive the rest
- Export constants from the module that owns the concept
- Avoid shadowing or overriding the same data in multiple layers

### Configuration Management
**Mandatory:** Centralize configuration management and validate all configuration values.
- Group related configuration in dedicated structs or modules
- Add validation for configuration values at load time
- Use typed configuration over raw strings or magic numbers
- Document all configuration options, their valid ranges, and default values
- Validate configuration values against constraints before use

### Test Segregation
**Mandatory:** Strictly separate test code from production code.
- **Unit tests**: Same directory as production code using sibling test files (`{module}_tests.rs`)
- **Integration tests**: `tests/` directory at workspace or crate root
- Test-only helpers must live in test files or separate test-only crates
- Never gate production behavior on `cfg(test)`
- Each test file should be named `{module}_tests.rs`
- Group related tests using modules
- Add integration tests that exercise multiple components together

### General Patterns
- Use `lazy_static!` for global statics requiring runtime initialization
- Use `bitflags!` for flag enums
- Prefer `BTreeMap`/`BTreeSet` for ordered collections; `IndexMap` for insertion-ordered maps
- Keep `mod` declarations alphabetical; use `pub mod` for public API, plain `mod` for internal

## Dependency Management
**Mandatory:** Regularly audit dependencies for security and maintainability.
- Use `cargo-audit` to check for known vulnerabilities
- Audit and update dependencies regularly
- Use workspace inheritance for shared dependency versions
- Pin major versions and avoid wildcard dependencies

## Async Patterns
- Document `Send` and `Sync` bounds for async code
- Add deadlock detection for code using multiple locks
- Use `tracing` for better async debugging
- Prefer `tokio` or `async-std` runtime primitives

## API Documentation
- Add examples for all public APIs in rustdoc comments
- Document safety requirements for `unsafe` functions
- Use `///` for public API documentation and `//` for internal notes
- Keep examples minimal but complete

## Automation
**Mandatory:** Use scripts for building, testing, linting, formatting, and deployment.
- Always run `cargo fmt` and `cargo clippy` before considering a task complete
- Use `cargo test` to verify changes
- Prefer `cargo check` over `cargo build` during development
- Batch changes and run single validation pass
- Cross-compilation targets ARM by default — use `--target x86_64-unknown-linux-gnu` for host builds

## Error Handling Process
**Mandatory:** Address errors in small increments, commit frequently, and review for accuracy.
- Fix one category of error at a time
- Run `cargo check` or `cargo test` after each small batch
- Commit working changes frequently with clear messages
- Review changes for grammatical, factual, and logical correctness
- Never leave the codebase in a broken state

### Error Resolution Sequence
When facing multiple compilation errors:
1. Dependency issues — fix version conflicts
2. Import resolution — validate module imports
3. Type mismatches — harmonize type definitions
4. Missing implementations — add missing methods/traits/types
5. Validate compilation and testing

## Task Discipline
**Mandatory:** Stay focused, validate incrementally, and prefer composition.
- One task at a time — avoid concurrent operations
- Decompose incrementally — break complex tasks into manageable steps
- Prefer composition over inheritance — build flexible systems through traits

## Build Verification
**Mandatory:** Achieve zero warnings and zero errors on every build target.

### Systematic Build Process
1. Incremental verification — compile for primary target after each change
2. Zero-tolerance policy — treat warnings as errors
3. Full build verification — clean build for all targets before considering task complete
4. Clippy validation — run clippy on host target after significant changes

### Task Decomposition
- One concern per change — isolate refactoring from functional changes
- Smallest viable diff — prefer several focused commits
- Verify before proceeding — compile successfully after each atomic change

### Code Quality Principles
- Rewrite over patch — rewrite files with significant technical debt
- Rust idioms only — follow current Rust best practices
- Root cause analysis — fix root causes, not surface-level workarounds
- Eliminate dead code — remove unused code immediately
- No backward compatibility — don't support old APIs unless explicitly required
- Project containment — all files must be inside project root

## Documentation Requirements

### Module Documentation
Each module should have a purpose statement at the top of its `mod.rs` file:
```rust
//! This module handles [specific responsibility].
//! 
//! It provides [key features] and interacts with [related modules].
//! 
//! # Examples
//! 
//! ```
//! use plato_core::module::Function;
//! let result = Function::new();
//! ```
//!
//! # Safety
//!
//! This module [safety considerations if applicable].
```

### API Documentation
All public APIs must have rustdoc comments with:
- Clear description of what the function does
- Parameters and return values explained
- `# Examples` section with runnable code
- `# Errors` section if the function can return `Result`
- `# Panics` section if the function can panic
- `# Safety` section for `unsafe` functions

### Architectural Documentation
Significant architectural decisions should be documented in:
- `docs/architecture/` directory
- Referenced from relevant module documentation
- Including rationale, trade-offs, and alternatives considered

## Commit Guidelines

### Commit Messages
- Use clear, descriptive messages
- Format: `<type>: <description>` (e.g., `feat: add login functionality`)
- Types: `feat` (feature), `fix` (bug fix), `docs` (documentation), `style` (formatting), `refactor`, `test`, `chore`
- Keep subject line under 50 characters
- Provide detailed explanation in body when needed
- Reference related issues: `Fixes #123` or `Related to #456`

### Pull Requests
- Keep PRs focused on a single concern
- Update documentation when changing functionality
- Ensure all tests pass
- Run `cargo fmt` and `cargo clippy` before submitting
- Keep PR size reasonable for review
- Respond to reviewer feedback promptly

## Reporting Issues
When reporting bugs or suggesting features:
- Check if issue already exists
- Provide clear, reproducible steps for bugs
- Include relevant logs, screenshots, or error messages
- For feature requests, explain the use case and benefits
- Follow the issue template if provided

## License
By contributing to Plato, you agree that your contributions will be licensed under the project's license.