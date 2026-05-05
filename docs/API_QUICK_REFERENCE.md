# Plato API Quick Reference

Fast lookup guide for common tasks.

---

## By Task

### I want to...

| Goal | Module | Key Type | Example |
|------|--------|----------|---------|
| **Detect device model** | `device` | `CURRENT_DEVICE`, `Model` | `CURRENT_DEVICE.model()` |
| **Get screen resolution** | `device` | `Device` trait | `CURRENT_DEVICE.dims()` |
| **Render to screen** | `framebuffer` | `Display` | `fb.update(&pixmap, rect, mode)?` |
| **Handle touch input** | `input` | `TouchProto` | `CURRENT_DEVICE.proto()` |
| **Recognize gestures** | `gesture` | `GestureRec` | `recognizer.push(contact, time)?` |
| **Open a document** | `document` | `Document` trait | `PdfDocument::new(&data)?` |
| **Load settings** | `settings` | `Settings` | `Settings::load()?` |
| **Access library** | `library` | `Library` | `Library::new()?.books()` |
| **Control frontlight** | `frontlight` | `Frontlight` trait | `fb.set_intensity(0.5)?` |
| **Get battery status** | `battery` | `Battery` trait | `battery.capacity_level()` |
| **Extract text** | `document` | Document methods | `doc.outline()` |
| **Search library** | `library` | `Library` | `library.search("query")?` |
| **Sync documents** | `sync` | `SyncService` trait | `sync_service.upload(path)?` |
| **Speak text** | `tts` | `TextToSpeech` | `tts.speak(text, voice)?` |

---

## By Module

### Core/Error

```rust
use plato_core::{PlatoResult, PlatoError};

let result: PlatoResult<()> = Ok(());
```

### Device

```rust
use plato_core::{CURRENT_DEVICE, Device};

let model = CURRENT_DEVICE.model();
let (w, h) = CURRENT_DEVICE.dims();
```

### Framebuffer

```rust
use plato_core::framebuffer::{Display, UpdateMode};

let mut fb = KoboFramebuffer2::new()?;
fb.update(&pixmap, rect, UpdateMode::Partial)?;
```

### Geometry

```rust
use plato_core::geom::{Point, Rectangle};

let rect = Rectangle::new(Point::new(0, 0), Point::new(100, 100));
```

### Color

```rust
use plato_core::color::Color;

let white = Color::white();
let black = Color::black();
```

### Input

```rust
use plato_core::input::{InputEvent, TouchEvent};

// Handle input events
```

### Gesture

```rust
use plato_core::gesture::GestureRec;

let mut rec = GestureRec::new();
if let Some(gesture) = rec.push(contact, time)? { }
```

### Document

```rust
use plato_core::document::{Document, PdfDocument};

let doc = PdfDocument::new(&data)?;
let pages = doc.pages_count();
```

### Settings

```rust
use plato_core::settings::Settings;

let mut settings = Settings::load()?;
settings.display.frontlight_level = 0.8;
settings.save()?;
```

### Library

```rust
use plato_core::library::Library;

let lib = Library::new()?;
let books = lib.books();
```

### Metadata

```rust
use plato_core::metadata::Metadata;

let meta = Metadata::from_document(&data, path)?;
println!("{}", meta.title);
```

### Theme

```rust
use plato_core::theme::Theme;

let theme = Theme::load()?;
```

### Font

```rust
use plato_core::font::{Fonts, Family};

let fonts = Fonts::new()?;
let font = fonts.open(Family::Serif, Style::Regular)?;
```

### Battery

```rust
use plato_core::battery::Battery;

let level = battery.capacity_level();  // 0-100
let charging = battery.is_charging();
```

### Frontlight

```rust
use plato_core::frontlight::Frontlight;

fb.set_intensity(0.5)?;
let intensity = fb.intensity();
```

### Light Sensor

```rust
use plato_core::lightsensor::LightSensor;

let lux = sensor.lux();  // Illuminance
```

### RTC

```rust
use plato_core::rtc::RealTimeClock;

let now = rtc.now();
```

### Network

```rust
use plato_core::network::Network;

if net.is_online() {
    println!("Connected");
}
```

### Sync

```rust
use plato_core::sync::SyncService;

service.upload(path)?;
service.download(remote, local)?;
```

### OPDS

```rust
use plato_core::opds::OpdsClient;

let catalogs = client.list_catalogs()?;
```

### Pocket

```rust
use plato_core::pocket::PocketClient;

let articles = pocket.get_articles()?;
```

### Instapaper

```rust
use plato_core::instapaper::InstapaperClient;

// Similar to Pocket API
```

### TTS

```rust
#[cfg(feature = "tts")]
use plato_core::tts::{TextToSpeech, Voice};

let tts = TextToSpeech::new()?;
tts.speak("Hello", voice)?;
```

### UI/Views

```rust
use plato_core::view::{View, Event, Hub};

impl View for MyView { }
```

### Plugin

```rust
use plato_core::plugin::PluginManager;

manager.load(path)?;
manager.call(name, args)?;
```

---

## Error Codes

| Variant | Meaning | Recovery |
|---------|---------|----------|
| `Io(e)` | File or system I/O error | Check file path and permissions |
| `InvalidFileFormat(fmt)` | Unsupported file format | Use supported format (.pdf, .epub, etc) |
| `MemoryError` | Memory allocation failed | Reduce buffer size, free memory |
| `WordNotFound(word)` | Word not in dictionary | Use alternative word |
| `InvalidCharacter(c)` | Invalid UTF-8 character | Check file encoding |
| `DeflateError(e)` | Decompression failed | File may be corrupted |
| `Database(e)` | Database operation failed | Check database state |
| `Ai(e)` | General error from dependency | Check underlying error |
| `Config(msg)` | Invalid configuration | Check settings file syntax |
| `Battery(e)` | Battery API error | Device may not support feature |
| `Document(e)` | Document processing error | Check document format |
| `Plugin(e)` | Plugin error | Check plugin compatibility |
| `Pdf(e)` | PDF processing error | Check PDF validity |

---

## Feature Flags

```toml
[features]
# Desktop TTS support (Linux/macOS/Windows)
tts = ["tts"]

# Android TTS support
tts-android = ["android"]

# Android build
android = []

# iOS build
ios = []

# Desktop build
desktop = []
```

---

## Platform Support

| Crate | Kobo | Android | iOS | Desktop |
|-------|------|---------|-----|---------|
| `core` | ✓ | ✓ | ✓ | ✓ |
| `framebuffer` | ✓ | ✓ | ✓ | ✓ |
| `eink` | ✓ | - | - | - |
| `tts` | - | ✓ | ✓ | ✓ |
| `plato-android` | - | ✓ | - | - |
| `plato-ios` | - | - | ✓ | - |
| `device` | ✓ | - | - | ✓ |
| `network` | ✓ | ✓ | ✓ | ✓ |

---

## Common Patterns

### Pattern: Device Detection

```rust
match CURRENT_DEVICE.model() {
    Model::Clara2E => { /* Clara 2E specific */ }
    Model::Libra2 => { /* Libra 2 specific */ }
    _ => { /* Generic fallback */ }
}
```

### Pattern: Error Handling

```rust
match operation() {
    Ok(value) => println!("Success: {:?}", value),
    Err(PlatoError::Io(e)) => eprintln!("I/O error: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Pattern: Resource Cleanup

```rust
{
    let resource = open_resource()?;
    use_resource(&resource)?;
    // Automatically dropped here
}
```

### Pattern: View Implementation

```rust
impl View for MyView {
    fn draw(&mut self, fb: &mut Display, rect: Rectangle) -> PlatoResult<()> {
        // Draw implementation
        Ok(())
    }
    
    fn handle(&mut self, event: &Event, hub: &Hub) -> PlatoResult<bool> {
        match event {
            Event::Touch(_) => Ok(true),
            _ => Ok(false),
        }
    }
    
    fn rect(&self) -> Rectangle { self.rect }
}
```

---

## Memory Limits

| Target | Available | Recommendation |
|--------|-----------|-----------------|
| Kobo (256MB) | ~180MB | <50MB buffers |
| Android | 512MB+ | <100MB buffers |
| iOS | 1GB+ | <200MB buffers |
| Desktop | System | <500MB buffers |

---

## Performance Tips

1. **Use `UpdateMode::Partial`** for fast updates (text, UI)
2. **Use `UpdateMode::Full`** for page transitions (smoother)
3. **Cache rendered pages** to avoid re-rendering
4. **Pre-allocate buffers** with `Vec::with_capacity()`
5. **Batch framebuffer updates** when possible
6. **Unload unused documents** to free memory
7. **Use thumbnail caching** for library browsing

---

## Debugging

### Enable Logging

```rust
#[cfg(debug_assertions)]
eprintln!("Debug info: {:?}", value);
```

### Test Mocks

```rust
use plato_core::test_mocks::{MockFramebuffer, MockBattery};

let mut fb = MockFramebuffer::new();
// Test without hardware
```

### Clippy Checks

```bash
cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings
```

---

## Related Files

- **Full API Reference**: `docs/API.md`
- **Integration Guide**: `docs/API_INTEGRATION_GUIDE.md`
- **Development Guide**: `AGENTS.md`
- **Contribution Guide**: `CONTRIBUTING.md`
- **Project README**: `README.md`

---

## Quick Start Template

```rust
use plato_core::{PlatoResult, CURRENT_DEVICE, framebuffer::*, document::*};

fn main() -> PlatoResult<()> {
    // 1. Detect device
    let device = &*CURRENT_DEVICE;
    println!("Device: {}", device.model());
    
    // 2. Initialize framebuffer
    let mut fb = KoboFramebuffer2::new()?;
    
    // 3. Load document
    let data = std::fs::read("document.pdf")?;
    let mut doc = PdfDocument::new(&data)?;
    
    // 4. Render page
    let rect = Rectangle::full();
    let pixmap = doc.render(0, rect)?;
    
    // 5. Update display
    fb.update(&pixmap, rect, UpdateMode::Full)?;
    
    Ok(())
}
```

