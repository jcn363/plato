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
echo "Building djvulibre for iOS device (ARM64)..."
BUILD_DIR=../target/djvulibre/iOS/arm64
mkdir -p $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0"
./configure --static --prefix=$BUILD_DIR --host=arm-apple-darwin
make clean || true
make -j$(sysctl -n hw.ncpu)
make install

# Build for iOS simulator (ARM64)
echo "Building djvulibre for iOS simulator (ARM64)..."
BUILD_DIR=../target/djvulibre/iOS-sim/arm64
mkdir -p $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
./configure --static --prefix=$BUILD_DIR --host=arm-apple-darwin
make clean || true
make -j$(sysctl -n hw.ncpu)
make install

# Build for iOS simulator (x86_64)
echo "Building djvulibre for iOS simulator (x86_64)..."
BUILD_DIR=../target/djvulibre/iOS-sim/x86_64
mkdir -p $BUILD_DIR
export CFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
./configure --static --prefix=$BUILD_DIR --host=x86_64-apple-darwin
make clean || true
make -j$(sysctl -n hw.ncpu)
make install

# Create universal library (device + simulator)
echo "Creating universal djvulibre library..."
mkdir -p ../target/djvulibre/iOS-device/lib
mkdir -p ../target/djvulibre/iOS-simulator/lib
lipo -create \
  ../target/djvulibre/iOS-sim/arm64/lib/libdjvulibre.a \
  ../target/djvulibre/iOS-sim/x86_64/lib/libdjvulibre.a \
  -output ../target/djvulibre/iOS-simulator/lib/libdjvulibre.a || echo "Some simulator architectures may have failed"
cp ../target/djvulibre/iOS/arm64/lib/libdjvulibre.a ../target/djvulibre/iOS-device/lib/

echo "djvulibre built successfully for iOS."
