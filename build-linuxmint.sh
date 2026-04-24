#!/bin/sh
#
# Build script for LinuxMint (desktop Linux with 8-16GB RAM)
#
# LinuxMint is a desktop Linux distribution with abundant resources:
# - Typical hardware: 4-8 core CPU, 8-16GB RAM
# - SSD storage (fast I/O)
# - Desktop environment (X-Cinnamon, MATE, Xfce)
# - No power constraints (unlike mobile/e-ink)
#
# This script builds with LinuxMint-specific configuration:
# - Target: x86_64-unknown-linux-gnu (host native)
# - Device: linuxmint (via PLATO_DEVICE environment variable for conditional compilation)
# - Clean build with zero warnings (cargo clean + clippy -D warnings)
# - Desktop-optimized cache and buffer sizes

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_NAME="plato"
TARGET="x86_64-unknown-linux-gnu"
PROFILE="release"

echo "=========================================="
echo "Building Plato for LinuxMint"
echo "Target: $TARGET (x86_64 host native)"
echo "Profile: $PROFILE"
echo "=========================================="
echo ""

cd "$SCRIPT_DIR"

# Verify we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Run from project root."
    exit 1
fi

# Set environment to indicate LinuxMint build (used by optimizations)
export PLATO_DEVICE="linuxmint"

# Number of parallel jobs (use all available cores on desktop)
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
echo "Step 4: Building Plato for LinuxMint..."
echo "Target: $TARGET (x86_64 host native)"
echo "Profile: $PROFILE"
echo "Desktop optimizations active:"
echo "  - Page cache: 200MB"
echo "  - Thumbnail buffer: 8MB"
echo "  - Document buffer: 32MB"
echo "  - Thumbnail workers: 6"
echo "  - Thumbnail cache: 100 entries"
echo "  - Preload ahead: 5 pages"
echo "  - Preload behind: 3 pages"
echo ""

cargo build -p plato --target "$TARGET" --profile "$PROFILE" -j "$JOBS"

echo ""
echo "Step 5: Creating Debian package..."
echo ""

# Check if dpkg-buildpackage is available
if ! command -v dpkg-buildpackage &> /dev/null; then
    echo "Warning: dpkg-buildpackage not found. Skipping .deb package creation."
    echo "To build .deb packages, install dpkg-dev:"
    echo "  sudo apt-get install dpkg-dev debhelper"
    echo ""
else
    # Build the debian package
    echo "Building Debian package..."
    dpkg-buildpackage -us -uc -b

    # Create dist directory and move .deb files there
    mkdir -p dist
    mv ../plato_*.deb dist/ 2>/dev/null || true
    mv ../plato-dbgsym_*.ddeb dist/ 2>/dev/null || true

    echo "Debian package created in dist/"
fi

echo ""
echo "=========================================="
echo "Build complete for LinuxMint!"
echo "=========================================="
echo ""
echo "Binary location:"
echo "  target/$TARGET/$PROFILE/plato"
echo ""
if [ -d "dist" ] && ls dist/*.deb 1> /dev/null 2>&1; then
    echo "Debian package location:"
    ls -lh dist/*.deb
    echo ""
fi
echo "To run Plato on LinuxMint:"
echo "  ./target/$TARGET/$PROFILE/plato"
echo ""
echo "Or install from .deb package:"
echo "  sudo dpkg -i dist/plato_*.deb"
echo ""
echo "Desktop optimizations active:"
echo "  ✓ 200MB page cache (abundant RAM)"
echo "  ✓ 8MB thumbnail buffers"
echo "  ✓ 32MB document buffers"
echo "  ✓ 6 thumbnail workers (multi-core CPU)"
echo "  ✓ 100 cache entries"
echo "  ✓ 5 pages preload ahead"
echo "  ✓ 3 pages preload behind"
echo "  ✓ XDG config/data directory support"
echo "  ✓ System font integration"
echo ""
