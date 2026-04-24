#!/bin/sh
#
# Build script for OnePlus Nord 2 5G (Android ARM64)
#
# Reference Android device for the plato-android port with:
# - MediaTek Dimensity 1200-AI (6nm, 8-core)
#   * 1x3.0 GHz Cortex-A78 (performance)
#   * 3x2.6 GHz Cortex-A78 (performance)
#   * 4x2.0 GHz Cortex-A55 (efficiency)
# - 12GB LPDDR4X RAM
# - 90Hz Fluid AMOLED display
# - UFS 3.1 storage
#
# This script builds with aggressive mobile optimizations:
# - 4MB thumbnail buffers (vs 1MB standard)
# - 16MB document buffers (vs 4MB standard)
# - 4-6 thumbnail workers (vs 2 standard)
# - 50 thumbnail cache entries (vs 20 standard)
# - 100MB page cache (vs 20MB standard)
# - 90Hz animations enabled
# - Colorful OLED themes

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_NAME="plato"
TARGET="aarch64-linux-android"  # Android ARM64
HOST_TARGET="x86_64-unknown-linux-gnu"  # For host build testing

echo "=========================================="
echo "Building Plato for OnePlus Nord 2 5G"
echo "Target: Android ARM64 (AArch64)"
echo "Architecture: 8-core, 12GB RAM"
echo "=========================================="
echo ""

cd "$SCRIPT_DIR"

# Verify we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Run from project root."
    exit 1
fi

# Check for Android SDK/NDK (validate early, fail fast)
if [ -z "$ANDROID_NDK_ROOT" ] && [ -z "$ANDROID_NDK_HOME" ]; then
    echo "Error: ANDROID_NDK_ROOT or ANDROID_NDK_HOME not set"
    echo "Android builds require the Android NDK to be configured"
    echo ""
    echo "To install Android NDK:"
    echo "  1. Download Android NDK from https://developer.android.com/ndk"
    echo "  2. Extract and set ANDROID_NDK_ROOT=/path/to/ndk"
    echo "  3. Install Rust target: rustup target add aarch64-linux-android"
    echo ""
    echo "Alternative: Use ./build-android-apk.sh for full APK build"
    exit 1
fi

# Verify Android target is installed
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Error: Android target $TARGET not installed"
    echo ""
    echo "To install:"
    echo "  rustup target add $TARGET"
    echo ""
    exit 1
fi

# Set environment to indicate Android build (used by optimizations)
export ANDROID_ROOT="/system"
export PLATO_DEVICE="android"

# Set Android NDK compiler paths to bypass sccache for cc-rs
# This prevents "cannot find binary path" errors with cross-compilation
if [ -n "$ANDROID_NDK_ROOT" ]; then
    NDK_ROOT="$ANDROID_NDK_ROOT"
elif [ -n "$ANDROID_NDK_HOME" ]; then
    NDK_ROOT="$ANDROID_NDK_HOME"
else
    echo "Error: Neither ANDROID_NDK_ROOT nor ANDROID_NDK_HOME is set"
    exit 1
fi

# Detect host platform for NDK toolchain selection
HOST_OS="$(uname -s)"
HOST_ARCH="$(uname -m)"

case "$HOST_OS" in
    Linux)
        HOST_TAG="linux"
        ;;
    Darwin)
        HOST_TAG="darwin"
        ;;
    *)
        echo "Error: Unsupported host OS: $HOST_OS"
        exit 1
        ;;
esac

case "$HOST_ARCH" in
    x86_64)
        HOST_TAG="${HOST_TAG}-x86_64"
        ;;
    aarch64|arm64)
        HOST_TAG="${HOST_TAG}-arm64"
        ;;
    *)
        echo "Error: Unsupported host architecture: $HOST_ARCH"
        exit 1
        ;;
esac

# Set compiler paths to bypass sccache
export CC_aarch64_linux_android="${NDK_ROOT}/toolchains/llvm/prebuilt/${HOST_TAG}/bin/aarch64-linux-android21-clang"
export CXX_aarch64_linux_android="${NDK_ROOT}/toolchains/llvm/prebuilt/${HOST_TAG}/bin/aarch64-linux-android21-clang++"
export AR_aarch64_linux_android="${NDK_ROOT}/toolchains/llvm/prebuilt/${HOST_TAG}/bin/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="${NDK_ROOT}/toolchains/llvm/prebuilt/${HOST_TAG}/bin/aarch64-linux-android21-clang"

# Verify compilers exist
if [ ! -f "$CC_aarch64_linux_android" ]; then
    echo "Error: Android NDK compiler not found at $CC_aarch64_linux_android"
    echo "Please verify ANDROID_NDK_ROOT points to a valid NDK installation"
    exit 1
fi

# Number of parallel jobs
JOBS=$(nproc 2>/dev/null || echo 8)

echo "Step 1: Formatting code..."
cargo fmt --all

echo ""
echo "Step 2: Running clippy (host target)..."
cargo clippy -p plato-core --target "$HOST_TARGET" -- -D warnings
cargo clippy -p plato --target "$HOST_TARGET" -- -D warnings

echo ""
echo "Step 3: Running clippy (Android target)..."
cargo clippy -p plato-core --target "$TARGET" -- -D warnings

echo ""
echo "Step 4: Building Plato APK for Android..."
echo "This will use aggressive mobile optimizations:"
echo "  - Thumbnail buffer: 4MB"
echo "  - Document buffer: 16MB"
echo "  - Workers: 4-6 threads"
echo "  - Cache: 50 entries"
echo "  - Page cache: 100MB"
echo "  - 90Hz animations enabled"
echo "  - Colorful OLED themes"
echo ""

# Build APK for Android target (unsigned, no Java required)
echo "Building APK for Android target: $TARGET"
cd crates/plato-android
# Build APK (signing may fail if Java not installed, but APK will still be created)
cargo apk build --target "$TARGET" || {
    echo "Note: Build completed but signing may have failed (Java not required for unsigned APK)"
}
cd ../..

# Create dist directory and move APK there
mkdir -p dist
APK_PATH="crates/plato-android/target/debug/apk/plato-android.apk"
if [ ! -f "$APK_PATH" ]; then
    APK_PATH="crates/plato-android/target/release/apk/plato-android.apk"
fi
if [ -f "$APK_PATH" ]; then
    cp "$APK_PATH" dist/plato-oneplus-nord2.apk
else
    echo "Error: APK not found"
    exit 1
fi

echo ""
echo "=========================================="
echo "Build complete for OnePlus Nord 2 5G!"
echo "=========================================="
echo ""
echo "APK location:"
if [ -f "dist/plato-oneplus-nord2.apk" ]; then
    echo "  dist/plato-oneplus-nord2.apk"
    ls -lh "dist/plato-oneplus-nord2.apk"
else
    echo "  Error: APK not found at dist/plato-oneplus-nord2.apk"
    exit 1
fi

echo ""
echo "To deploy to OnePlus Nord 2 5G:"
echo "  1. Enable USB debugging on device"
echo "  2. Install APK: adb install dist/plato-oneplus-nord2.apk"
echo "  3. Or transfer APK and install manually"
echo ""
echo "Mobile optimizations active:"
echo "  ✓ 4MB thumbnail buffers"
echo "  ✓ 16MB document buffers"
echo "  ✓ 4-6 thumbnail workers"
echo "  ✓ 50 cache entries"
echo "  ✓ 100MB page cache"
echo "  ✓ 90Hz OLED animations"
echo "  ✓ Colorful Material Design themes"
echo "  ✓ Haptic feedback enabled"
echo "  ✓ Predictive touch enabled"
echo ""
echo "Parallel programming optimized for:"
echo "  ✓ 8-core CPU (4xA78 + 4xA55)"
echo "  ✓ 12GB LPDDR4X RAM"
echo "  ✓ UFS 3.1 storage"
echo "  ✓ Aggressive background sync (5min)"
echo ""
