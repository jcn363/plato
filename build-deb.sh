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

echo "Build complete!"
echo "Package location: ../plato_*.deb"
