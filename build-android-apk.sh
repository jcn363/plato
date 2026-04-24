#!/usr/bin/env bash

set -e

echo "Building Plato APK for Android ARM64..."

# Check for Java (required for APK signing)
if ! command -v java &> /dev/null; then
    echo "Error: Java not found. APK signing requires Java."
    echo "Please install Java (e.g., 'sudo apt-get install default-jre')"
    exit 1
fi

# Build APK
echo "Building APK..."
cd crates/plato-android
cargo apk build --target aarch64-linux-android --release
cd ../..

# Create dist directory and move APK there
mkdir -p dist
cp crates/plato-android/target/release/apk/plato-android.apk dist/

echo "APK built successfully at: dist/plato-android.apk"
