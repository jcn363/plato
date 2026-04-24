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

mkdir -p dist
mkdir dist/dictionaries

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

# No external libraries needed - pure Rust build
$STRIP_TOOL dist/plato
