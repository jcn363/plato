#!/bin/sh
#
# Build script for Kobo Elipsa devices (32-bit ARM with 1GB RAM)
#
# The Elipsa uses the Allwinner B300 (ARMv7, 4-core) with:
# - 1GB LPDDR4 RAM (vs 256-512MB on standard Kobo)
# - Stylus support (Wacom)
# - Gyroscope (automatic rotation)
#
# This script builds with device-specific optimizations:
# - 2MB thumbnail buffers (vs 1MB standard)
# - 8MB document buffers (vs 4MB standard)
# - 3 thumbnail workers (vs 2 standard)
# - 35 thumbnail cache entries (vs 20 standard)
# - 40MB page cache (vs 20MB standard)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_NAME="plato"
TARGET="arm-unknown-linux-gnueabihf"
PROFILE="release-arm"

echo "=========================================="
echo "Building Plato for Kobo Elipsa"
echo "Target: $TARGET (32-bit ARM)"
echo "Profile: $PROFILE"
echo "=========================================="
echo ""

cd "$SCRIPT_DIR"

# Verify we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Run from project root."
    exit 1
fi

# Set environment to indicate Elipsa build (used by optimizations)
export PLATO_DEVICE="elipsa"

# Number of parallel jobs
JOBS=$(nproc 2>/dev/null || echo 4)

echo "Step 1: Formatting code..."
cargo fmt --all

echo ""
echo "Step 2: Running clippy..."
cargo clippy -p plato-core --target "$TARGET" -- -D warnings
cargo clippy -p plato --target "$TARGET" -- -D warnings

echo ""
echo "Step 3: Building third-party libraries..."
if [ -x "./thirdparty/build.sh" ]; then
    ./thirdparty/build.sh "$TARGET" fast
else
    echo "Warning: thirdparty/build.sh not found, skipping..."
fi

echo ""
echo "Step 4: Building Plato for Elipsa..."
echo "This will use device-optimized settings:"
echo "  - Thumbnail buffer: 2MB"
echo "  - Document buffer: 8MB"
echo "  - Workers: 3 threads"
echo "  - Cache: 35 entries"
echo "  - Page cache: 40MB"
echo ""

RUSTFLAGS="-C target-cpu=cortex-a7 -C target-feature=+vfpv4,+neon" \
    cargo build -p plato --target "$TARGET" --profile "$PROFILE" -j "$JOBS"

echo ""
echo "=========================================="
echo "Build complete for Kobo Elipsa!"
echo "=========================================="
echo ""
echo "Binary location:"
echo "  target-arm/$TARGET/$PROFILE/plato"
echo ""
echo "To deploy to Elipsa:"
echo "  1. Connect via USB"
echo "  2. Copy binary to /mnt/onboard/.adds/plato/"
echo "  3. Run: ./plato.sh"
echo ""
echo "Device optimizations active:"
echo "  ✓ 2MB thumbnail buffers"
echo "  ✓ 8MB document buffers"
echo "  ✓ 3 thumbnail workers"
echo "  ✓ 35 cache entries"
echo "  ✓ Stylus support enabled"
echo "  ✓ Gyroscope rotation enabled"
echo ""
