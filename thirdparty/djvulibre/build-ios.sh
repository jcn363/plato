#!/bin/sh

set -e

case "$OSTYPE" in
  darwin*) ;;
  *) echo "Error: iOS builds require macOS."; exit 1 ;;
esac

echo "Skipping djvulibre for iOS - libtool build system incompatible with iOS cross-compilation"
echo "djvulibre is not critical for iOS MuPDF functionality"
exit 0
