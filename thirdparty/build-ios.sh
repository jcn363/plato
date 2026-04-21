#!/usr/bin/env bash

set -e

echo "Building native dependencies for iOS..."

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
  echo "Error: iOS builds require macOS. This script must be run on macOS."
  echo "For cross-compilation from Linux, use the main build-ios.sh script in the project root."
  exit 1
fi

# Set up iOS SDK paths
export IOS_SDK=$(xcrun --sdk iphoneos --show-sdk-path)
export IOS_SIM_SDK=$(xcrun --sdk iphonesimulator --show-sdk-path)
export IOS_CC=$(xcrun --sdk iphoneos --find clang)
export IOS_AR=$(xcrun --sdk iphoneos --find ar)

# Common iOS build flags
export IOS_CFLAGS="-arch arm64 -isysroot $IOS_SDK -miphoneos-version-min=12.0 -fPIC -O2"
export IOS_SIM_CFLAGS_ARM64="-arch arm64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"
export IOS_SIM_CFLAGS_X86_64="-arch x86_64 -isysroot $IOS_SIM_SDK -mios-simulator-version-min=12.0 -fPIC -O2"

# Build each library
LIBS=("zlib" "bzip2" "libpng" "libjpeg" "openjpeg" "jbig2dec" "freetype2" "harfbuzz" "gumbo" "djvulibre" "mupdf")

for lib in "${LIBS[@]}"; do
  if [ -d "$lib" ] && [ -f "$lib/build-ios.sh" ]; then
    echo "Building $lib for iOS..."
    cd "$lib"
    ./build-ios.sh
    cd ..
  elif [ -d "$lib" ]; then
    echo "Warning: $lib/build-ios.sh not found, skipping $lib"
    echo "Create $lib/build-ios.sh to enable iOS builds for this library"
  else
    echo "Warning: Directory $lib not found, skipping"
  fi
done

echo "Native dependencies build completed."
