#!/bin/sh
#
# Build script for Kobo Elipsa devices (32-bit ARM with 1GB RAM)
#
# The Elipsa uses the Allwinner B300 (ARMv7, 4-core) with:
# - 1GB LPDDR4 RAM (vs 256-512MB on standard Kobo)
# - Stylus support (Wacom)
# - Gyroscope (automatic rotation)
#
# This script builds with Elipsa-specific configuration:
# - Target CPU: cortex-a7 (via RUSTFLAGS)
# - Device: elipsa (via PLATO_DEVICE environment variable for conditional compilation)
# - Clean build with zero warnings (cargo clean + clippy -D warnings)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_NAME="plato"
TARGET="arm-unknown-linux-gnueabihf"
PROFILE="release-arm"
NICKEL_MENU_ARCHIVE="${1:-}"

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

echo "Step 1: Cleaning previous builds..."
cargo clean

echo ""
echo "Step 2: Formatting code..."
cargo fmt --all

echo ""
echo "Step 3: Running clippy (workspace-wide, warnings as errors)..."
cargo clippy --target "$TARGET" --workspace -- -D warnings

echo ""
echo "Step 4: Building Plato for Elipsa..."
echo "Target: $TARGET (32-bit ARM)"
echo "Profile: $PROFILE"
echo "Target CPU: cortex-a7"
echo ""

RUSTFLAGS="-C target-cpu=cortex-a7" \
    cargo build -p plato --target "$TARGET" --profile "$PROFILE" -j "$JOBS"

echo ""
echo "Step 5: Creating distribution bundle..."
DIST_DIR="dist"
[ -d "$DIST_DIR" ] && rm -Rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Copy built binary (no external libraries needed - pure Rust)
cp "target/$TARGET/$PROFILE/plato" "$DIST_DIR/"

# Create bundle (simple or full with NickelMenu depending on archive)
echo ""
echo "Step 6: Creating bundle..."
./bundle.sh "$NICKEL_MENU_ARCHIVE"

echo ""
echo "=========================================="
echo "Build complete for Kobo Elipsa!"
echo "=========================================="
echo ""
echo "Distribution location:"
echo "  $DIST_DIR/"
echo ""
echo "Binary location:"
echo "  $DIST_DIR/plato"
echo ""
echo "Bundle location:"
echo "  $DIST_DIR/plato-bundle-*.zip"
echo ""
echo "To deploy to Elipsa:"
echo "  1. Connect via USB"
echo "  2. Extract plato-bundle-*.zip to /mnt/onboard/.adds/plato/"
echo "  3. Run: ./plato.sh"
echo ""
echo "Build configuration:"
echo "  ✓ Target CPU: cortex-a7"
echo "  ✓ Device: Elipsa (PLATO_DEVICE=elipsa)"
echo "  ✓ Zero warnings (clippy -D warnings)"
echo "  ✓ Clean build (cargo clean)"
echo "  ✓ Bundle created"
if [ -n "$NICKEL_MENU_ARCHIVE" ]; then
    echo "  ✓ NickelMenu integration enabled"
fi
echo ""
