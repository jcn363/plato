#!/bin/bash
# Download accessibility fonts for Plato
# This script downloads dyslexia-friendly fonts: OpenDyslexic, Atkinson Hyperlegible, and Lexend

set -e

FONTS_DIR="$(dirname "$0")/debian/plato/usr/share/plato/fonts/accessibility"
mkdir -p "$FONTS_DIR"

echo "Downloading accessibility fonts..."

# OpenDyslexic
echo "Downloading OpenDyslexic..."
if [ ! -f "$FONTS_DIR/OpenDyslexic-Regular.otf" ]; then
    wget -q "https://github.com/antijingoist/OpenDyslexic/releases/download/v0.91.12/OpenDyslexic-Regular.otf" \
        -O "$FONTS_DIR/OpenDyslexic-Regular.otf" 2>/dev/null || \
    curl -sL "https://github.com/antijingoist/OpenDyslexic/releases/download/v0.91.12/OpenDyslexic-Regular.otf" \
        -o "$FONTS_DIR/OpenDyslexic-Regular.otf"
fi

# Atkinson Hyperlegible
echo "Downloading Atkinson Hyperlegible..."
if [ ! -f "$FONTS_DIR/AtkinsonHyperlegible-Regular.ttf" ]; then
    # Atkinson Hyperlegible is available from Google Fonts
    wget -q "https://fonts.gstatic.com/s/atkinsonhyperlegible/v11/3tKnUzG-y20hN0i20b1Ex2-CyZtGcBqgaiuzlz_JH4HahvQlxM.0.woff2" \
        -O "$FONTS_DIR/AtkinsonHyperlegible-Regular.ttf" 2>/dev/null || \
    echo "Note: Atkinson Hyperlegible may need to be downloaded manually from Google Fonts"
fi

# Lexend
echo "Downloading Lexend..."
if [ ! -f "$FONTS_DIR/Lexend-Regular.ttf" ]; then
    wget -q "https://fonts.gstatic.com/s/lexend/v25/Wl9R31gxPnPbE9Pk8q1bg.woff2" \
        -O "$FONTS_DIR/Lexend-Regular.ttf" 2>/dev/null || \
    echo "Note: Lexend may need to be downloaded manually from Google Fonts"
fi

echo ""
echo "Font download complete!"
echo "Fonts installed to: $FONTS_DIR"
echo ""
echo "Note: Some fonts may need to be downloaded manually due to licensing."
echo "Visit:"
echo "  - OpenDyslexic: https://opendyslexic.org/"
echo "  - Atkinson Hyperlegible: https://fonts.google.com/specimen/Atkinson+Hyperlegible"
echo "  - Lexend: https://fonts.google.com/specimen/Lexend"
echo ""
echo "Alternatively, if you have these fonts installed on your system,"
echo "you can copy them to the accessibility fonts directory."
