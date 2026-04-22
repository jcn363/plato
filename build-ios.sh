#!/usr/bin/env bash

set -e

TARGET=${1:-universal}

echo "Building Plato for iOS..."

case $TARGET in
  device)
    echo "Building for iOS device (ARM64)..."
    IOS_TARGETS=("aarch64-apple-ios")
    ;;
  simulator)
    echo "Building for iOS simulator (ARM64 and x86_64)..."
    IOS_TARGETS=("aarch64-apple-ios-sim" "x86_64-apple-ios")
    ;;
  universal)
    echo "Building universal iOS library (device + simulator)..."
    IOS_TARGETS=("aarch64-apple-ios" "aarch64-apple-ios-sim" "x86_64-apple-ios")
    ;;
  *)
    echo "Usage: $0 [device|simulator|universal]"
    echo "  device     - Build for iOS device only (ARM64)"
    echo "  simulator  - Build for iOS simulator only (ARM64 + x86_64)"
    echo "  universal  - Build for both (default)"
    exit 1
    ;;
esac

# Check if we're on macOS for final steps
if [[ "$OSTYPE" == "darwin"* ]]; then
  ON_MACOS=true
  echo "Running on macOS - will perform full build including linking"
else
  ON_MACOS=false
  echo "Running on Linux - will cross-compile only (final linking requires macOS)"
fi

# Build native libraries for iOS
echo "Building native libraries for iOS..."
cd thirdparty

# Check if build-ios.sh exists for each library
LIBS=()

for lib in "${LIBS[@]}"; do
  if [ -f "$lib/build-ios.sh" ]; then
    echo "Building $lib for iOS..."
    cd "$lib"
    ./build-ios.sh
    cd ..
  else
    echo "Warning: $lib/build-ios.sh not found, skipping..."
  fi
done

cd ..

# mupdf and mupdf_wrapper removed - MuPDF replaced by PDFPurr (pure Rust)

echo "Native library build step completed."

# Build Rust core library for iOS targets
echo "Building Rust core library for iOS targets..."
for target in "${IOS_TARGETS[@]}"; do
  echo "Building for target: $target"
  cargo build --target "$target" --release -p plato-core || {
    echo "Warning: Failed to build for $target (may require macOS for some targets)"
  }
done

# Create universal library if on macOS
if [ "$ON_MACOS" = true ] && [ "$TARGET" = "universal" ]; then
  echo "Creating universal library with lipo..."
  
  UNIVERSAL_DIR="target/universal"
  mkdir -p "$UNIVERSAL_DIR"
  
  # Create universal static library
  lipo -create \
    target/aarch64-apple-ios/release/libplato_core.a \
    target/aarch64-apple-ios-sim/release/libplato_core.a \
    target/x86_64-apple-ios/release/libplato_core.a \
    -output "$UNIVERSAL_DIR/libplato_core.a" || {
    echo "Warning: Failed to create universal library (some targets may not have been built)"
  }
  
  echo "Universal library created at: $UNIVERSAL_DIR/libplato_core.a"
fi

echo "iOS build process completed!"
echo ""
echo "Next steps:"
if [ "$ON_MACOS" = true ]; then
  echo "1. Create Xcode project to link the static library"
  echo "2. Add the built .a files to your Xcode project"
  echo "3. Build and sign the iOS app with Xcode or xcodebuild"
else
  echo "1. Push changes to trigger macOS CI workflow"
  echo "2. The CI will complete the build on macOS"
  echo "3. Download the built IPA from GitHub Actions artifacts"
  echo ""
  echo "Or manually:"
  echo "1. Copy the built artifacts to a macOS machine"
  echo "2. Run this script on macOS to complete the build"
  echo "3. Use Xcode to create the final iOS app"
fi
