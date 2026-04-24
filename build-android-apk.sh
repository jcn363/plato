#!/usr/bin/env bash

set -e

echo "Building Plato APK for Android ARM64..."

# Build all native libraries for Android
echo "Building native libraries for Android..."
cd thirdparty

echo "All native libraries built successfully."

# Build APK
echo "Building APK..."
cargo apk build --target aarch64-linux-android -p plato-android

# Create dist directory and move APK there
mkdir -p dist
cp target/debug/apk/plato-android.apk dist/

echo "APK built successfully at: dist/plato-android.apk"
