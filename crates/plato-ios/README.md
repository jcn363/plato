# Plato iOS

iOS-specific implementation for Plato document reader.

## Overview

This crate provides the iOS-specific implementation for Plato, including:

- Metal-based framebuffer rendering
- Touch event translation from UITouch
- iOS sandbox-aware path resolution
- C API for Swift bridge integration

## Building

```bash
# Build for iOS device (ARM64)
./build-ios.sh device

# Build for iOS simulator (ARM64 + x86_64)
./build-ios.sh simulator

# Build universal library (device + simulator)
./build-ios.sh universal
```

## Swift Integration

The crate provides a C API defined in `plato_ios.h` that can be called from Swift:

```swift
import Foundation

class PlatoBridge {
    static func initialize(width: UInt32, height: UInt32, 
                          libraryPath: String?, settingsPath: String?) -> Bool {
        let libraryPathData = libraryPath?.data(using: .utf8)
        let settingsPathData = settingsPath?.data(using: .utf8)
        
        return libraryPathData?.withUnsafeBytes { libraryPtr in
            settingsPathData?.withUnsafeBytes { settingsPtr in
                plato_init(
                    width, height,
                    libraryPtr.baseAddress?.assumingMemoryBound(to: UInt8.self),
                    libraryPathData?.count ?? 0,
                    settingsPtr.baseAddress?.assumingMemoryBound(to: UInt8.self),
                    settingsPathData?.count ?? 0
                )
            } ?? false
        } ?? false
    }
    
    static func handleTouch(fingerId: Int32, x: Float, y: Float, status: Int32) {
        plato_handle_touch(fingerId, x, y, status)
    }
    
    static func render() -> Bool {
        plato_render()
    }
    
    static func cleanup() {
        plato_cleanup()
    }
}
```

## Architecture

### Framebuffer (framebuffer.rs)

The `IOSFramebuffer` implements the `Framebuffer` trait using a software buffer that can be uploaded to Metal textures. For MVP, it uses a simple RGBA8888 buffer that Swift can copy to a Metal texture.

### Input (input.rs)

The `input` module translates iOS touch events to Plato's `DeviceEvent` system. It handles:

- Touch down/motion/up events
- Basic gesture recognition (tap, pinch, pan)

### Storage (storage.rs)

The `storage` module provides iOS-specific path resolution for:

- Library directory (Documents)
- Settings directory (Library)
- Cache directory (Caches)
- Temporary directory

### C API (lib.rs)

The C API provides functions that Swift can call:

- `plato_init`: Initialize the app with screen dimensions and paths
- `plato_get_context`: Get the global context pointer
- `plato_handle_touch`: Process touch events
- `plato_render`: Render the current view
- `plato_cleanup`: Cleanup resources

## MVP Limitations

This is a foundation implementation. The following are simplified for MVP:

- Gesture processing is basic (full gesture recognition needs more work)
- Metal rendering is stubbed (needs actual Metal texture management in Swift)
- Path resolution uses placeholders (needs actual iOS path APIs from Swift)
- View management is simplified (needs proper view lifecycle)

## Next Steps

1. Implement actual Metal texture management in Swift
2. Complete gesture recognition with proper gesture processor thread
3. Integrate with iOS file picker for document import
4. Add proper iOS app lifecycle handling (background/foreground transitions)
5. Implement proper view management and navigation
6. Add iOS-specific UI adaptations (touch-friendly controls, etc.)
