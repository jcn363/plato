# Plato Debian Package for Linux Mint

This document explains how to build and install Plato on Linux Mint/Debian-based systems.

## Prerequisites

```bash
sudo apt-get update
sudo apt-get install -y dpkg-dev debhelper cargo rustc pkg-config
```

## Building the DEB Package

### Automated Build (Recommended)

```bash
cd /home/user/Desktop/plato
bash build-deb.sh
```

This will:
1. Check for required build dependencies
2. Build the x86_64 Linux binary using Cargo
3. Create the Debian package structure in `debian/plato/`
4. Build the `.deb` package using `dpkg-buildpackage`
5. Move the `.deb` file to `dist/`

### Manual Build

If you prefer to build manually:

```bash
# Build the binary
cd /home/user/Desktop/plato
cargo build --release --package plato --target x86_64-unknown-linux-gnu

# Create package structure
mkdir -p debian/plato/usr/bin
mkdir -p debian/plato/usr/share/applications
mkdir -p debian/plato/usr/share/icons/hicolor/scalable/apps
mkdir -p debian/plato/usr/share/plato
mkdir -p debian/plato/DEBIAN

# Copy files
cp target/x86_64-unknown-linux-gnu/release/plato debian/plato/usr/bin/
cp debian/plato.desktop debian/plato/usr/share/applications/
cp icons/plato.svg debian/plato/usr/share/icons/hicolor/scalable/apps/
cp -r fonts css icons keyboard-layouts translations debian/plato/usr/share/plato/

# Create control file
cat > debian/plato/DEBIAN/control << 'EOF'
Package: plato
Version: 0.9.45-1
Section: text
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.28)
Maintainer: Plato Team <plato@example.com>
Description: Document reader for Linux
 Plato is a document reader supporting PDF, EPUB, CBZ, and DJVU formats.
 This package provides the Linux version using the Linux framebuffer.
EOF

# Build package
dpkg-deb --build debian/plato plato_0.9.45-1_amd64.deb
mv plato_*.deb dist/
```

## Installing the DEB Package

```bash
cd /home/user/Desktop/plato/dist
sudo dpkg -i plato_0.9.45-1_amd64.deb

# Fix any missing dependencies
sudo apt-get install -f
```

## Running Plato on Linux Mint

After installation, Plato can be run from:
- Applications menu (search for "Plato")
- Command line: `plato`

## Package Contents

The DEB package includes:
- Binary: `/usr/bin/plato`
- Desktop file: `/usr/share/applications/plato.desktop`
- Icon: `/usr/share/icons/hicolor/scalable/apps/plato.svg`
- Resources: `/usr/share/plato/` (fonts, css, icons, keyboard layouts, translations)

## Known Issues

1. **No `sudo` access**: If you can't install build dependencies, use the manual build method and create the package without `dpkg-buildpackage`
2. **LSP errors in dependencies**: The `rar` crate may show LSP errors (trait bound issues), but these don't affect compilation
3. **Missing `speech-dispatcher`**: For TTS support, install `libspeechd-dev`

## AppImage (Portable) - Linux Mint

Plato provides an AppImage for Linux Mint/Debian-based systems:

```bash
# Download or build AppImage
cd /home/user/Desktop/plato/dist
chmod +x plato-x86_64.AppImage

# Run directly without installation (portable)
./plato-x86_64.AppImage
```

The AppImage includes all resources and runs on any Linux distribution without installation.

### Building AppImage for Linux Mint

```bash
# 1. Download appimagetool
cd /home/user/Desktop/plato
wget -q https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage -O appimagetool
chmod +x appimagetool

# 2. Create AppDir structure
mkdir -p AppDir/usr/bin AppDir/usr/share/applications
mkdir -p AppDir/usr/share/icons/hicolor/scalable/apps
mkdir -p AppDir/usr/share/plato

# 3. Copy files
cp target/x86_64-unknown-linux-gnu/release/plato AppDir/usr/bin/
cp debian/plato.desktop AppDir/
cp icons/plato.svg AppDir/
cp -r fonts css icons keyboard-layouts translations AppDir/usr/share/plato/

# 4. Create AppRun script
cat > AppDir/AppRun << 'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS}"
exec "${HERE}/usr/bin/plato" "$@"
EOF
chmod +x AppDir/AppRun

# 5. Build AppImage
./appimagetool AppDir plato-x86_64.AppImage

# 6. Move to dist/
mv plato-x86_64.AppImage dist/
chmod +x dist/plato-x86_64.AppImage
```

The AppImage is portable and runs on Linux Mint without installation.

## File Locations

- DEB package: `dist/plato_0.9.45-1_amd64.deb`
- AppImage: `dist/plato-x86_64.AppImage`
- Build script: `build-deb.sh`
- Debian config: `debian/` directory
- Control file: `debian/control`

## Troubleshooting

### Build fails with "cargo: command not found"
Install Rust: https://rustup.rs

### Package installs but Plato won't start
Check dependencies:
```bash
ldd /usr/bin/plato
```

### Icon not showing in menu
Update icon cache:
```bash
sudo update-icon-caches /usr/share/icons/*
```
