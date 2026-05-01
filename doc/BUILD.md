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

## Build Phase

```bash
./build.sh [OPTIONS] [TARGET] [METHOD]
```

This script will:

1. Check for required tools (cargo, rustc, cross-compiler)
2. Run `cargo fmt` and `cargo clippy --workspace` (with target-specific exclusions)
3. Build the project using Cargo

Note: The project uses pure Rust libraries. No external C dependencies are required.

### Common Options

- `--no-clean`: Skip `cargo clean`
- `--no-clippy`: Skip `cargo clippy`
- `--no-fmt`: Skip `cargo fmt`
- `-j JOBS`: Number of parallel jobs (default: number of CPU cores)

### Target

- `arm` (default): Build for 32-bit ARM Kobo devices
- `arm64`: Build for 64-bit ARM Kobo devices (Libra 2, Sage, Clara 2E, etc.)
- `host`: Build for development machine (x86_64)

### Method (Legacy - No longer used)

The `fast`, `slow`, and `skip` methods are legacy options kept for compatibility. Since the project now uses pure Rust libraries (PDFPurr for PDF rendering, lopdf for PDF manipulation), no external library downloads or builds are required.

Example:

```bash
./build.sh --no-clean arm
```

## Alternative Build Commands

You can also build directly with Cargo for specific targets:

```bash
# Build for 32-bit ARM (original Kobo devices) — DEFAULT
cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato

# Build for 64-bit ARM (newer Kobo devices: Libra 2, Sage, Clara 2E, Elipsa 2E, etc.)
cargo build --target aarch64-unknown-linux-gnu --profile release-arm64 -p plato

# Build for host (development/testing)
cargo build --target x86_64-unknown-linux-gnu -p plato


# Build the importer helper
cargo build --target x86_64-unknown-linux-gnu -p importer

# Build the article fetcher
cargo build --target x86_64-unknown-linux-gnu -p fetcher
```

## Distribution and Desktop Execution

### AppImage

```bash
./dist.sh
```

This script creates an AppImage for x86_64 Linux (`Plato.AppImage`) in the current directory.
The AppImage bundles the Plato binary, fonts, icons, CSS, and the software framebuffer backend.

### Desktop Execution

Plato can now run on desktop Linux systems without requiring a physical framebuffer device (/dev/fb0).

```bash
# Build for desktop
cargo build --target x86_64-unknown-linux-gnu -p plato

# Run directly
./target/x86_64-unknown-linux-gnu/debug/plato

# Run with debug framebuffer output (saves PNG for each update)
PLATO_DEBUG_FB=/tmp/framebuffer.png ./target/x86_64-unknown-linux-gnu/debug/plato
```

The software framebuffer implementation:
- Renders to an in-memory pixel buffer instead of /dev/fb0
- Enables development and testing on standard Linux desktops
- Provides optional PNG export for debugging
- Uses the same Framebuffer trait as hardware implementations
- Produces identical behavior for all non-display operations

**Note**: The current implementation renders to memory only. For a full GUI experience with display output, integration with Wayland/X11 would be needed as a future enhancement.

## Developer Tools

The project has migrated to pure Rust libraries. No native C dependencies are required:

- PDF rendering: PDFPurr (pure Rust)
- Font rendering: skrifa, rustybuzz, ab_glyph (pure Rust)
- Compression: bzip2, flate2 (pure Rust)
- Image handling: image crate (pure Rust)

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
