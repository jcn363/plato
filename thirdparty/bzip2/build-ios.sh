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
echo "Building bzip2 for iOS device (ARM64)..."
BUILD_DIR=../target/bzip2/iOS/arm64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0 -fPIC -O2"
make -f ../../../thirdparty/bzip2/Makefile -C ../../../thirdparty/bzip2 clean || true
make -f ../../../thirdparty/bzip2/Makefile -C ../../../thirdparty/bzip2 libbz2.a CC="$IOS_CC" CFLAGS="$CFLAGS"
cp ../../../thirdparty/bzip2/libbz2.a .
cd ../../..

# Build for iOS simulator (ARM64)
echo "Building bzip2 for iOS simulator (ARM64)..."
BUILD_DIR=../target/bzip2/iOS-sim/arm64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
make -f ../../../thirdparty/bzip2/Makefile -C ../../../thirdparty/bzip2 clean || true
make -f ../../../thirdparty/bzip2/Makefile -C ../../../thirdparty/bzip2 libbz2.a CC="$IOS_CC" CFLAGS="$CFLAGS"
cp ../../../thirdparty/bzip2/libbz2.a .
cd ../../..

# Build for iOS simulator (x86_64)
echo "Building bzip2 for iOS simulator (x86_64)..."
BUILD_DIR=../target/bzip2/iOS-sim/x86_64
mkdir -p $BUILD_DIR
cd $BUILD_DIR
export CFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
make -f ../../../thirdparty/bzip2/Makefile -C ../../../thirdparty/bzip2 clean || true
make -f ../../../thirdparty/bzip2/Makefile -C ../../../thirdparty/bzip2 libbz2.a CC="$IOS_CC" CFLAGS="$CFLAGS"
cp ../../../thirdparty/bzip2/libbz2.a .
cd ../../..

# Create universal library
echo "Creating universal bzip2 library..."
mkdir -p ../target/bzip2/iOS-universal/lib
lipo -create \
  ../target/bzip2/iOS/arm64/libbz2.a \
  ../target/bzip2/iOS-sim/arm64/libbz2.a \
  ../target/bzip2/iOS-sim/x86_64/libbz2.a \
  -output ../target/bzip2/iOS-universal/lib/libbz2.a || echo "Some architectures may have failed"

echo "bzip2 built successfully for iOS."
