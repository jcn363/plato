#!/bin/sh

set -e

case "$OSTYPE" in
  darwin*) ;;
  *) echo "Error: iOS builds require macOS."; exit 1 ;;
esac

export IOS_SDK=$(xcrun --sdk iphoneos --show-sdk-path)
export IOS_SIM_SDK=$(xcrun --sdk iphonesimulator --show-sdk-path)
export IOS_CC=$(xcrun --sdk iphoneos --find clang)
export IOS_AR=$(xcrun --sdk iphoneos --find ar)

# Build for iOS device (ARM64)
echo "Building mupdf for iOS device (ARM64)..."
BUILD_DIR=../target/mupdf/iOS/arm64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0"
make -C ../../../mupdf clean || true
make -C ../../../mupdf -j$(sysctl -n hw.ncpu) build=release XCFLAGS="$CFLAGS" LDFLAGS="$LDFLAGS"
cd ../../..

# Build for iOS simulator (ARM64)
echo "Building mupdf for iOS simulator (ARM64)..."
BUILD_DIR=../target/mupdf/iOS-sim/arm64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
make -C ../../ clean || true
make -C ../../ -j$(sysctl -n hw.ncpu) build=release XCFLAGS="$CFLAGS" LDFLAGS="$LDFLAGS"
cd ../../..

# Build for iOS simulator (x86_64)
echo "Building mupdf for iOS simulator (x86_64)..."
BUILD_DIR=../target/mupdf/iOS-sim/x86_64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
make -C ../../ clean || true
make -C ../../ -j$(sysctl -n hw.ncpu) build=release XCFLAGS="$CFLAGS" LDFLAGS="$LDFLAGS"
cd ../../..

# Create universal library
echo "Creating universal mupdf library..."
mkdir -p ../target/mupdf/iOS-universal/lib
lipo -create \
  ../target/mupdf/iOS/arm64/build/release/libmupdf.a \
  ../target/mupdf/iOS-sim/arm64/build/release/libmupdf.a \
  ../target/mupdf/iOS-sim/x86_64/build/release/libmupdf.a \
  -output ../target/mupdf/iOS-universal/lib/libmupdf.a || echo "Some architectures may have failed"

# Also create mupdf-third universal library
lipo -create \
  ../target/mupdf/iOS/arm64/build/release/libmupdf-third.a \
  ../target/mupdf/iOS-sim/arm64/build/release/libmupdf-third.a \
  ../target/mupdf/iOS-sim/x86_64/build/release/libmupdf-third.a \
  -output ../target/mupdf/iOS-universal/lib/libmupdf-third.a || echo "Some architectures may have failed"

echo "mupdf built successfully for iOS."
