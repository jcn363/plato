#!/bin/sh
#
# Unified build script for Kobo Elipsa and OnePlus Nord 2 5G
#
# Usage:
#   ./build-devices.sh elipsa      # Build for Kobo Elipsa
#   ./build-devices.sh oneplus     # Build for OnePlus Nord 2 5G
#   ./build-devices.sh all         # Build for both devices
#   ./build-devices.sh help        # Show help

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEVICE="${1:-help}"

print_usage() {
    echo "Usage: $0 [DEVICE]"
    echo ""
    echo "Devices:"
    echo "  elipsa     Build for Kobo Elipsa (32-bit ARM, 1GB RAM)"
    echo "  oneplus    Build for OnePlus Nord 2 5G (Android ARM64, 12GB RAM)"
    echo "  all        Build for both devices"
    echo "  help       Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 elipsa          # Build for Elipsa only"
    echo "  $0 oneplus         # Build for OnePlus Nord 2 5G"
    echo "  $0 all             # Build for both"
    echo ""
    echo "Device specs:"
    echo ""
    echo "  Kobo Elipsa:"
    echo "    - Allwinner B300 (ARMv7, 4-core)"
    echo "    - 1GB LPDDR4 RAM"
    echo "    - E-ink display with stylus"
    echo "    - Optimizations: 2MB/8MB buffers, 3 workers"
    echo ""
    echo "  OnePlus Nord 2 5G:"
    echo "    - Dimensity 1200-AI (8-core, 6nm)"
    echo "    - 12GB LPDDR4X RAM"
    echo "    - 90Hz OLED display"
    echo "    - Optimizations: 4MB/16MB buffers, 4-6 workers, 90Hz animations"
    echo ""
}

case "$DEVICE" in
    elipsa)
        echo "=========================================="
        echo "Building for Kobo Elipsa"
        echo "=========================================="
        echo ""
        if [ -x "$SCRIPT_DIR/build-elipsa.sh" ]; then
            "$SCRIPT_DIR/build-elipsa.sh"
        else
            echo "Error: build-elipsa.sh not found"
            exit 1
        fi
        ;;
    
    oneplus|oneplus-nord2|nord2)
        echo "=========================================="
        echo "Building for OnePlus Nord 2 5G"
        echo "=========================================="
        echo ""
        if [ -x "$SCRIPT_DIR/build-oneplus-nord2.sh" ]; then
            "$SCRIPT_DIR/build-oneplus-nord2.sh"
        else
            echo "Error: build-oneplus-nord2.sh not found"
            exit 1
        fi
        ;;
    
    all|both)
        echo "=========================================="
        echo "Building for all supported devices"
        echo "=========================================="
        echo ""
        
        # Build Elipsa
        echo ""
        echo ">>> [1/2] Building for Kobo Elipsa..."
        echo ""
        if [ -x "$SCRIPT_DIR/build-elipsa.sh" ]; then
            "$SCRIPT_DIR/build-elipsa.sh"
        fi
        
        # Build OnePlus
        echo ""
        echo ">>> [2/2] Building for OnePlus Nord 2 5G..."
        echo ""
        if [ -x "$SCRIPT_DIR/build-oneplus-nord2.sh" ]; then
            "$SCRIPT_DIR/build-oneplus-nord2.sh"
        fi
        
        echo ""
        echo "=========================================="
        echo "All device builds complete!"
        echo "=========================================="
        echo ""
        echo "Binaries:"
        if [ -f "$SCRIPT_DIR/target-arm/arm-unknown-linux-gnueabihf/release-arm/plato" ]; then
            echo "  Elipsa:  target-arm/arm-unknown-linux-gnueabihf/release-arm/plato"
            ls -lh "$SCRIPT_DIR/target-arm/arm-unknown-linux-gnueabihf/release-arm/plato" 2>/dev/null || true
        fi
        if [ -f "$SCRIPT_DIR/target/aarch64-linux-android/release/plato" ]; then
            echo "  OnePlus: target/aarch64-linux-android/release/plato"
            ls -lh "$SCRIPT_DIR/target/aarch64-linux-android/release/plato" 2>/dev/null || true
        fi
        echo ""
        ;;
    
    help|--help|-h)
        print_usage
        exit 0
        ;;
    
    *)
        echo "Error: Unknown device '$DEVICE'"
        echo ""
        print_usage
        exit 1
        ;;
esac

exit 0
