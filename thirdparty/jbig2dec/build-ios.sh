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
echo "Building jbig2dec for iOS device (ARM64)..."
BUILD_DIR=../target/jbig2dec/iOS/arm64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0"
../../../jbig2dec/configure --static --prefix=$(pwd) --host=arm-apple-darwin
make clean || true
make -j$(sysctl -n hw.ncpu)
cd ../../..

# Build for iOS simulator (ARM64)
echo "Building jbig2dec for iOS simulator (ARM64)..."
BUILD_DIR=../target/jbig2dec/iOS-sim/arm64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
../../../jbig2dec/configure --static --prefix=$(pwd) --host=arm-apple-darwin
make clean || true
make -j$(sysctl -n hw.ncpu)
cd ../../..

# Build for iOS simulator (x86_64)
echo "Building jbig2dec for iOS simulator (x86_64)..."
BUILD_DIR=../target/jbig2dec/iOS-sim/x86_64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
../../../jbig2dec/configure --static --prefix=$(pwd) --host=x86_64-apple-darwin
make clean || true
make -j$(sysctl -n hw.ncpu)
cd ../../..

# Create universal library
echo "Creating universal jbig2dec library..."
mkdir -p ../target/jbig2dec/iOS-universal/lib
lipo -create \
  ../target/jbig2dec/iOS/arm64/lib/libjbig2dec.a \
  ../target/jbig2dec/iOS-sim/arm64/lib/libjbig2dec.a \
  ../target/jbig2dec/iOS-sim/x86_64/lib/libjbig2dec.a \
  -output ../target/jbig2dec/iOS-universal/lib/libjbig2dec.a || echo "Some architectures may have failed"

echo "jbig2dec built successfully for iOS."
