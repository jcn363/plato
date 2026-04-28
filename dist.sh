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

# Create zip bundle for Kobo
if [ "$TARGET" = "arm" ]; then
    echo "Creating ZIP bundle for Kobo devices..."
    cd dist && zip -r "../plato-kobo.zip" . && cd ..
    echo "Created: plato-kobo.zip"
elif [ "$TARGET" = "arm64" ]; then
    echo "Creating ZIP bundle for Kobo 64-bit devices..."
    cd dist && zip -r "../plato-kobo-arm64.zip" . && cd ..
    echo "Created: plato-kobo-arm64.zip"
fi

# Create .deb for Linux mint (x86_64)
if [ "$TARGET" = "host" ] || [ "$TARGET" = "linuxmint" ]; then
    echo "Creating .deb package for Linux..."
    # Create debian package structure
    mkdir -p debian/DEBIAN
    mkdir -p debian/usr/local/bin
    mkdir -p debian/usr/share/applications
    mkdir -p debian/usr/share/icons/hicolor/64x64/apps
    
    # Copy binary
    cp "$TARGET_DIR/$CARGO_TARGET/$CARGO_PROFILE/plato" debian/usr/local/bin/plato
    
    # Create desktop file
    cat > debian/usr/share/applications/plato.desktop << 'DEOF'
[Desktop Entry]
Name=Plato
Exec=/usr/local/bin/plato
Type=Application
Icon=plato
Terminal=false
Categories=Office;Viewer;
DEOF
    
    # Copy icon (if exists)
    if [ -f "icons/plato.png" ]; then
        cp icons/plato.png debian/usr/share/icons/hicolor/64x64/apps/
    fi
    
    # Create control file
    cat > debian/DEBIAN/control << 'DEOF'
Package: plato
Version: $(date +%Y%m%d)
Section: office
Priority: optional
Architecture: amd64
Maintainer: Plato Team
Description: Document reader for Kobo and desktop
 Plato is a document reader for Kobo e-readers and Linux desktop.
DEOF
    
    # Build .deb
    dpkg-deb --build debian plato-linux.deb
    rm -Rf debian
    echo "Created: plato-linux.deb"
fi

echo "Distribution package created for $TARGET!"
