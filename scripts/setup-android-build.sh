#!/usr/bin/env bash

# Setup script for Plato Android APK builds
# This script configures the environment for building Plato for Android devices (OnePlus, etc.)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Android NDK configuration
ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT:-$HOME/Android/sdk/android-ndk-r26b}"
NDK_HOST_TOOLCHAIN="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64"
ANDROID_API_LEVEL="${ANDROID_API_LEVEL:-21}"

# Verify NDK exists
if [ ! -d "$ANDROID_NDK_ROOT" ]; then
    echo "Error: Android NDK not found at $ANDROID_NDK_ROOT"
    echo "Please set ANDROID_NDK_ROOT environment variable or install NDK"
    exit 1
fi

# Verify toolchain exists
if [ ! -d "$NDK_HOST_TOOLCHAIN" ]; then
    echo "Error: NDK toolchain not found at $NDK_HOST_TOOLCHAIN"
    exit 1
fi

echo "Configuring Android NDK environment..."
echo "  NDK Root: $ANDROID_NDK_ROOT"
echo "  API Level: $ANDROID_API_LEVEL"

# Export environment variables for aarch64
export CC_aarch64_linux_android="$NDK_HOST_TOOLCHAIN/bin/aarch64-linux-android${ANDROID_API_LEVEL}-clang"
export CXX_aarch64_linux_android="$NDK_HOST_TOOLCHAIN/bin/aarch64-linux-android${ANDROID_API_LEVEL}-clang++"
export AR_aarch64_linux_android="$NDK_HOST_TOOLCHAIN/bin/llvm-ar"
export CFLAGS_aarch64_linux_android="-fPIC"
export CXXFLAGS_aarch64_linux_android="-fPIC -stdlib=libc++"
export LDFLAGS_aarch64_linux_android="-L$NDK_HOST_TOOLCHAIN/lib64"

# Export environment variables for armv7
export CC_armv7_linux_androideabi="$NDK_HOST_TOOLCHAIN/bin/armv7a-linux-androideabi${ANDROID_API_LEVEL}-clang"
export CXX_armv7_linux_androideabi="$NDK_HOST_TOOLCHAIN/bin/armv7a-linux-androideabi${ANDROID_API_LEVEL}-clang++"
export AR_armv7_linux_androideabi="$NDK_HOST_TOOLCHAIN/bin/llvm-ar"
export CFLAGS_armv7_linux_androideabi="-fPIC"
export CXXFLAGS_armv7_linux_androideabi="-fPIC -stdlib=libc++"
export LDFLAGS_armv7_linux_androideabi="-L$NDK_HOST_TOOLCHAIN/lib"

# Disable sccache for Android (causes cross-compilation issues)
export RUSTC_WRAPPER=""

echo "Android build environment configured successfully"
echo ""
echo "To build the Plato Android library:"
echo "  cd crates/plato-android"
echo "  cargo build --target aarch64-linux-android --release"
echo ""
echo "To package as APK (requires cargo-apk):"
echo "  cargo apk build --target aarch64-linux-android --release"
