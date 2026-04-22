# Plan to Support iOS and Android in Plato

## Overview

This document provides a comprehensive plan to extend Plato from Kobo e-readers to iOS and Android mobile platforms. The plan uses a single codebase with conditional compilation (`#[cfg()]`) and replaces C/C++ dependencies with Rust-native alternatives to enable fully open-source distribution via sideloading (no proprietary libraries requiring App Store/Play Store compliance).

## Architecture Summary

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Application Code                          │
│  (plato-core: document handling, UI, library, settings)          │
├─────────────────────────────────────────────────────────────────┤
│                    Platform Abstraction Layer                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │  Renderer   │  │    Input    │  │   Platform  │            │
│  │   Trait     │  │    Trait    │  │   Services  │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
├─────────────┬─────────────┬─────────────┬─────────────┬────────┤
│   Kobo      │    iOS      │   Android   │  Emulator   │ Tests   │
│  (framebuf) │  (Metal/    │ (Vulkan/    │   (SDL2)    │ (mock)  │
│             │  softbuf)   │ softbuf)    │             │         │
└─────────────┴─────────────┴─────────────┴─────────────┴────────┘
```

## Key Design Decisions

| Decision                        | Rationale                                           |
|---------------------------------|-----------------------------------------------------|
| Single codebase with `#[cfg()]` | Existing patterns in Plato; simpler than multi-repo |
| Native Rust rendering           | Full control, no SDL2 dependency, true open source  |
| Rust-native alternatives        | Avoids proprietary blobs, enables sideloading       |
| Runtime polymorphism (traits)   | Already used in Plato; allows platform switching    |

---

## Phase 1: Core Architecture & Abstraction

### 1.1 Expand Platform Abstraction Layer

**Location:** `crates/core/src/`

| Module               | Changes                                                                                         |
|----------------------|-------------------------------------------------------------------------------------------------|
| `device.rs`          | Add `Platform` enum: `Kobo`, `IOS`, `Android`; replace `CURRENT_DEVICE` with `CURRENT_PLATFORM` |
| `framebuffer/mod.rs` | Extend `Framebuffer` trait with mobile-specific methods                                         |
| `context.rs`         | Make platform services generic via trait objects                                                |

**New module:** `crates/core/src/platform/mod.rs`

```rust
pub trait PlatformServices {
    fn framebuffer(&self) -> Box<dyn Framebuffer>;
    fn input(&self) -> Box<dyn Input>;
    fn battery(&self) -> Box<dyn Battery>;
    fn filesystem(&self) -> Box<dyn FileSystem>;
    fn notification(&self) -> Box<dyn Notification>;
}
```

### 1.2 Conditional Compilation Setup

**Location:** `crates/core/src/`

```rust
#[cfg(target_os = "linux")]
pub mod platform { /* Kobo implementation */ }

#[cfg(target_os = "ios")]
pub mod platform { /* iOS implementation */ }

#[cfg(target_os = "android")]
pub mod platform { /* Android implementation */ }

#[cfg(feature = "emulator")]
pub mod platform { /* SDL2 implementation */ }
```

Add to `Cargo.toml`:

```toml
[target.'cfg(not(any(target_os = "ios", target_os = "android")))'.dependencies]
sdl2 = "0.37"

[target.'cfg(target_os = "ios")'.dependencies]
metal = "0.3"
ios = "0.1"

[target.'cfg(target_os = "android")'.dependencies]
vulkan = "0.4"
android = "0.1"
```

---

## Phase 2: Rendering Layer

### 2.1 Graphics Stack Migration

| Platform | Primary | Fallback   | Crates                          |
|----------|---------|------------|---------------------------------|
| iOS      | Metal   | softbuffer | `metal`, `wgpu`, `softbuffer`   |
| Android  | Vulkan  | softbuffer | `vulkano`, `wgpu`, `softbuffer` |

### 2.2 Implement Mobile Renderers

**New file:** `crates/core/src/renderer/mod.rs`

```rust
pub trait Renderer {
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn draw_rect(&mut self, rect: Rectangle, color: Color);
    fn draw_text(&mut self, text: &str, position: Point, font: &Font);
    fn draw_image(&mut self, image: &Image);
}

#[cfg(target_os = "ios")]
pub struct MetalRenderer { /* ... */ }

#[cfg(target_os = "android")]
pub struct VulkanRenderer { /* ... */ }
```

### 2.3 Integrate with RenderQueue

Existing `RenderQueue` in `view/mod.rs` remains; add platform-specific dispatch:

```rust
impl RenderQueue {
    fn flush_to_platform(&mut self, fb: &mut dyn Framebuffer) {
        #[cfg(target_os = "ios")]
        self.flush_metal(fb);

        #[cfg(target_os = "android")]
        self.flush_vulkan(fb);

        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        self.flush_framebuffer(fb);
    }
}
```

---

## Phase 3: Input Handling

### 3.1 Abstract Input System

**Location:** `crates/core/src/input.rs`

```rust
pub trait InputSource {
    fn poll_events(&mut self) -> Vec<InputEvent>;
}

#[cfg(target_os = "ios")]
pub struct IosInputSource { /* UIEvent handling */ }

#[cfg(target_os = "android")]
pub struct AndroidInputSource { /* MotionEvent handling */ }
```

### 3.2 Touch & Gesture Support

| Feature    | Implementation         |
|------------|------------------------|
| Single tap | Map to tap event       |
| Long press | Map to context menu    |
| Pinch zoom | Scale document view    |
| Swipe      | Page turn / scroll     |
| Drag       | Selection / annotation |

**New module:** `crates/core/src/gesture/mod.rs`

- Extend existing `GestureEvent` enum for mobile gestures
- Add gesture recognizer for iOS/Android touch patterns

---

## Phase 4: Platform Services

### 4.1 iOS Services (Swift interop via C ABI)

| Service       | Implementation                  |
|---------------|---------------------------------|
| File access   | `NSFileManager` via `ffi_rs`    |
| Battery       | `UIDevice.current.batteryLevel` |
| Notifications | `UNUserNotificationCenter`      |
| Haptics       | `UIFeedbackGenerator`           |

**New crate:** `crates/plato-ios/`

- Swift UI layer for iOS-specific features
- Rust core embedded via C bridging

### 4.2 Android Services (JNI)

| Service       | Implementation              |
|---------------|-----------------------------|
| File access   | `ContentResolver` via `jni` |
| Battery       | `BatteryManager`            |
| Notifications | `NotificationManager`       |
| Haptics       | `Vibrator`                  |

**New crate:** `crates/plato-android/`

- Kotlin UI layer for Android-specific features
- Rust core embedded via JNI

---

## Phase 5: Library Migration (Rust-Native)

### 5.1 Font Stack Replacement

| Current        | Replacement               | Status           |
|----------------|---------------------------|------------------|
| `freetype_sys` | `skrifa` + `ab_glyph`     | Production-ready |
| `harfbuzz_sys` | `harfrust` or `rustybuzz` | Production-ready |

**Migration:**

```rust
// Old (C FFI)
use freetype_sys::*;

// New (Pure Rust)
use skrifa::FontRef;
use rustybuzz::UnicodeMapping;
```

### 5.2 Graphics Replacement

| Current         | Replacement            | Status           |
|-----------------|------------------------|------------------|
| SDL2 (emulator) | `softbuffer` or `wgpu` | Production-ready |

### 5.3 PDF Handling

| Current               | Replacement                | Status                |
|-----------------------|----------------------------|-----------------------|
| `mupdf_sys` + wrapper | `hayro` or `pdfium-render` | Partial (limitations) |

**Note:** Pure Rust PDF libraries lack some MuPDF features. For production:

- **Option A:** Keep MuPDF (LGPL) for PDF, use Rust-native for other formats
- **Option B:** Use `hayro` with known limitations (no encrypted PDFs)
- **Option C:** Accept feature gaps for full open-source stack

### 5.4 Dependency Updates

**`Cargo.toml` additions:**

```toml
[dependencies]
# Text rendering (replace FreeType + HarfBuzz)
skrifa = "0.42.0"
rustybuzz = "0.20"
ab_glyph = "0.2"

# Graphics (replace SDL2)
softbuffer = "0.12"
wgpu = "23"

# PDF (replace MuPDF - optional)
hayro = "0.5"

[target.'cfg(not(any(target_os = "ios", target_os = "android")))'.dependencies]
sdl2 = "0.37"
```

---

## Phase 6: Build System

### 6.1 iOS Build Configuration

```bash
# Target
aarch64-apple-ios

# Toolchain
rustup target add aarch64-apple-ios

# Build
cargo build --target aarch64-apple-ios -p plato-core
```

### 6.2 Android Build Configuration

```bash
# Target
aarch64-linux-android

# Toolchain
rustup target add aarch64-linux-android

# Build
cargo build --target aarch64-linux-android -p plato-core
```

### 6.3 Native Library Build Scripts

**`build-mobile.sh`:**

```bash
#!/bin/bash
case "$1" in
    ios)
        cd mupdf_wrapper && TARGET_OS=iOS ./build.sh
        ;;
    android)
        cd mupdf_wrapper && TARGET_OS=Android ./build.sh
        ;;
esac
```

---

## Phase 7: UI Adaptations

### 7.1 Touch-Optimized Interface

| Component    | Changes                             |
|--------------|-------------------------------------|
| Navigation   | Larger touch targets (48dp minimum) |
| Menu system  | Swipe gestures instead of tap       |
| Reading view | Tap zones for page turn             |
| Settings     | Scrollable lists with clear labels  |

### 7.2 Responsive Layouts

- Support portrait/landscape rotation
- Handle notch/dynamic island
- Adaptive layout for tablet (iPad/Android tablets)

---

## Phase 8: Testing Strategy

### 8.1 Unit Tests

Existing tests in `crates/core/src/*_tests.rs` remain unchanged - they use mocks.

### 8.2 Platform Tests

| Platform | Test Type   | Tool                    |
|----------|-------------|-------------------------|
| iOS      | UI          | Xcode + XCTest          |
| Android  | UI          | Espresso + Compose Test |
| Both     | Integration | CI emulator runs        |

### 8.3 Mock Implementations

**`crates/core/src/test_mocks.rs`** - Extend with:

- `MockFramebuffer` (already exists)
- `MockInputSource`
- `MockPlatformServices`

---

## Phase 9: Risks & Mitigations

| Risk                           | Impact | Mitigation                                   |
|--------------------------------|--------|----------------------------------------------|
| PDF feature gaps               | Medium | Keep MuPDF as optional; document limitations |
| Graphics performance           | Medium | Use wgpu for GPU; softbuffer fallback        |
| iOS App Store rejection        | High   | Sideload only; no App Store distribution     |
| Android Play Store rejection   | Medium | Sideload only; F-Droid compatible            |
| Rust mobile ecosystem maturity | Low    | Mature crates available (wgpu, skrifa)       |

---

## Phase 10: Implementation Timeline

| Phase                | Duration      | Deliverable                |
|----------------------|---------------|----------------------------|
| 1. Architecture      | 2 weeks       | Platform abstraction layer |
| 2. Rendering         | 3 weeks       | Metal/Vulkan renderers     |
| 3. Input             | 2 weeks       | Touch/gesture handling     |
| 4. Platform Services | 3 weeks       | iOS/Android services       |
| 5. Library Migration | 4 weeks       | Rust-native替换            |
| 6. Build System      | 1 week        | iOS/Android builds         |
| 7. UI Adaptations    | 2 weeks       | Touch-optimized UI         |
| 8. Testing           | 2 weeks       | Platform tests             |
| **Total**            | **~19 weeks** | Full mobile support        |

---

## Open Questions

1. **PDF limitations**: Accept `hayro` limitations (no encrypted PDFs) or keep MuPDF for PDFs?
2. **Distribution**: Target only sideloading (F-Droid, AltStore) or attempt App Store/Play Store?
3. **UI ownership**: How much UI in Swift/Kotlin vs pure Rust?
4. **Feature parity**: Which Kobo features (frontlight, buttons) to port or skip?

---

This plan provides a complete roadmap for iOS and Android support with full open-source dependencies.

---
