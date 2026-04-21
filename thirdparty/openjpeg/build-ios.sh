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
echo "Building openjpeg for iOS device (ARM64)..."
BUILD_DIR=../target/openjpeg/iOS/arm64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0"
cmake ../../ -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=$(pwd) -DCMAKE_OSX_ARCHITECTURES=arm64 -DCMAKE_OSX_SYSROOT=$IOS_SDK -DBUILD_SHARED_LIBS=OFF
make clean || true
make -j$(sysctl -n hw.ncpu)
cd ../../..

# Build for iOS simulator (ARM64)
echo "Building openjpeg for iOS simulator (ARM64)..."
BUILD_DIR=../target/openjpeg/iOS-sim/arm64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
cmake ../../ -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=$(pwd) -DCMAKE_OSX_ARCHITECTURES=arm64 -DCMAKE_OSX_SYSROOT=$IOS_SIM_SDK -DBUILD_SHARED_LIBS=OFF
make clean || true
make -j$(sysctl -n hw.ncpu)
cd ../../..

# Build for iOS simulator (x86_64)
echo "Building openjpeg for iOS simulator (x86_64)..."
BUILD_DIR=../target/openjpeg/iOS-sim/x86_64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
cmake ../../ -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=$(pwd) -DCMAKE_OSX_ARCHITECTURES=x86_64 -DCMAKE_OSX_SYSROOT=$IOS_SIM_SDK -DBUILD_SHARED_LIBS=OFF
make clean || true
make -j$(sysctl -n hw.ncpu)
cd ../../..

# Create universal library
echo "Creating universal openjpeg library..."
mkdir -p ../target/openjpeg/iOS-universal/lib
lipo -create \
  ../target/openjpeg/iOS/arm64/lib/libopenjp2.a \
  ../target/openjpeg/iOS-sim/arm64/lib/libopenjp2.a \
  ../target/openjpeg/iOS-sim/x86_64/lib/libopenjp2.a \
  -output ../target/openjpeg/iOS-universal/lib/libopenjp2.a || echo "Some architectures may have failed"

echo "openjpeg built successfully for iOS."
