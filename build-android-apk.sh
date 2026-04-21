#!/usr/bin/env bash

set -e

echo "Building Plato APK for Android ARM64..."

# Build all native libraries for Android
echo "Building native libraries for Android..."
cd thirdparty

# zlib
echo "Building zlib..."
cd zlib
./build-android.sh
cd ..

# bzip2
echo "Building bzip2..."
cd bzip2
./build-android.sh
cd ..

# libpng
echo "Building libpng..."
cd libpng
./build-android.sh
cd ..

# libjpeg
echo "Building libjpeg..."
cd libjpeg
./build-android.sh
cd ..

# openjpeg
echo "Building openjpeg..."
cd openjpeg
./build-android.sh
cd ..

# jbig2dec
echo "Building jbig2dec..."
cd jbig2dec
./build-android.sh
cd ..

# freetype2
echo "Building freetype2..."
cd freetype2
./build-android.sh
cd ..

# harfbuzz
echo "Building harfbuzz..."
cd harfbuzz
./build-android.sh
cd ..

# gumbo
echo "Building gumbo..."
cd gumbo
./build-android.sh
cd ..

# djvulibre
echo "Building djvulibre..."
cd djvulibre
./build-android.sh
cd ..

# mupdf
echo "Building mupdf..."
cd mupdf
./build-android.sh
cd ..

cd ..

# Build mupdf_wrapper
echo "Building mupdf_wrapper..."
cd mupdf_wrapper
./build-android.sh
cd ..

echo "All native libraries built successfully."

# Build APK
echo "Building APK..."
cargo apk build --target aarch64-linux-android -p plato-android

echo "APK built successfully at: target/debug/apk/plato-android.apk"
