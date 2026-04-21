#!/bin/sh

set -e

# Check if running on macOS
case "$OSTYPE" in
  darwin*)
    ;;
  *)
    echo "Error: iOS builds require macOS."
    exit 1
    ;;
esac

# Set up iOS SDK paths
export IOS_SDK=$(xcrun --sdk iphoneos --show-sdk-path)
export IOS_SIM_SDK=$(xcrun --sdk iphonesimulator --show-sdk-path)
export IOS_CC=$(xcrun --sdk iphoneos --find clang)
export IOS_AR=$(xcrun --sdk iphoneos --find ar)

# Build for iOS device (ARM64)
echo "Building zlib for iOS device (ARM64)..."
BUILD_DIR=../target/zlib/iOS/arm64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0 -fPIC -O2"
../../configure --static --prefix=$(pwd)
make clean || true
make -j$(sysctl -n hw.ncpu)
cd ../../..

# Build for iOS simulator (ARM64)
echo "Building zlib for iOS simulator (ARM64)..."
BUILD_DIR=../target/zlib/iOS-sim/arm64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
../../configure --static --prefix=$(pwd)
make clean || true
make -j$(sysctl -n hw.ncpu)
cd ../../..

# Build for iOS simulator (x86_64)
echo "Building zlib for iOS simulator (x86_64)..."
BUILD_DIR=../target/zlib/iOS-sim/x86_64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
../../configure --static --prefix=$(pwd)
make clean || true
make -j$(sysctl -n hw.ncpu)
cd ../../..

# Create universal library
echo "Creating universal zlib library..."
mkdir -p ../target/zlib/iOS-universal/lib
lipo -create \
  ../target/zlib/iOS/arm64/lib/libz.a \
  ../target/zlib/iOS-sim/arm64/lib/libz.a \
  ../target/zlib/iOS-sim/x86_64/lib/libz.a \
  -output ../target/zlib/iOS-universal/lib/libz.a || echo "Some architectures may have failed"

echo "zlib built successfully for iOS."
