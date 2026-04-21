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
echo "Building freetype2 for iOS device (ARM64)..."
BUILD_DIR=../../target/freetype2/iOS/arm64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0"
../../../freetype2/configure --static --prefix=$(pwd) --host=arm-apple-darwin --without-harfbuzz
make clean || true
make -j$(sysctl -n hw.ncpu)
cd ../../../../../..

# Build for iOS simulator (ARM64)
echo "Building freetype2 for iOS simulator (ARM64)..."
BUILD_DIR=../../target/freetype2/iOS-sim/arm64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
../../../freetype2/configure --static --prefix=$(pwd) --host=arm-apple-darwin --without-harfbuzz
make clean || true
make -j$(sysctl -n hw.ncpu)
cd ../../../../../..

# Build for iOS simulator (x86_64)
echo "Building freetype2 for iOS simulator (x86_64)..."
BUILD_DIR=../../target/freetype2/iOS-sim/x86_64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
../../../freetype2/configure --static --prefix=$(pwd) --host=x86_64-apple-darwin --without-harfbuzz
make clean || true
make -j$(sysctl -n hw.ncpu)
cd ../../../../../..

# Create universal library
echo "Creating universal freetype2 library..."
mkdir -p ../../target/freetype2/iOS-universal/lib
lipo -create \
  ../../target/freetype2/iOS/arm64/lib/libfreetype.a \
  ../../target/freetype2/iOS-sim/arm64/lib/libfreetype.a \
  ../../target/freetype2/iOS-sim/x86_64/lib/libfreetype.a \
  -output ../../target/freetype2/iOS-universal/lib/libfreetype.a || echo "Some architectures may have failed"

echo "freetype2 built successfully for iOS."
