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
echo "Building harfbuzz for iOS device (ARM64)..."
BUILD_DIR=../target/harfbuzz/iOS/arm64
mkdir -p $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0"
export PKG_CONFIG_PATH=""
./configure --static --prefix=$BUILD_DIR --host=arm-apple-darwin --with-freetype=yes --with-fontconfig=no
make clean || true
make -j$(sysctl -n hw.ncpu)

# Build for iOS simulator (ARM64)
echo "Building harfbuzz for iOS simulator (ARM64)..."
BUILD_DIR=../target/harfbuzz/iOS-sim/arm64
mkdir -p $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
export PKG_CONFIG_PATH=""
./configure --static --prefix=$BUILD_DIR --host=arm-apple-darwin --with-freetype=yes --with-fontconfig=no
make clean || true
make -j$(sysctl -n hw.ncpu)

# Build for iOS simulator (x86_64)
echo "Building harfbuzz for iOS simulator (x86_64)..."
BUILD_DIR=../target/harfbuzz/iOS-sim/x86_64
mkdir -p $BUILD_DIR
export CFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
export PKG_CONFIG_PATH=""
./configure --static --prefix=$BUILD_DIR --host=x86_64-apple-darwin --with-freetype=yes --with-fontconfig=no
make clean || true
make -j$(sysctl -n hw.ncpu)

# Create universal library (device + simulator)
echo "Creating universal harfbuzz library..."
mkdir -p ../target/harfbuzz/iOS-device/lib
mkdir -p ../target/harfbuzz/iOS-simulator/lib
lipo -create \
  ../target/harfbuzz/iOS-sim/arm64/lib/libharfbuzz.a \
  ../target/harfbuzz/iOS-sim/x86_64/lib/libharfbuzz.a \
  -output ../target/harfbuzz/iOS-simulator/lib/libharfbuzz.a || echo "Some simulator architectures may have failed"
cp ../target/harfbuzz/iOS/arm64/lib/libharfbuzz.a ../target/harfbuzz/iOS-device/lib/

echo "harfbuzz built successfully for iOS."
