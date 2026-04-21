#!/bin/sh

set -e

# Check if running on macOS
case "$OSTYPE" in
  darwin*)
    # On macOS, continue with build
    ;;
  *)
    echo "Error: iOS builds require macOS. This script must be run on macOS."
    exit 1
    ;;
esac

# Set up iOS SDK paths
export IOS_SDK=$(xcrun --sdk iphoneos --show-sdk-path)
export IOS_SIM_SDK=$(xcrun --sdk iphonesimulator --show-sdk-path)
export IOS_CC=$(xcrun --sdk iphoneos --find clang)
export IOS_AR=$(xcrun --sdk iphoneos --find ar)

# Build for iOS device (ARM64)
echo "Building mupdf_wrapper for iOS device (ARM64)..."
export CFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0 -fPIC -O2"
BUILD_DIR=../target/mupdf_wrapper/iOS/arm64
mkdir -p $BUILD_DIR
$IOS_CC $CPPFLAGS $CFLAGS -I../thirdparty/mupdf/include -c mupdf_wrapper.c -o ${BUILD_DIR}/mupdf_wrapper.o
$IOS_AR -rcs ${BUILD_DIR}/libmupdf_wrapper.a ${BUILD_DIR}/mupdf_wrapper.o

# Build for iOS simulator (ARM64)
echo "Building mupdf_wrapper for iOS simulator (ARM64)..."
export CFLAGS="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
BUILD_DIR_SIM_ARM64=../target/mupdf_wrapper/iOS-sim/arm64
mkdir -p $BUILD_DIR_SIM_ARM64
$IOS_CC $CPPFLAGS $CFLAGS -I../thirdparty/mupdf/include -c mupdf_wrapper.c -o ${BUILD_DIR_SIM_ARM64}/mupdf_wrapper.o
$IOS_AR -rcs ${BUILD_DIR_SIM_ARM64}/libmupdf_wrapper.a ${BUILD_DIR_SIM_ARM64}/mupdf_wrapper.o

# Build for iOS simulator (x86_64)
echo "Building mupdf_wrapper for iOS simulator (x86_64)..."
export CFLAGS="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
BUILD_DIR_SIM_X86_64=../target/mupdf_wrapper/iOS-sim/x86_64
mkdir -p $BUILD_DIR_SIM_X86_64
$IOS_CC $CPPFLAGS $CFLAGS -I../thirdparty/mupdf/include -c mupdf_wrapper.c -o ${BUILD_DIR_SIM_X86_64}/mupdf_wrapper.o
$IOS_AR -rcs ${BUILD_DIR_SIM_X86_64}/libmupdf_wrapper.a ${BUILD_DIR_SIM_X86_64}/mupdf_wrapper.o

# Create universal library
echo "Creating universal mupdf_wrapper library..."
mkdir -p ../target/mupdf_wrapper/iOS-universal
lipo -create \
  ${BUILD_DIR}/libmupdf_wrapper.a \
  ${BUILD_DIR_SIM_ARM64}/libmupdf_wrapper.a \
  ${BUILD_DIR_SIM_X86_64}/libmupdf_wrapper.a \
  -output ../target/mupdf_wrapper/iOS-universal/libmupdf_wrapper.a

echo "mupdf_wrapper built successfully for iOS."
