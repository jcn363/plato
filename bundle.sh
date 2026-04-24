#! /bin/sh

NICKEL_MENU_ARCHIVE="${1:-}"

[ -d dist ] || ./dist.sh

# Simple bundle mode: create full Plato distribution structure (pure Rust, no C libraries)
if [ -z "$NICKEL_MENU_ARCHIVE" ]; then
    echo "Creating simple bundle (no NickelMenu integration)..."
    PLATO_VERSION=$(cargo pkgid -p plato | cut -d '#' -f 2)
    
    # Create full distribution structure in dist/
    cp -r keyboard-layouts dist/
    cp -r css dist/
    cp -r hyphenation-patterns dist/
    cp -r icons dist/
    cp -r scripts dist/
    cp -r fonts dist/
    cp -r bin dist/
    cp -r dictionaries dist/
    cp -r resources dist/
    cp plato.sh dist/
    cp nickel.sh dist/
    cp config-sample.sh dist/
    cp Settings-sample.toml dist/
    cp convert-dictionary.sh dist/
    cp LICENSE-AGPLv3 dist/
    
    # Create KoboRoot.tgz with proper Kobo structure
    BUNDLE_TEMP="bundle-temp"
    [ -d "$BUNDLE_TEMP" ] && rm -Rf "$BUNDLE_TEMP"
    mkdir -p "$BUNDLE_TEMP/mnt/onboard/.adds/plato"
    cp -r dist/* "$BUNDLE_TEMP/mnt/onboard/.adds/plato/"
    cd "$BUNDLE_TEMP" || exit 1
    tar -czvf ../KoboRoot.tgz mnt
    cd ..
    mv KoboRoot.tgz dist/
    rm -Rf "$BUNDLE_TEMP"
    
    # Create zip bundle
    cd dist || exit 1
    zip -r "../plato-bundle-$PLATO_VERSION.zip" *
    cd ..
    
    # Clean up copied files and directories (keep only bundle)
    rm -rf dist/keyboard-layouts
    rm -rf dist/css
    rm -rf dist/hyphenation-patterns
    rm -rf dist/icons
    rm -rf dist/scripts
    rm -rf dist/fonts
    rm -rf dist/bin
    rm -rf dist/dictionaries
    rm -rf dist/resources
    rm -f dist/plato
    rm -f dist/plato.sh
    rm -f dist/nickel.sh
    rm -f dist/config-sample.sh
    rm -f dist/Settings-sample.toml
    rm -f dist/convert-dictionary.sh
    rm -f dist/LICENSE-AGPLv3
    
    echo "Bundle created: plato-bundle-$PLATO_VERSION.zip"
    echo "KoboRoot.tgz created in dist/"
    exit 0
fi

echo "Creating full bundle with NickelMenu integration..."

# Use a temporary directory within dist for bundling
BUNDLE_TEMP="dist/bundle-temp"
[ -d "$BUNDLE_TEMP" ] && rm -Rf "$BUNDLE_TEMP"
mkdir -p "$BUNDLE_TEMP"
cd "$BUNDLE_TEMP" || exit 1

if gzip -tq "$NICKEL_MENU_ARCHIVE"; then
	ln -s "$NICKEL_MENU_ARCHIVE" KoboRoot.tgz
else
	unzip "$NICKEL_MENU_ARCHIVE" KoboRoot.tgz 2>/dev/null || echo "Warning: Archive not found, continuing..."
fi

tar -xzvf KoboRoot.tgz
rm KoboRoot.tgz
mv mnt/onboard/.adds .
rm -Rf mnt

cp -r ../../dist/* .adds/plato
cp ../../contrib/NickelMenu/* .adds/nm

mkdir .kobo
tar -czvf .kobo/KoboRoot.tgz usr
rm -Rf usr

PLATO_VERSION=$(cargo pkgid -p plato | cut -d '#' -f 2)

zip -r plato-bundle-"$PLATO_VERSION".zip .adds .kobo
rm -Rf .adds .kobo
rm -f dist/KoboRoot.tgz

# Move bundle to dist directory and clean up temp
mv plato-bundle-"$PLATO_VERSION".zip ../../dist/
cd ../..
rm -Rf "$BUNDLE_TEMP"
