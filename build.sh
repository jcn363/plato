#!/bin/sh

set -e

# Enhanced build script for Plato project
# Supports building for different targets (arm, arm64, host) with various methods (fast, slow, skip)

# Default values
TARGET="arm"
METHOD="fast"
SKIP_CLEAN=0
SKIP_CLIPPY=0
SKIP_FMT=0
THIRDPARTY_ARGS=""

print_usage() {
    echo "Usage: $0 [OPTIONS] [TARGET] [METHOD] [THIRDPARTY_ARGS...]"
    echo ""
    echo "OPTIONS:"
    echo "  --no-clean       Skip cargo clean"
    echo "  --no-clippy      Skip cargo clippy"
    echo "  --no-fmt         Skip cargo fmt"
    echo "  --help, -h       Show this help message"
    echo ""
    echo "TARGET: arm (default), arm64, host"
    echo "METHOD: fast (default), slow, skip"
    echo ""
    echo "Examples:"
    echo "  $0                    # Build for ARM using fast method"
    echo "  $0 --no-clean arm64   # Build for ARM64 without cleaning"
    echo "  $0 host skip          # Build for host skipping thirdparty steps"
}

# Parse arguments
while [ $# -gt 0 ]; do
    case "$1" in
        --no-clean) SKIP_CLEAN=1; shift ;;
        --no-clippy) SKIP_CLIPPY=1; shift ;;
        --no-fmt) SKIP_FMT=1; shift ;;
        --help|-h) print_usage; exit 0 ;;
        arm|arm64|host) TARGET="$1"; shift; break ;;
        *)
            if [ -n "$1" ] && [ "${1#-}" != "$1" ]; then
                echo "Error: Unknown option '$1'"
                print_usage
                exit 1
            fi
            break
            ;;
    esac
done

if [ $# -gt 0 ]; then
    METHOD="$1"
    shift
fi

THIRDPARTY_ARGS="$@"

# Validate target
case "$TARGET" in
    arm|arm64|host) ;;
    *)
        echo "Error: Invalid target '$TARGET'."
        exit 1
        ;;
esac

# Validate method
case "$METHOD" in
    fast|slow|skip) ;;
    *)
        echo "Error: Invalid method '$METHOD'."
        exit 1
        ;;
esac

echo "Building Plato for target: $TARGET using method: $METHOD"

# Check for required tools
check_tool() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Error: Required tool '$1' not found."
        exit 1
    fi
}

check_tool cargo
check_tool rustc

if [ "$TARGET" = "arm" ]; then
    check_tool arm-linux-gnueabihf-gcc
elif [ "$TARGET" = "arm64" ]; then
    check_tool aarch64-linux-gnu-gcc
fi

# Function to get target-specific cargo flags
get_cargo_target_flags() {
    case "$1" in
        arm) echo "--target=arm-unknown-linux-gnueabihf" ;;
        arm64) echo "--target=aarch64-unknown-linux-gnu" ;;
        host) echo "" ;;
    esac
}

# Function to get target-specific profile
get_cargo_profile() {
    case "$1" in
        arm) echo "--profile release-arm" ;;
        arm64) echo "--profile release-arm64" ;;
        host) echo "--release" ;;
    esac
}

# Function to get library directory name for target
get_lib_dir() {
    case "$1" in
        arm) echo "libs" ;;
        arm64) echo "libs64" ;;
        host) echo "libs_host" ;;
    esac
}

LIB_DIR=$(get_lib_dir "$TARGET")

# Create symlinks for libraries if the directory exists
ensure_symlinks() {
    local dir="$1"
    if [ -d "$dir" ]; then
        echo "Ensuring symlinks in $dir..."
        (
            cd "$dir"
            ln -sf libz.so.1 libz.so 2>/dev/null || true
            ln -sf libbz2.so.1.0 libbz2.so 2>/dev/null || true
            ln -sf libpng16.so.16 libpng16.so 2>/dev/null || true
            ln -sf libjpeg.so.9 libjpeg.so 2>/dev/null || true
            ln -sf libopenjp2.so.7 libopenjp2.so 2>/dev/null || true
            ln -sf libjbig2dec.so.0 libjbig2dec.so 2>/dev/null || true
            ln -sf libfreetype.so.6 libfreetype.so 2>/dev/null || true
            ln -sf libharfbuzz.so.0 libharfbuzz.so 2>/dev/null || true
            ln -sf libgumbo.so.2 libgumbo.so 2>/dev/null || true
            ln -sf libdjvulibre.so.21 libdjvulibre.so 2>/dev/null || true
        )
    fi
}

# Clean previous builds
if [ "$SKIP_CLEAN" -eq 0 ]; then
    echo "Running cargo clean..."
    cargo clean
fi

# Format code
if [ "$SKIP_FMT" -eq 0 ]; then
    echo "Running cargo fmt..."
    cargo fmt
fi

# Run clippy
if [ "$SKIP_CLIPPY" -eq 0 ]; then
    echo "Running cargo clippy..."
    CARGO_TARGET_FLAGS=$(get_cargo_target_flags "$TARGET")
    if [ -n "$CARGO_TARGET_FLAGS" ]; then
        cargo clippy $CARGO_TARGET_FLAGS --workspace --exclude emulator -- -D warnings
    else
        cargo clippy --workspace -- -D warnings
    fi
fi

# Handle thirdparty libraries
THIRDPARTY_DIR="thirdparty"

if [ "$METHOD" = "fast" ]; then
    if [ ! -d "$LIB_DIR" ]; then
        echo "Using fast method - downloading prebuilt libraries"
        ./download.sh "$LIB_DIR/*"
    fi
    ensure_symlinks "$LIB_DIR"
elif [ "$METHOD" = "slow" ]; then
    echo "Building thirdparty libraries from source"
    cd "$THIRDPARTY_DIR"
    ./download.sh $THIRDPARTY_ARGS
    ./build.sh $THIRDPARTY_ARGS
    cd ..
    
    mkdir -p "$LIB_DIR"
    case "$TARGET" in
        arm|arm64)
            cp thirdparty/zlib/libz.so "$LIB_DIR/" 2>/dev/null || true
            cp thirdparty/bzip2/libbz2.so "$LIB_DIR/" 2>/dev/null || true
            cp thirdparty/libpng/.libs/libpng16.so "$LIB_DIR/" 2>/dev/null || true
            cp thirdparty/libjpeg/.libs/libjpeg.so "$LIB_DIR/" 2>/dev/null || true
            cp thirdparty/openjpeg/build/bin/libopenjp2.so "$LIB_DIR/" 2>/dev/null || true
            cp thirdparty/jbig2dec/.libs/libjbig2dec.so "$LIB_DIR/" 2>/dev/null || true
            cp thirdparty/freetype2/objs/.libs/libfreetype.so "$LIB_DIR/" 2>/dev/null || true
            cp thirdparty/harfbuzz/build/src/libharfbuzz.so "$LIB_DIR/" 2>/dev/null || true
            cp thirdparty/gumbo/.libs/libgumbo.so "$LIB_DIR/" 2>/dev/null || true
            cp thirdparty/djvulibre/libdjvu/.libs/libdjvulibre.so "$LIB_DIR/" 2>/dev/null || true
            cp thirdparty/mupdf/build/release/libmupdf.so "$LIB_DIR/" 2>/dev/null || true
            ;;
        host)
            find thirdparty -name "*.so*" -type f -exec cp {} "$LIB_DIR/" \; 2>/dev/null || true
            ;;
    esac
    ensure_symlinks "$LIB_DIR"
elif [ "$METHOD" = "skip" ]; then
    echo "Skipping thirdparty library build/download"
    ensure_symlinks "$LIB_DIR"
fi

# Build mupdf_wrapper
echo "Building mupdf_wrapper for $TARGET target..."
cd mupdf_wrapper
case "$TARGET" in
    arm|arm64)
        ./build-kobo.sh
        ;;
    host)
        ./build.sh
        ;;
esac
cd ..

# Build all crates in the workspace
echo "Building Plato workspace crates..."
CARGO_TARGET_FLAGS=$(get_cargo_target_flags "$TARGET")
CARGO_PROFILE=$(get_cargo_profile "$TARGET")

# Crate list - conditionally include emulator
if [ "$TARGET" = "host" ]; then
    CRATES="plato-core plato epub_editor emulator importer fetcher"
else
    CRATES="plato-core plato epub_editor importer fetcher"
fi

for crate in $CRATES; do
    if [ -d "crates/$crate" ]; then
        echo "Building $crate..."
        if [ -n "$CARGO_TARGET_FLAGS" ]; then
            cargo build $CARGO_TARGET_FLAGS $CARGO_PROFILE -p $crate
        else
            cargo build $CARGO_PROFILE -p $crate
        fi
    fi
done

echo "Build completed successfully for $TARGET target!"
