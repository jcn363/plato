# Plan to Support iPhone and iPad in Plato

## Overview

Plato is currently designed for Kobo e-readers with e-ink displays. To support iPhone and iPad, we need to adapt the rendering, input handling, and platform-specific code to work with iOS while maintaining the core document reading functionality.

## Key Changes Required

### 1. Build System Modifications

- Add iOS target support (aarch64-apple-ios, potentially x86_64-apple-ios for simulator)
- Configure Cargo.toml for iOS dependencies
- Set up proper linking for iOS frameworks (UIKit, Metal, CoreText, etc.)
- Adjust build scripts for iOS toolchain

### 2. Rendering Layer Abstraction

- Replace direct framebuffer access with platform-agnostic rendering interface
- Implement iOS-specific renderer using Metal
- Maintain existing SDL2 renderer for emulator/desktop
- Keep existing framebuffer renderer for Kobo devices

### 3. Input Handling

- Abstract input events from platform-specific implementations
- Map iOS touch events to Plato's input system
- Handle multi-touch, gestures (pinch to zoom, etc.)
- Adapt button handling (Kobo has physical buttons, iOS uses touch)

### 4. Platform Specific Code

- Replace CURRENT_DEVICE mechanism with iOS-specific device detection
- Adapt file system access for iOS sandboxing
- Handle iOS app lifecycle events (applicationDidEnterBackground, etc.)
- Implement proper permissions handling (photo library, files, etc.)

### 5. Dependencies Assessment

- Verify PDFPurr, skrifa, rustybuzz support iOS
- Check SDL2 compatibility with iOS (may need to replace)
- Review all dependencies for iOS support

### 6. UI Adaptations

- Adapt UI layouts for touch interface
- Implement touch-friendly controls
- Consider different screen sizes and resolutions (iPhone vs iPad)
- Support for rotation and multitasking (split view on iPad)

### 7. Testing Strategy

- Unit tests should remain mostly unchanged
- Integration tests need iOS-specific adaptations
- Manual testing on actual devices
- Consider using Xcode for UI testing

## Detailed Implementation Plan

### Phase 1: Foundation

- Set up iOS build environment
- Create platform abstraction layer in plato-core
- Implement basic iOS render target
- Get core library building for iOS

### Phase 2: Rendering

- Implement Metal-based renderer
- Adapt RenderQueue to work with Metal
- Test basic document rendering

### Phase 3: Input and UI

- Implement touch event handling
- Adapt UI components for touch
- Implement basic navigation

### Phase 4: Platform Integration

- Handle file system access
- Implement app lifecycle management
- Add iOS-specific features (share sheet, etc.)

### Phase 5: Polish and Testing

- Performance optimization
- Battery usage considerations
- App Store preparation
- Comprehensive testing

## Risks and Mitigations

- **Rendering performance**: Metal should provide good performance; profile early
- **App Store compliance**: Review guidelines early in development
- **Dependency issues**: Have fallback plans for any incompatible dependencies
- **UI/UX differences**: Design touch interface from ground up, don't just port Kobo UI

## Success Criteria

- Plato core functionality works on iPhone and iPad
- Documents render correctly with good performance
- Touch interface is intuitive and responsive
- App meets Apple's App Store guidelines
- Existing Kobo functionality remains unaffected

## Open Questions

1. Should we maintain a single codebase with platform flags, or create separate iOS-specific branches?
2. How much of the UI should be in Rust vs Swift?
3. What level of iOS integration is desired (share extensions, file provider, etc.)?
