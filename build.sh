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
JOBS=$(nproc 2>/dev/null || echo 4)
THIRDPARTY_ARGS=""

print_usage() {
    echo "Usage: $0 [OPTIONS] [TARGET] [METHOD] [THIRDPARTY_ARGS...]"
    echo ""
    echo "OPTIONS:"
    echo "  --no-clean       Skip cargo clean"
    echo "  --no-clippy      Skip cargo clippy"
    echo "  --no-fmt         Skip cargo fmt"
    echo "  -j JOBS          Number of parallel jobs (default: $JOBS)"
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
        -j) shift; JOBS="$1"; shift ;;
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
# This is the canonical target-to-directory mapping and the single source of truth for it in shell scripts:
# - arm (arm-unknown-linux-gnueabihf) → libs/ (ARM 32-bit for original Kobo devices)
# - arm64 (aarch64-unknown-linux-gnu) → libs64/ (ARM 64-bit for newer Kobo devices)
# - host (x86_64-unknown-linux-gnu) → libs_host/ (host/x86_64 for development/emulator)
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

check_lib_exists() {
    local lib="$1"
    if [ ! -f "$LIB_DIR/$lib" ]; then
        return 1
    fi
    return 0
}

THIRDPARTY_NEED_REBUILD=0

if [ "$METHOD" = "fast" ] || [ "$METHOD" = "slow" ] || [ "$METHOD" = "skip" ]; then
    mkdir -p "$LIB_DIR"
    
    NEED_REBUILD=0
    for lib in libmupdf.so libfreetype.so libharfbuzz.so libpng16.so libjpeg.so libopenjp2.so libz.so libbz2.so libjbig2dec.so libdjvulibre.so libgumbo.so; do
        if ! check_lib_exists "$lib"; then
            NEED_REBUILD=1
            echo "Library $lib not found in $LIB_DIR"
            break
        fi
    done
    
    if [ "$METHOD" = "fast" ]; then
        if [ ! -d "$LIB_DIR" ] || [ "$NEED_REBUILD" = "1" ]; then
            echo "Using fast method - downloading prebuilt libraries"
            ./download.sh "$LIB_DIR/*"
        fi
        ensure_symlinks "$LIB_DIR"
    elif [ "$METHOD" = "slow" ]; then
        if [ "$NEED_REBUILD" = "1" ]; then
            echo "Building thirdparty libraries from source with $JOBS jobs"
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
        else
            echo "Thirdparty libraries up to date, skipping rebuild."
        fi
    elif [ "$METHOD" = "skip" ]; then
        if [ "$NEED_REBUILD" = "1" ]; then
            echo "Warning: Required library missing, using fast method to download..."
            ./download.sh "$LIB_DIR/*"
        else
            echo "Skipping thirdparty library build/download"
        fi
        ensure_symlinks "$LIB_DIR"
    fi
fi

# Build mupdf_wrapper if needed
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MUPDF_WRAPPER_DIR="$SCRIPT_DIR/target/mupdf_wrapper/Kobo"
MUPDF_WRAPPER_LIB="$MUPDF_WRAPPER_DIR/libmupdf_wrapper.a"
MUPDF_WRAPPER_C="$SCRIPT_DIR/mupdf_wrapper/mupdf_wrapper.c"

echo "Checking mupdf_wrapper..."
cd "$SCRIPT_DIR/mupdf_wrapper"
NEED_BUILD=0
if [ ! -f "$MUPDF_WRAPPER_LIB" ]; then
    NEED_BUILD=1
    echo "mupdf_wrapper not found, building..."
elif [ "$MUPDF_WRAPPER_C" -nt "$MUPDF_WRAPPER_LIB" ]; then
    NEED_BUILD=1
    echo "mupdf_wrapper.c modified, rebuilding..."
fi

if [ "$NEED_BUILD" = "1" ]; then
    echo "Building mupdf_wrapper for $TARGET target..."
    case "$TARGET" in
        arm)
            TARGET_OS=Kobo CC=arm-linux-gnueabihf-gcc AR=arm-linux-gnueabihf-ar ./build.sh
            ;;
        arm64)
            TARGET_OS=Kobo CC=aarch64-linux-gnu-gcc AR=aarch64-linux-gnu-ar ./build.sh
            ;;
        host)
            ./build.sh
            ;;
    esac
else
    echo "mupdf_wrapper up to date, skipping build."
fi
cd "$SCRIPT_DIR"

# Build all crates in the workspace
echo "Building Plato workspace crates..."
CARGO_TARGET_FLAGS=$(get_cargo_target_flags "$TARGET")
CARGO_PROFILE=$(get_cargo_profile "$TARGET")

if [ "$TARGET" = "host" ]; then
    echo "Building all crates for host..."
    cargo build $CARGO_PROFILE --workspace
else
    echo "Building crates for $TARGET (excluding emulator)..."
    # Filter out emulator for ARM targets
    # We can't use --exclude easily with cargo build when we want to be explicit about what's built
    # but since we're in a workspace, we can just build the workspace and exclude it.
    if [ -n "$CARGO_TARGET_FLAGS" ]; then
        cargo build $CARGO_TARGET_FLAGS $CARGO_PROFILE --workspace --exclude emulator
    else
        cargo build $CARGO_PROFILE --workspace --exclude emulator
    fi
fi

echo "Build completed successfully for $TARGET target!"
