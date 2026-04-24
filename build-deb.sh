#!/bin/bash
# Build Debian package for Plato Document Reader
# This script builds the x86_64 Linux binary and packages it as a .deb

set -e

echo "Building Plato Document Reader for Linux..."

# Check if dpkg-buildpackage is available
if ! command -v dpkg-buildpackage &> /dev/null; then
    echo "Error: dpkg-buildpackage not found. Please install dpkg-dev:"
    echo "  sudo apt-get install dpkg-dev debhelper"
    exit 1
fi

# Build the debian package
echo "Building Debian package..."
dpkg-buildpackage -us -uc -b

# Create dist directory and move .deb files there
mkdir -p dist
mv ../plato_*.deb dist/ 2>/dev/null || true
mv ../plato-dbgsym_*.ddeb dist/ 2>/dev/null || true

echo "Build complete!"
echo "Package location: dist/plato_*.deb"
