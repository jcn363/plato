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

# Get absolute path to project root
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJECT_ROOT=$(cd "$SCRIPT_DIR/../.." && pwd)

# Build for iOS device (ARM64)
echo "Building gumbo for iOS device (ARM64)..."
BUILD_DIR=$PROJECT_ROOT/target/gumbo/iOS/arm64
mkdir -p $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0"
if [ ! -f ./configure ]; then
  ./autogen.sh
fi
./configure --prefix=$BUILD_DIR --host=arm-apple-darwin --enable-static
make clean || true
make -j$(sysctl -n hw.ncpu)
make install

# Build for iOS simulator (ARM64)
echo "Building gumbo for iOS simulator (ARM64)..."
BUILD_DIR=$PROJECT_ROOT/target/gumbo/iOS-sim/arm64
mkdir -p $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
./configure --prefix=$BUILD_DIR --host=arm-apple-darwin --enable-static
make clean || true
make -j$(sysctl -n hw.ncpu)
make install

# Build for iOS simulator (x86_64)
echo "Building gumbo for iOS simulator (x86_64)..."
BUILD_DIR=$PROJECT_ROOT/target/gumbo/iOS-sim/x86_64
mkdir -p $BUILD_DIR
export CFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export LDFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0"
./configure --prefix=$BUILD_DIR --host=x86_64-apple-darwin --enable-static
make clean || true
make -j$(sysctl -n hw.ncpu)
make install

# Create universal library (device + simulator)
echo "Creating universal gumbo library..."
mkdir -p $PROJECT_ROOT/target/gumbo/iOS-device/lib
mkdir -p $PROJECT_ROOT/target/gumbo/iOS-simulator/lib
lipo -create \
  $PROJECT_ROOT/target/gumbo/iOS-sim/arm64/lib/libgumbo.a \
  $PROJECT_ROOT/target/gumbo/iOS-sim/x86_64/lib/libgumbo.a \
  -output $PROJECT_ROOT/target/gumbo/iOS-simulator/lib/libgumbo.a || echo "Some simulator architectures may have failed"
cp $PROJECT_ROOT/target/gumbo/iOS/arm64/lib/libgumbo.a $PROJECT_ROOT/target/gumbo/iOS-device/lib/

echo "gumbo built successfully for iOS."
