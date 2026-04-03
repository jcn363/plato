#! /bin/sh

set -e

# Determine target directory (respect CARGO_TARGET_DIR if set)
TARGET_DIR="${CARGO_TARGET_DIR:-target}"

[ -d dist ] && rm -Rf dist

[ -d bin ] || ./download.sh 'bin/*'
[ -d resources ] || ./download.sh 'resources/*'
[ -d hyphenation-patterns ] || ./download.sh 'hyphenation-patterns/*'
[ -e "$TARGET_DIR/arm-unknown-linux-gnueabihf/release-arm/plato" ] || ./build.sh

mkdir -p dist/libs
mkdir dist/dictionaries

cp libs/libz.so dist/libs/libz.so.1
cp libs/libbz2.so dist/libs/libbz2.so.1.0

cp libs/libpng16.so dist/libs/libpng16.so.16
cp libs/libjpeg.so dist/libs/libjpeg.so.9
cp libs/libopenjp2.so dist/libs/libopenjp2.so.7
cp libs/libjbig2dec.so dist/libs/libjbig2dec.so.0

cp libs/libfreetype.so dist/libs/libfreetype.so.6
cp libs/libharfbuzz.so dist/libs/libharfbuzz.so.0

cp libs/libgumbo.so dist/libs/libgumbo.so.2
cp libs/libdjvulibre.so dist/libs/libdjvulibre.so.21
cp libs/libmupdf.so dist/libs

cp -R hyphenation-patterns dist
cp -R keyboard-layouts dist
cp -R bin dist
cp -R scripts dist
cp -R icons dist
cp -R resources dist
cp -R fonts dist
cp -R css dist
find dist/css -name '*-user.css' -delete
find dist/keyboard-layouts -name '*-user.json' -delete
find dist/hyphenation-patterns -name '*.bounds' -delete
find dist/scripts -name 'wifi-*-*.sh' -delete
cp "$TARGET_DIR/arm-unknown-linux-gnueabihf/release-arm/plato" dist/

# Build epub_editor if not present
if [ ! -e "$TARGET_DIR/arm-unknown-linux-gnueabihf/release/epub_editor" ]; then
	echo "Building epub_editor..."
	cd epub_editor
	CARGO_TARGET_DIR="$TARGET_DIR" cargo build --release --target arm-unknown-linux-gnueabihf
	cd ..
fi
cp "$TARGET_DIR/arm-unknown-linux-gnueabihf/release/epub_editor" dist/
cp contrib/*.sh dist
cp contrib/Settings-sample.toml dist
cp LICENSE-AGPLv3 dist

if command -v patchelf > /dev/null 2>&1; then
	patchelf --remove-rpath dist/libs/*
else
	echo "Warning: patchelf not found, skipping rpath removal."
fi

arm-linux-gnueabihf-strip dist/plato dist/libs/*
