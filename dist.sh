#! /bin/sh

set -e

# Accept TARGET argument (arm | arm64), default to arm for backward compatibility
TARGET="${1:-arm}"

# Resolve target-derived variables
case "$TARGET" in
	arm)
		LIB_DIR="libs"
		CARGO_TARGET="arm-unknown-linux-gnueabihf"
		CARGO_PROFILE="release-arm"
		STRIP_TOOL="arm-linux-gnueabihf-strip"
		;;
	arm64)
		LIB_DIR="libs64"
		CARGO_TARGET="aarch64-unknown-linux-gnu"
		CARGO_PROFILE="release-arm64"
		STRIP_TOOL="aarch64-linux-gnu-strip"
		;;
	*)
		echo "Error: Invalid target '$TARGET'. Use 'arm' or 'arm64'."
		exit 1
		;;
esac

# Determine target directory (respect CARGO_TARGET_DIR if set)
TARGET_DIR="${CARGO_TARGET_DIR:-target}"

# Fail-fast prerequisite checks
echo "Checking prerequisites for target '$TARGET'..."

if [ ! -d "$LIB_DIR" ]; then
	echo "Error: Library directory '$LIB_DIR' not found. Run './build.sh $TARGET' first."
	exit 1
fi

# Check for required shared libraries
required_libs="libz.so libbz2.so libpng16.so libjpeg.so libopenjp2.so libjbig2dec.so libfreetype.so libharfbuzz.so libgumbo.so libdjvulibre.so libmupdf.so"
for lib in $required_libs; do
	if [ ! -e "$LIB_DIR/$lib" ]; then
		echo "Error: Required library '$LIB_DIR/$lib' not found."
		exit 1
	fi
done

# Check for strip tool
if ! command -v "$STRIP_TOOL" > /dev/null 2>&1; then
	echo "Error: Strip tool '$STRIP_TOOL' not found on PATH."
	exit 1
fi

echo "Prerequisites check passed."

[ -d dist ] && rm -Rf dist

[ -d bin ] || ./download.sh 'bin/*'
[ -d resources ] || ./download.sh 'resources/*'
[ -d hyphenation-patterns ] || ./download.sh 'hyphenation-patterns/*'

# Target-specific build fallback
if [ ! -e "$TARGET_DIR/$CARGO_TARGET/$CARGO_PROFILE/plato" ]; then
	echo "Building plato for target '$TARGET'..."
	./build.sh "$TARGET"
fi

mkdir -p dist/libs
mkdir dist/dictionaries

cp "$LIB_DIR/libz.so" dist/libs/libz.so.1
cp "$LIB_DIR/libbz2.so" dist/libs/libbz2.so.1.0

cp "$LIB_DIR/libpng16.so" dist/libs/libpng16.so.16
cp "$LIB_DIR/libjpeg.so" dist/libs/libjpeg.so.9
cp "$LIB_DIR/libopenjp2.so" dist/libs/libopenjp2.so.7
cp "$LIB_DIR/libjbig2dec.so" dist/libs/libjbig2dec.so.0

cp "$LIB_DIR/libfreetype.so" dist/libs/libfreetype.so.6
cp "$LIB_DIR/libharfbuzz.so" dist/libs/libharfbuzz.so.0

cp "$LIB_DIR/libgumbo.so" dist/libs/libgumbo.so.2
cp "$LIB_DIR/libdjvulibre.so" dist/libs/libdjvulibre.so.21
cp "$LIB_DIR/libmupdf.so" dist/libs

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
cp "$TARGET_DIR/$CARGO_TARGET/$CARGO_PROFILE/plato" dist/

# Build epub_editor if not present
if [ ! -e "$TARGET_DIR/$CARGO_TARGET/$CARGO_PROFILE/epub_editor" ]; then
	echo "Building epub_editor for target '$TARGET'..."
	cd crates/epub_editor
	CARGO_TARGET_DIR="$TARGET_DIR" cargo build --profile "$CARGO_PROFILE" --target "$CARGO_TARGET"
	cd ../..
fi
cp "$TARGET_DIR/$CARGO_TARGET/$CARGO_PROFILE/epub_editor" dist/
cp contrib/*.sh dist
cp contrib/Settings-sample.toml dist
cp LICENSE-AGPLv3 dist

if command -v patchelf > /dev/null 2>&1; then
	patchelf --remove-rpath dist/libs/*
else
	echo "Warning: patchelf not found, skipping rpath removal."
fi

$STRIP_TOOL dist/plato dist/libs/*
