#!/bin/bash

set -e

TARGET="arm-unknown-linux-gnueabihf"
CROSS_PREFIX="${TARGET}"
BUILD_DIR="$(pwd)/target/${TARGET}/release"

echo "Building epub_editor for ${TARGET}..."

if ! rustup target list | grep -q "${TARGET}"; then
    echo "Adding target ${TARGET}..."
    rustup target add ${TARGET}
fi

echo "Compiling..."
cargo build --release --target ${TARGET} --target-dir ./target

echo "Build complete!"
echo "Binary: ${BUILD_DIR}/epub_editor"

echo ""
echo "To copy to Kobo:"
echo "  scp ${BUILD_DIR}/epub_editor kobo:/mnt/onboard/.adds/plato/bin/"
