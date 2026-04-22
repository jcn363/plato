# Build

This document covers the supported ways to build Plato for Kobo devices and for host development.

Start by cloning the repository:

```sh
git clone https://github.com/baskerville/plato.git
cd plato
```

## Plato

### Preliminary

Install the appropriate [compiler toolchain](https://drive.google.com/drive/folders/1YT6x2X070-cg_E8iWvNUUrWg5-t_YcV0) (the binaries of the `bin` directory need to be in your path).

Install the required dependencies: `wget`, `curl`, `git`, `pkg-config`, `unzip`, `jq`, `patchelf`.

Install *rustup*:

```sh
curl https://sh.rustup.rs -sSf | sh
```

Install the appropriate targets:

```sh
rustup target add arm-unknown-linux-gnueabihf
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-gnu
```

The Rust workspace contains `crates/core`, `crates/plato`, `crates/emulator`, `crates/importer`, `crates/fetcher`, `crates/epub_edit`, `crates/epub_editor`, and `crates/plato-android`.

## Build Phase

```bash
./build.sh [OPTIONS] [TARGET] [METHOD]
```

This script will:

1. Check for required tools (cargo, rustc, cross-compiler)
2. Handle thirdparty libraries (download or build)
3. Ensure necessary symlinks are in the library directory
4. Run `cargo fmt` and `cargo clippy --workspace` (with target-specific exclusions)
5. Build the workspace crates (optionally skipping the emulator for ARM)

### Common Options

- `--no-clean`: Skip `cargo clean`
- `--no-clippy`: Skip `cargo clippy`
- `--no-fmt`: Skip `cargo fmt`
- `-j JOBS`: Number of parallel jobs (default: number of CPU cores)

### Target

- `arm` (default), `arm64`, `host`

### Method

- `fast` (default): Download pre-compiled libraries
- `slow`: Build libraries from source
- `skip`: Use existing libraries

Example:

```bash
./build.sh --no-clean arm skip
```

PDF rendering is now handled by PDFPurr, a pure Rust library, so no MuPDF wrapper is required.

## Alternative Build Commands

You can also build directly with Cargo for specific targets:

```bash
# Build for 32-bit ARM (original Kobo devices) — DEFAULT
cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato

# Build for 64-bit ARM (newer Kobo devices: Libra 2, Sage, Clara 2E, Elipsa 2E, etc.)
cargo build --target aarch64-unknown-linux-gnu --profile release-arm64 -p plato

# Build for host (development/testing)
cargo build --target x86_64-unknown-linux-gnu -p plato

# Build the desktop emulator binary
cargo build --target x86_64-unknown-linux-gnu -p emulator

# Build the importer helper
cargo build --target x86_64-unknown-linux-gnu -p importer

# Build the article fetcher
cargo build --target x86_64-unknown-linux-gnu -p fetcher
```

## Distribution

```bash
./dist.sh
```

## Developer Tools

Install the required dependencies: *DjVuLibre*, *FreeType*, *HarfBuzz*.

PDF rendering is now handled by PDFPurr, a pure Rust library, so no MuPDF installation is required.

### Emulator

Install one additional dependency: *SDL2*.

You can then run the emulator with:

```bash
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

### Importer

You can install the importer with:

```bash
./install-importer.sh
```
