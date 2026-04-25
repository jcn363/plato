# Building Plato for iOS from Linux

This guide explains how to build Plato for iOS devices (iPhone and iPad) while developing primarily on Linux. Plato follows a hybrid approach: develop and cross-compile on Linux, then use macOS CI for final packaging.

## Architecture Overview

Plato for iOS uses a multi-stage build process:

1. **Linux stage**: Cross-compile Rust core libraries for iOS targets
2. **macOS stage**: Link with iOS frameworks, sign, and package the final app

This approach leverages Linux for fast development cycles while using macOS only for Apple-specific requirements that cannot be avoided.

## Quick Start

### Prerequisites

On your Linux development machine:

- Rust toolchain with iOS targets
- Docker or Podman (for cross-compilation)
- Git (for pushing to CI)

On macOS (or macOS CI):

- Xcode Command Line Tools
- Xcode (for final builds)
- Apple Developer account (for App Store distribution)

### One-Time Setup

```bash
# Add iOS Rust targets
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

# Install cargo-apk for Android (if needed)
cargo install cargo-apk

# Verify cross-compilation setup
cargo install cross
```

## Build Process

### Option 1: Automated Build (Recommended)

Use the provided automation scripts:

```bash
# Build for iOS device (ARM64)
./build-ios.sh device

# Build for iOS simulator (ARM64 and x86_64)
./build-ios.sh simulator

# Build for both and create universal library
./build-ios.sh universal
```

The `build-ios.sh` script handles:

- Building native dependencies (zlib, bzip2, MuPDF, etc.) for iOS
- Compiling the Rust core library for iOS targets
- Creating universal binaries when needed

### Option 2: Manual Build Steps

#### 1. Build Native Dependencies

```bash
cd thirdparty

# Build each library for iOS
./build-ios.sh

cd ..
```

#### 2. Build MuPDF Wrapper

```bash
cd mupdf_wrapper
./build-ios.sh
cd ..
```

#### 3. Cross-Compile Rust Core

```bash
# For iOS device (ARM64)
cargo build --target aarch64-apple-ios --release -p plato-core

# For iOS simulator (ARM64)
cargo build --target aarch64-apple-ios-sim --release -p plato-core

# For iOS simulator (x86_64)
cargo build --target x86_64-apple-ios --release -p plato-core
```

#### 4. Create Universal Library (on macOS)

```bash
# This step must be done on macOS
lipo -create \
  target/aarch64-apple-ios/release/libplato_core.a \
  target/aarch64-apple-ios-sim/release/libplato_core.a \
  target/x86_64-apple-ios/release/libplato_core.a \
  -output target/universal/libplato_core.a
```

## CI/CD Automation

### GitHub Actions Workflow

The project includes a GitHub Actions workflow (`.github/workflows/ios.yml`) that:

1. Triggers on push to main branch or manual dispatch
2. Runs on macOS runner (required for iOS tooling)
3. Builds native dependencies for iOS
4. Compiles Rust code for all iOS targets
5. Creates universal library with lipo
6. Builds and signs the iOS app
7. Uploads IPA artifact

### Manual CI Trigger

```bash
# Trigger iOS build via GitHub CLI
gh workflow run ios.yml

# Or via GitHub web interface:
# Repository → Actions → iOS Build → Run workflow
```

## Project Structure

```text
plato/
├── crates/
│   ├── core/              # Core library (cross-platform)
│   ├── plato-ios/         # iOS-specific crate (to be created)
│   └── plato-android/     # Android crate (reference implementation)
├── thirdparty/            # Native dependencies
│   ├── build-ios.sh       # iOS native library build script
│   └── ...
├── build-ios.sh           # Main iOS build automation
└── .github/
    └── workflows/
        └── ios.yml        # GitHub Actions CI workflow
```

## Platform Abstraction

Plato uses a platform abstraction layer to support multiple targets:

- **Kobo**: Direct framebuffer access, Linux-based
- **Android**: NDK-based, via `plato-android` crate
- **iOS**: UIKit/Metal-based, via `plato-ios` crate (planned)
- **Desktop/Emulator**: SDL2-based, via `emulator` crate

The core library (`plato-core`) contains all document handling, rendering logic, and UI components that are platform-agnostic.

## iOS-Specific Considerations

### Rendering

- Use Metal for GPU-accelerated rendering
- Adapt e-ink display optimizations for LCD/OLED
- Support high-DPI displays (Retina)
- Handle color display vs grayscale

### Input Handling

- Map touch events to Plato's input system
- Support multi-touch gestures (pinch zoom, pan)
- Adapt physical button UI to touch controls
- Handle iOS-specific gestures (swipe, long press)

### File System

- Adapt to iOS sandboxing restrictions
- Use iOS file picker for document import
- Support iOS Files app integration
- Handle app-specific document directory

### App Lifecycle

- Implement iOS app lifecycle callbacks
- Handle background/foreground transitions
- Support iOS multitasking and split view
- Manage memory pressure warnings

## Testing

### Unit Tests

Unit tests in `plato-core` run on any platform:

```bash
cargo test -p plato-core --target x86_64-unknown-linux-gnu
```

### Integration Tests

iOS-specific integration tests require iOS simulator or device:

```bash
# Run on iOS simulator (macOS only)
xcrun simctl boot "iPhone 15"
cargo test -p plato-ios --target aarch64-apple-ios-sim
```

### Manual Testing

- Test on physical iOS devices
- Verify document rendering accuracy
- Test touch interactions and gestures
- Validate performance on different device classes

## Troubleshooting

### Cross-Compilation Issues

**Problem**: `error: linker not found`

```bash
# Solution: Install cross-compilation toolchain
cargo install cross
```

**Problem**: Missing iOS SDK

```bash
# Solution: This is expected on Linux
# Use macOS CI for final builds
# Or set up osxcross (advanced, not recommended)
```

### Signing Issues

**Problem**: Code signing failed

```bash
# Solution: Ensure Apple Developer credentials are set
# In CI: Use GitHub Actions secrets for certificates
export DEVELOPER_TEAM="Your Team ID"
export DEVELOPER_ID="Your Apple ID"
```

### Dependency Build Failures

**Problem**: Native library fails to build for iOS

```bash
# Solution: Check thirdparty/build-ios.sh
# Ensure iOS SDK paths are correct on macOS
# Verify architecture flags match target
```

## References

- [Plato iOS Plan](APPLE-PLAN.md) - Detailed iOS implementation plan
- [Plato Build Documentation](../doc/BUILD.md) - General build instructions
- [Rust iOS Guide](https://mozilla.github.io/firefox-browser-architecture/experiments/ios-rust-build.html)
- [Cross-Compilation Guide](https://github.com/cross-rs/cross)
- [GitHub Actions macOS](https://docs.github.com/en/actions/using-github-hosted-runners/about-github-hosted-runners#supported-runners-and-hardware-resources)

## Related Documentation

- [Android Build](../build-android-apk.sh) - Reference for mobile platform builds
- [Kobo Build](../build.sh) - Original target platform build
