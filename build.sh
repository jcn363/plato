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
        cargo clippy $CARGO_TARGET_FLAGS --workspace -- -D warnings
    else
        cargo clippy --workspace -- -D warnings
    fi
fi

# Third-party C libraries removed - project migrated to pure Rust
# - bzip2, html5ever, openjp2, hayro-jbig2, djvu-rs (Rust equivalents)
# - skrifa, rustybuzz, ab_glyph (Rust font stack, replaces FreeType + HarfBuzz)
# - pdfpurr (Pure Rust PDF, replaces MuPDF)

# Build all crates in the workspace
echo "Building Plato workspace crates..."
CARGO_TARGET_FLAGS=$(get_cargo_target_flags "$TARGET")
CARGO_PROFILE=$(get_cargo_profile "$TARGET")

if [ "$TARGET" = "host" ]; then
    echo "Building all crates for host..."
    cargo build $CARGO_PROFILE --workspace
else
    echo "Building crates for $TARGET..."
    if [ -n "$CARGO_TARGET_FLAGS" ]; then
        cargo build $CARGO_TARGET_FLAGS $CARGO_PROFILE --workspace
    else
        cargo build $CARGO_PROFILE --workspace
    fi
fi

echo "Build completed successfully for $TARGET target!"
