# Testing Guide

This document explains how to run and write tests for the Plato project, and describes the testing philosophy as outlined in AGENTS.md.

## Running tests

All tests must be executed for the **host** target (x86_64) because the default workspace target is ARM.

```bash
# Run all tests
cargo test --target x86_64-unknown-linux-gnu

# Run tests for a specific crate
cargo test -p plato-core --target x86_64-unknown-linux-gnu

# Run a single test by name
cargo test -p plato-core test_device_canonical_rotation --target x86_64-unknown-linux-gnu

# Run tests in a specific module
cargo test -p plato-core geom::tests --target x86_64-unknown-linux-gnu
```

## Test performance requirements

**Mandatory rule:** Tests must complete quickly; slow tests are a sign of poor test design.

- **60‑second threshold** – Any test running longer than 60 seconds must be either:
  - Rewritten from scratch with better performance characteristics
  - Removed entirely if it cannot be made fast (indicates a design flaw or unnecessary scope)
- **Fast feedback loops** – Unit tests should complete in milliseconds; integration tests should complete in seconds
- **Parallel test execution** – Structure tests to allow `cargo test` to run them in parallel without conflicts

## Test segregation

**Mandatory rule:** Strictly separate test code from production code to avoid contamination and overhead.

- **Unit tests** must be in the same directory as production code using sibling test files (e.g., `loop.rs` and `loop_tests.rs`)
- Test files should include a `mod loop;` (or `use super::*;`) to access the production code they test
- **Integration tests** go in `tests/` directory at the workspace or crate root
- Test‑only helpers, fixtures, and utilities must live in test files or separate test‑only crates
- Never gate production behavior on `cfg(test)` – the compiled binary should be identical whether tests exist or not
- Avoid test‑specific dependencies leaking into the main dependency tree; use `[dev-dependencies]` in `Cargo.toml`

## Using mocks

The crate `plato_core::test_mocks` provides mock implementations of the core traits:

- `MockFramebuffer` – headless framebuffer for rendering tests
- `MockFrontlight` – configurable frontlight
- `MockBattery` – battery with fake capacity/status
- `MockDevice` – device with programmable model, dimensions, DPI, etc.
- `MockLightSensor` – light sensor returning a fixed value
- `MockDocument` – document with a few pages of dummy text

Example:

```rust
use plato_core::test_mocks::MockDevice;
use plato_core::Device;

#[test]
fn test_something_with_mock_device() {
    let device = MockDevice::new(plato_core::Model::Forma);
    assert_eq!(device.model(), plato_core::Model::Forma);
}
```

## Lint and format checks

Before committing, always run:

```bash
# Format code
cargo fmt

# Check for clippy warnings (treat warnings as errors)
cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings
```

## CI expectations

The GitHub Actions workflow (`.github/workflows/rust.yml`) runs:

1. `cargo build --verbose`
2. `cargo clippy -- -D warnings`
3. `cargo test --verbose`
4. `cargo audit` (security)

All steps must pass before a pull request can be merged.
