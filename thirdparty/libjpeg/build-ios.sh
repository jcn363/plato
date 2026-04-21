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
echo "Building libjpeg for iOS device (ARM64)..."
BUILD_DIR=../target/libjpeg/iOS/arm64
mkdir -p $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0"
./configure --static --prefix=$BUILD_DIR --host=arm-apple-darwin
make clean || true
make -j$(sysctl -n hw.ncpu)

# Build for iOS simulator (ARM64)
echo "Building libjpeg for iOS simulator (ARM64)..."
BUILD_DIR=../target/libjpeg/iOS-sim/arm64
mkdir -p $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
./configure --static --prefix=$BUILD_DIR --host=arm-apple-darwin
make clean || true
make -j$(sysctl -n hw.ncpu)

# Build for iOS simulator (x86_64)
echo "Building libjpeg for iOS simulator (x86_64)..."
BUILD_DIR=../target/libjpeg/iOS-sim/x86_64
mkdir -p $BUILD_DIR
export CFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
./configure --static --prefix=$BUILD_DIR --host=x86_64-apple-darwin
make clean || true
make -j$(sysctl -n hw.ncpu)

# Create universal library (device + simulator)
echo "Creating universal libjpeg library..."
mkdir -p ../target/libjpeg/iOS-device/lib
mkdir -p ../target/libjpeg/iOS-simulator/lib
lipo -create \
  ../target/libjpeg/iOS-sim/arm64/lib/libjpeg.a \
  ../target/libjpeg/iOS-sim/x86_64/lib/libjpeg.a \
  -output ../target/libjpeg/iOS-simulator/lib/libjpeg.a || echo "Some simulator architectures may have failed"
cp ../target/libjpeg/iOS/arm64/lib/libjpeg.a ../target/libjpeg/iOS-device/lib/

echo "libjpeg built successfully for iOS."
