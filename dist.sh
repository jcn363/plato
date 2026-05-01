#!/bin/sh

set -e

# Fix sccache issue: unset RUSTC_WRAPPER if sccache not available
if ! command -v sccache &> /dev/null; then
    echo "Warning: sccache not found, disabling RUSTC_WRAPPER..."
    unset RUSTC_WRAPPER
    export RUSTC_WRAPPER=""
fi

# Accept TARGET argument (arm | arm64 | host | linuxmint), default to arm for backward compatibility
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
	host|linuxmint)
		LIB_DIR=""
		CARGO_TARGET="x86_64-unknown-linux-gnu"
		CARGO_PROFILE="release"
		STRIP_TOOL="strip"
		;;
	android-arm64)
		echo "Building for Android ARM64..."
		unset RUSTC_WRAPPER
		export RUSTC_WRAPPER=""
		export ANDROID_NDK=/home/user/Android/sdk/android-ndk-r26b
		export PATH=$ANDROID_NDK/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH
		CARGO_TARGET="aarch64-linux-android"
		CARGO_PROFILE="release-arm64"
		./build.sh android-arm64
		exit 0
		;;
	android-arm32)
		echo "Building for Android ARM32..."
		unset RUSTC_WRAPPER
		export RUSTC_WRAPPER=""
		export ANDROID_NDK=/home/user/Android/sdk/android-ndk-r26b
		export PATH=$ANDROID_NDK/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH
		CARGO_TARGET="armv7-linux-androideabi"
		CARGO_PROFILE="release-arm"
		./build.sh android-arm32
		exit 0
		;;
	*)
		echo "Error: Invalid target '$TARGET'. Use 'arm', 'arm64', 'host', 'linuxmint', 'android-arm64', or 'android-arm32'."
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
cp LICENSE-AGPLv3 dist/

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
    echo '[Desktop Entry]' > debian/usr/share/applications/plato.desktop
    echo 'Name=Plato' >> debian/usr/share/applications/plato.desktop
    echo 'Exec=/usr/local/bin/plato' >> debian/usr/share/applications/plato.desktop
    echo 'Type=Application' >> debian/usr/share/applications/plato.desktop
    echo 'Icon=plato' >> debian/usr/share/applications/plato.desktop
    echo 'Terminal=false' >> debian/usr/share/applications/plato.desktop
    echo 'Categories=Office;Viewer;' >> debian/usr/share/applications/plato.desktop
    
    # Copy icon (if exists)
    if [ -f "icons/plato.png" ]; then
        cp icons/plato.png debian/usr/share/icons/hicolor/64x64/apps/
    fi
    
    # Create control file
    echo "Package: plato" > debian/DEBIAN/control
    echo "Version: $(date +%Y%m%d)" >> debian/DEBIAN/control
    echo "Section: office" >> debian/DEBIAN/control
    echo "Priority: optional" >> debian/DEBIAN/control
    echo "Architecture: amd64" >> debian/DEBIAN/control
    echo "Depends: libc6 (>= 2.28)" >> debian/DEBIAN/control
    echo "Maintainer: Plato Team" >> debian/DEBIAN/control
    echo "Description: Document reader for Kobo and desktop" >> debian/DEBIAN/control
    echo " Plato is a document reader for Kobo e-readers and Linux desktop." >> debian/DEBIAN/control
    
    # Build .deb
    dpkg-deb --build debian plato-linux.deb
    rm -Rf debian
    echo "Created: plato-linux.deb"
fi

echo "Distribution package created for $TARGET!"
