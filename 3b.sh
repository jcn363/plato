#!/bin/bash
set -e

# Optimized parallel build script for LinuxMint, Elipsa, and OnePlus Nord 2
# Usage: ./3b.sh [--clean]
# --clean: Force clean build (removes all build artifacts)

CLEAN=false
for arg in "$@"; do
  case $arg in
    --clean) CLEAN=true ;;
    *)
      echo "Unknown argument: $arg"
      echo "Usage: $0 [--clean]"
      exit 1
      ;;
  esac
done

echo "=========================================="
echo "Optimized Parallel Build for All Targets"
echo "=========================================="

# Step 1: Format code (once)
echo "Step 1: Formatting code..."
cargo fmt

# Step 2: Run clippy (once, for host target catches most issues)
echo "Step 2: Running clippy (workspace-wide, warnings as errors)..."
cargo clippy -- -D warnings

# Step 3: Clean if requested
if [ "$CLEAN" = true ]; then
  echo "Step 3: Cleaning previous builds..."
  cargo clean
else
  echo "Step 3: Skipping clean (incremental build)"
fi

# Step 4: Build all three targets in parallel
echo "Step 4: Building all targets in parallel..."
echo ""

# LinuxMint (x86_64)
echo "  [1/3] Starting LinuxMint build (x86_64)..."
PLATO_DEVICE=linuxmint cargo build --release --package plato --target x86_64-unknown-linux-gnu &
PID_LINUXMINT=$!

# Elipsa (armv7)
echo "  [2/3] Starting Elipsa build (armv7)..."
PLATO_DEVICE=elipsa RUSTFLAGS="-C target-cpu=cortex-a7" cargo build --profile release-arm --package plato --target arm-unknown-linux-gnueabihf &
PID_ELIPSA=$!

# OnePlus (Android ARM64)
echo "  [3/3] Starting OnePlus build (Android ARM64)..."
(cd crates/plato-android && ANDROID_ROOT="/system" PLATO_DEVICE="android" cargo apk build) &
PID_ONEPLUS=$!

echo ""
echo "Waiting for all builds to complete..."
echo ""

# Wait for all builds and capture exit status
wait $PID_LINUXMINT || LINUXMINT_STATUS=$?
if [ -z "$LINUXMINT_STATUS" ]; then
  echo "  ✓ LinuxMint build complete"
else
  echo "  ✗ LinuxMint build failed (exit code: $LINUXMINT_STATUS)"
fi

wait $PID_ELIPSA || ELIPSA_STATUS=$?
if [ -z "$ELIPSA_STATUS" ]; then
  echo "  ✓ Elipsa build complete"
else
  echo "  ✗ Elipsa build failed (exit code: $ELIPSA_STATUS)"
fi

wait $PID_ONEPLUS || ONEPLUS_STATUS=$?
if [ -z "$ONEPLUS_STATUS" ]; then
  echo "  ✓ OnePlus build complete"
else
  echo "  ✗ OnePlus build failed (exit code: $ONEPLUS_STATUS)"
fi

# Step 5: Create distribution packages
echo ""
echo "Step 5: Creating distribution packages..."
mkdir -p dist

# LinuxMint: Create Debian package
echo "  [1/3] Creating LinuxMint Debian package..."
if which dpkg-buildpackage > /dev/null 2>&1; then
  rm -rf debian/.debhelper
  if dpkg-buildpackage -us -uc -b -d; then
    mv ../plato_*.deb dist/ 2>/dev/null || true
    mv ../plato-dbgsym_*.ddeb dist/ 2>/dev/null || true
    echo "  ✓ LinuxMint Debian package created"
  else
    echo "  ✗ LinuxMint Debian package failed (copying binary instead)"
    cp target/x86_64-unknown-linux-gnu/release/plato dist/plato-linuxmint
  fi
else
  echo "  ! dpkg-buildpackage not found, copying binary instead"
  cp target/x86_64-unknown-linux-gnu/release/plato dist/plato-linuxmint
fi

# Elipsa: Create bundle
echo "  [2/3] Creating Elipsa bundle..."
# Rebuild Elipsa if it was cleaned by dpkg-buildpackage
if [ ! -f "target/arm-unknown-linux-gnueabihf/release-arm/plato" ]; then
  echo "  ! Elipsa binary not found (cleaned by dpkg-buildpackage), rebuilding..."
  PLATO_DEVICE=elipsa RUSTFLAGS="-C target-cpu=cortex-a7" cargo build --profile release-arm --package plato --target arm-unknown-linux-gnueabihf
fi
cp target/arm-unknown-linux-gnueabihf/release-arm/plato dist/plato
if [ -f "./bundle.sh" ]; then
  ./bundle.sh
  echo "  ✓ Elipsa bundle created"
else
  echo "  ! bundle.sh not found, copying binary only"
fi

# OnePlus: Copy APK
echo "  [3/3] Copying OnePlus APK..."
cp crates/plato-android/target/debug/apk/plato-android.apk dist/plato-oneplus-nord2.apk
echo "  ✓ OnePlus APK copied"

echo ""
echo "=========================================="
echo "Optimized Build Complete!"
echo "=========================================="
echo ""
echo "Artifacts in dist/:"
ls -lh dist/
