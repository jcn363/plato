# Plato API Documentation

Comprehensive reference for all 46 crates in the Plato e-reader ecosystem.

**Last Updated**: May 5, 2026  
**Plato Version**: 0.9.45+  
**Rust Edition**: 2021

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Error Handling](#error-handling)
3. [Core Types & Utilities](#core-types--utilities)
4. [Hardware Abstraction](#hardware-abstraction)
5. [Document Handling](#document-handling)
6. [Display & Rendering](#display--rendering)
7. [UI & Views](#ui--views)
8. [Library & Metadata](#library--metadata)
9. [Settings & Configuration](#settings--configuration)
10. [Network & Sync](#network--sync)
11. [Plugins & Extensions](#plugins--extensions)
12. [Platform-Specific APIs](#platform-specific-apis)
13. [Utilities & Helpers](#utilities--helpers)

---

## Quick Start

### Adding Plato to Your Project

```toml
[dependencies]
plato-core = { path = "../crates/core" }
plato-error = { path = "../crates/error" }
```

### Basic Usage Pattern

```rust
use plato_core::{Device, CURRENT_DEVICE, PlatoResult};

fn main() -> PlatoResult<()> {
    let device = &*CURRENT_DEVICE;
    let (width, height) = device.dims();
    println!("Device: {:?}, Resolution: {}x{}", 
             device.model(), width, height);
    Ok(())
}
```

---

## Error Handling

### `crates/error` - Core Error Types

**Purpose**: Unified error representation for the entire Plato ecosystem.

#### Key Types

- **`PlatoError`** - Main error enum with variants for:
  - `Io(io::Error)` - File I/O and system errors
  - `InvalidCharacter(char, Option<usize>, Option<usize>)` - Invalid UTF-8/characters
  - `InvalidFileFormat(String, Option<String>)` - Unsupported file format
  - `MemoryError` - Memory allocation failures
  - `WordNotFound(String)` - Dictionary/search failures
  - `Utf8Error(FromUtf8Error)` - UTF-8 decoding
  - `DeflateError(String)` - Compression errors
  - `Format(String)` - Generic format errors
  - `Database(String)` - Database operation failures
  - `Ai(anyhow::Error)` - Catch-all for external errors
  - `Config(String)` - Configuration errors
  - `Battery(String)` - Battery/power errors
  - `Document(String)` - Document processing errors
  - `Plugin(String)` - Plugin loading/execution errors
  - `Pdf(String)` - PDF processing errors
  - `Unknown` - Unknown/unclassified errors

- **`PlatoResult<T>`** - Type alias for `Result<T, PlatoError>`

#### Helper Functions

```rust
pub fn into_plato_err<E>(e: E) -> PlatoError
where
    E: std::error::Error + Send + Sync + 'static
```

Converts any standard error to `PlatoError::Ai`.

#### Error Handling Guidelines

```rust
use plato_core::{PlatoResult, PlatoError};

// Use ? operator for automatic conversion
fn process_file(path: &str) -> PlatoResult<String> {
    std::fs::read_to_string(path)
        .map_err(PlatoError::from)
}

// Explicit error creation
fn custom_error() -> PlatoResult<()> {
    Err(PlatoError::Config("Invalid setting".to_string()))
}
```

---

## Core Types & Utilities

### `crates/geom` - Geometry Types

**Purpose**: 2D geometry primitives for layout and rendering.

#### Key Types

- **`Point`** - 2D point with `x`, `y` coordinates
- **`Vec2`** - 2D vector for relative positions
- **`Rectangle`** - Axis-aligned rectangular region
- **`Region`** - Collection of rectangles
- **`Boundary`** - Axis-aligned bounding box

#### Key Enums

- **`Dir`** - Cardinal direction (N, S, E, W)
- **`LinearDir`** - Linear directions (Forward, Backward)
- **`DiagDir`** - Diagonal directions (NE, SE, SW, NW)
- **`Axis`** - Horizontal or Vertical axis
- **`CycleDir`** - Cycle direction (Forward, Backward)

#### Constants

```rust
pub const SQRT2: f32;      // √2
pub const SQRT2_INV: f32;  // 1/√2
```

#### Example

```rust
use plato_core::geom::{Point, Rectangle};

let rect = Rectangle::new(Point::new(10, 20), Point::new(100, 200));
let area = (rect.width() * rect.height()) as usize;
```

### `crates/color` - Color Types

**Purpose**: Color representation and manipulation for e-ink displays.

#### Key Types

- **Color spaces**: RGB, HSV, indexed palettes
- **Dithering**: Support for 16-color, 256-color e-ink displays
- **Color conversion**: RGB ↔ Grayscale, Color reduction

### `crates/consts` - Constants

**Purpose**: Global constants for Plato.

Contains:
- Device model identifiers
- Display resolutions
- Default paths and configurations
- UI timing constants
- Memory limits

### `crates/buffer_pool` - Memory Pool Management

**Purpose**: Pre-allocated buffer pool to reduce allocation overhead.

```rust
pub trait BufferPool {
    fn get(&mut self, size: usize) -> Vec<u8>;
    fn put(&mut self, buffer: Vec<u8>);
}
```

---

## Hardware Abstraction

### `crates/device` - Device Models & Hardware Detection

**Purpose**: Hardware abstraction for different Kobo e-reader models.

#### Device Models

```rust
pub enum Model {
    // Color displays
    LibraColour,
    ClaraColour,
    Elipsa2E,
    
    // High-res B&W
    Clara2E,
    Libra2,
    Sage,
    ClaraHD,
    
    // Standard models
    Nia,
    AuraH2O,
    Touch2,
    GloHD,
    // ... and 25+ more
}
```

#### Device Trait

```rust
pub trait Device {
    fn model(&self) -> Model;
    fn dims(&self) -> (u32, u32);           // Width, height
    fn dpi(&self) -> u16;                   // Pixels per inch
    fn color_samples(&self) -> usize;       // Color depth (1, 2, 4, 8)
    fn frontlight_kind(&self) -> FrontlightKind;
    fn has_natural_light(&self) -> bool;
    fn has_lightsensor(&self) -> bool;
    fn has_gyroscope(&self) -> bool;
    fn has_page_turn_buttons(&self) -> bool;
    fn proto(&self) -> TouchProto;          // Touch protocol version
    fn orientation(&self, rotation: i8) -> Orientation;
}
```

#### Global Device Instance

```rust
pub static CURRENT_DEVICE: LazyLock<Box<dyn Device>>;
```

Automatically detects the running device.

#### Frontlight Types

```rust
pub enum FrontlightKind {
    Standard,           // Simple on/off
    Natural,            // RGB with temperature
    Powerled,           // High-power LED
}
```

### `crates/framebuffer` - Display Framebuffer

**Purpose**: Hardware framebuffer abstraction for rendering.

#### Key Types

- **`Display`** - Main framebuffer trait
- **`Pixmap`** - Drawable canvas
- **`UpdateMode`** - Display update strategy
  - `Partial` - Fast partial refresh
  - `Full` - Complete refresh (slower)
  - `Fast` - Very fast partial (lower quality)

#### Implementations

- **`KoboFramebuffer1`** - Kobo Gen 1 devices
- **`KoboFramebuffer2`** - Kobo Gen 2+ devices  
- **`SoftwareFramebuffer`** - CPU-based rendering
- **`DesktopFramebuffer`** - Linux desktop (non-ARM)

#### Example

```rust
use plato_core::framebuffer::{Display, UpdateMode};

let fb = KoboFramebuffer2::new()?;
fb.update(&pixmap, Rectangle::full(), UpdateMode::Partial)?;
```

### `crates/eink` - E-ink Display Modes

**Purpose**: E-ink-specific display algorithms (dithering, halftoning).

#### Key Functions

- Color quantization for 4-bit/8-bit displays
- Floyd-Steinberg dithering
- Error diffusion algorithms
- Ghosting reduction

### `crates/input` - Input Device Handling

**Purpose**: Touch and button input processing.

#### Touch Protocols

```rust
pub enum TouchProto {
    A,      // Kobo's proprietary protocol
    B,      // Enhanced version
    C,      // Latest version with gestures
}
```

#### Key Types

- **`FingerStatus`** - Current touch state
- **`Contact`** - Single finger contact
- **`GestureEvent`** - Recognized gestures

#### Input Events

```rust
pub enum InputEvent {
    Touch(TouchEvent),
    Button(ButtonEvent),
    Custom(u16),  // Device-specific
}
```

### `crates/gesture` - Gesture Recognition

**Purpose**: High-level gesture recognition from touch input.

#### Recognized Gestures

- Tap
- Double-tap
- Long-press
- Swipe (4 directions)
- Pinch
- Two-finger tap
- Corner taps

#### Example

```rust
use plato_core::gesture::{GestureEvent, GestureRec};

let mut recognizer = GestureRec::new();
let gesture = recognizer.push(contact, time)?;
```

### `crates/battery` - Battery & Power Management

**Purpose**: Battery status monitoring and power management.

#### Key Traits

```rust
pub trait Battery {
    fn status(&self) -> BatteryStatus;
    fn capacity_level(&self) -> u8;        // 0-100
    fn is_charging(&self) -> bool;
    fn is_powered(&self) -> bool;
}
```

#### Battery Status

```rust
pub enum BatteryStatus {
    Charging,
    Discharging,
    NotCharging,
    Unknown,
}
```

### `crates/frontlight` - Frontlight Control

**Purpose**: Manage device frontlight/backlight.

#### Key Trait

```rust
pub trait Frontlight {
    fn intensity(&self) -> f32;             // 0.0 = off, 1.0 = max
    fn set_intensity(&mut self, intensity: f32) -> PlatoResult<()>;
    fn is_natural(&self) -> bool;
    fn temperature(&self) -> Option<u16>;   // Color temperature in K
}
```

### `crates/lightsensor` - Ambient Light Sensor

**Purpose**: Ambient light sensing for auto-frontlight adjustment.

```rust
pub trait LightSensor {
    fn lux(&self) -> f32;   // Illuminance in lux
}
```

### `crates/rtc` - Real-time Clock

**Purpose**: RTC (real-time clock) access.

```rust
pub trait RealTimeClock {
    fn now(&self) -> SystemTime;
    fn set(&mut self, time: SystemTime) -> PlatoResult<()>;
}
```

---

## Document Handling

### `crates/doc` - Document Formats

**Purpose**: Unified document interface for multiple formats.

#### Supported Formats

- **PDF** - Via `pdfpurr` (pure Rust, replaces MuPDF)
- **EPUB** - Via `epub` crate
- **DJVU** - Via `djvu-rs`
- **Comic** - CBZ/CBR support
- **HTML** - Via `html` module
- **Plain text** - UTF-8 text files

#### Document Trait

```rust
pub trait Document {
    fn pages_count(&self) -> usize;
    fn page_dims(&self, index: usize) -> (f32, f32);
    fn render(&mut self, index: usize, rect: Rectangle) -> PlatoResult<Pixmap>;
    fn outline(&self) -> Option<Vec<OutlineItem>>;
    fn is_reflowable(&self) -> bool;
}
```

#### Document Types

```rust
pub enum DocumentKind {
    Pdf(PdfDocument),
    Epub(EpubDocument),
    Djvu(DjvuDocument),
    Comic(ComicDocument),
    Html(HtmlDocument),
    Text(TextDocument),
}
```

### `crates/core/src/document/pdf` - PDF Handling

**Pure Rust PDF rendering using `pdfpurr`** (no C dependencies).

#### Key Types

```rust
pub struct PdfDocument {
    // Internal pdfpurr document
}

impl Document for PdfDocument {
    // Implementation details
}
```

#### Features

- Text extraction
- Annotation support
- Form field handling
- Embedded font rendering

### `crates/core/src/document/epub` - EPUB Handling

**EPUB 2 and EPUB 3 support**.

#### Key Functions

- Reflow rendering (text adapts to page size)
- Table of contents extraction
- Metadata access

### `crates/font` - Font Management

**Purpose**: Font loading and text rendering.

#### Font Loading

```rust
pub struct Fonts {
    // Font collection
}

impl Fonts {
    pub fn open(&self, family: FontFamily, style: Style) -> PlatoResult<Font>;
}
```

#### Font Family

```rust
pub enum Family {
    Serif,
    SansSerif,
    Monospace,
    Custom(String),
}
```

#### Text Rendering

```rust
pub struct Font {
    pub fn text_width(&self, text: &str, size: f32) -> f32;
    pub fn draw(&self, text: &str, start: Point, size: f32) -> Vec<Glyph>;
}
```

### `crates/core/src/document/ocr` - Optical Character Recognition

**Optional OCR for scanned documents**.

Uses Tesseract or equivalent.

### `crates/thumbnail` - Document Thumbnails

**Purpose**: Generate and cache document thumbnails.

```rust
pub struct Thumbnail {
    pub fn generate(document: &mut dyn Document) -> PlatoResult<Pixmap>;
    pub fn cache_dir() -> PathBuf;
}
```

### `crates/core/src/document/progressive_loader` - Progressive Loading

**Purpose**: Load large documents progressively for responsiveness.

Supports:
- Page caching
- Background loading
- Memory-efficient streaming

---

## Display & Rendering

### `crates/theme` - UI Theme Management

**Purpose**: Unified theme system for UI elements.

#### Theme Definition

```rust
pub struct Theme {
    pub colors: HashMap<String, Color>,
    pub fonts: HashMap<String, FontSettings>,
    pub spacing: HashMap<String, f32>,
}
```

#### Theme Modes

- Light
- Dark (with reduced flashing on e-ink)
- High-contrast
- Custom user themes

### `crates/mobile_theme` - Mobile Optimizations

**Purpose**: Platform-specific theme adjustments for Android/iOS.

```rust
pub enum MobileThemeMode {
    Light,
    Dark,
    Auto,  // Follow system setting
}

pub fn set_mobile_theme_mode(mode: MobileThemeMode);
```

### `crates/color` - Color Manipulation

**Purpose**: Color space conversions and dithering.

#### Supported Color Spaces

- RGB/RGBA
- HSV
- Grayscale
- Indexed (palette-based)

#### Dithering Algorithms

- Floyd-Steinberg
- Bayer matrix
- Ordered dithering
- Error diffusion

---

## UI & Views

### `crates/ui` / `crates/plato-view` - UI Framework

**Purpose**: Event-driven UI system for Plato.

#### Core Types

```rust
pub trait View {
    fn draw(&mut self, fb: &mut Display, rect: Rectangle) -> PlatoResult<()>;
    fn handle(&mut self, event: &Event, hub: &Hub) -> PlatoResult<bool>;
    fn rect(&self) -> Rectangle;
}
```

#### Event System

```rust
pub enum Event {
    Touch(TouchEvent),
    Button(ButtonEvent),
    Gesture(GestureEvent),
    Custom(CustomEvent),
}

pub struct Hub {
    // Event bus for inter-widget communication
}
```

#### View Hierarchy

- `Window` - Top-level container
- `Panel` - Layout container (horizontal/vertical)
- `Button` - Interactive button
- `Menu` - List selection
- `TextBox` - Text input
- `PageCanvas` - Document rendering

#### Example

```rust
use plato_core::view::{View, Event, Hub};

struct MyView {
    rect: Rectangle,
}

impl View for MyView {
    fn draw(&mut self, fb: &mut Display, rect: Rectangle) -> PlatoResult<()> {
        // Draw implementation
        Ok(())
    }
    
    fn handle(&mut self, event: &Event, hub: &Hub) -> PlatoResult<bool> {
        match event {
            Event::Touch(_) => {
                // Handle touch
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
```

### `crates/core/src/view` - View Utilities

**Purpose**: Helper macros and utilities for view implementation.

#### Key Macros

```rust
impl_view_boilerplate!(MyView);  // Implement common methods
handle_event!(Event::Touch => handle_touch);
```

### `crates/plato-library` - Library View

**Purpose**: Main library/bookshelf interface.

Features:
- Book listing and search
- Cover display
- Category/collection management
- Reading history

### `crates/plato-document` - Document View

**Purpose**: Document viewing interface.

Features:
- Page navigation
- Zoom and rotation
- Text selection
- Annotation

---

## Library & Metadata

### `crates/metadata` - Document Metadata

**Purpose**: Extract and cache document metadata.

#### Metadata Types

```rust
pub struct Metadata {
    pub title: String,
    pub author: String,
    pub publisher: Option<String>,
    pub isbn: Option<String>,
    pub language: Option<String>,
    pub pages: Option<usize>,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}
```

#### Metadata Sources

- EPUB metadata
- PDF document info
- EXIF data
- File properties

### `crates/core/src/library` - Library Management

**Purpose**: Unified library interface.

```rust
pub struct Library {
    // Library implementation
}

impl Library {
    pub fn books(&self) -> Vec<Book>;
    pub fn search(&self, query: &str) -> Vec<Book>;
    pub fn add_book(&mut self, path: &Path) -> PlatoResult<()>;
    pub fn remove_book(&mut self, id: &str) -> PlatoResult<()>;
}
```

### `crates/core/src/article` - Article Management

**Purpose**: Articles from Pocket/Instapaper.

```rust
pub struct Article {
    pub id: String,
    pub title: String,
    pub author: String,
    pub content: String,
    pub url: String,
    pub added_at: DateTime<Utc>,
}
```

---

## Settings & Configuration

### `crates/settings` - Settings Management

**Purpose**: Persistent application settings.

#### Settings Categories

```rust
pub struct Settings {
    pub display: DisplaySettings,
    pub interface: InterfaceSettings,
    pub library: LibrarySettings,
    pub reading: ReadingSettings,
    pub theme: ThemeSettings,
    pub opds: OpdsSettings,
    pub tools: ToolsSettings,
    pub thumbnail: ThumbnailSettings,
}
```

#### Key Settings

**Display Settings**
```rust
pub struct DisplaySettings {
    pub frontlight_level: f32,
    pub frontlight_warmth: Option<f32>,  // Color temperature
    pub invert_colors: bool,
    pub high_contrast: bool,
}
```

**Reading Settings**
```rust
pub struct ReadingSettings {
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub margin: u32,
    pub page_turns_animation: bool,
}
```

**Interface Settings**
```rust
pub struct InterfaceSettings {
    pub font_size: f32,
    pub font_family: String,
    pub dark_mode: bool,
    pub navigation_bar_hidden: bool,
    pub button_delay: u64,  // milliseconds
}
```

#### Settings Persistence

```rust
pub fn load() -> PlatoResult<Settings>;
pub fn save(&self) -> PlatoResult<()>;
pub fn load_from_path(path: &Path) -> PlatoResult<Settings>;
pub fn save_to_path(&self, path: &Path) -> PlatoResult<()>;
```

### `crates/config` - Configuration Files

**Purpose**: TOML-based configuration.

#### Config Format

```toml
[display]
frontlight_level = 0.8
invert_colors = false

[reading]
font_family = "DejaVu Sans"
font_size = 12.0

[library]
sort_by = "title"
```

### `crates/consts` - Built-in Constants

**Purpose**: Compile-time configuration.

Contains default values for:
- Font paths
- Color palettes
- UI dimensions
- Timeouts

---

## Network & Sync

### `crates/network` - Network Interface

**Purpose**: WiFi and network connectivity.

```rust
pub trait Network {
    fn is_online(&self) -> bool;
    fn signal_strength(&self) -> Option<u8>;  // 0-100
    fn scan_networks(&self) -> PlatoResult<Vec<NetworkInfo>>;
    fn connect(&mut self, ssid: &str, password: &str) -> PlatoResult<()>;
}
```

### `crates/sync` - Library Sync

**Purpose**: Synchronize library with cloud services.

#### Sync Services

- Dropbox
- Google Drive
- OneDrive
- WebDAV

```rust
pub trait SyncService {
    fn authenticate(&mut self, token: &str) -> PlatoResult<()>;
    fn list_files(&self) -> PlatoResult<Vec<RemoteFile>>;
    fn upload(&self, local_path: &Path) -> PlatoResult<()>;
    fn download(&self, remote_file: &str, local_path: &Path) -> PlatoResult<()>;
}
```

### `crates/fetcher` - Content Fetching

**Purpose**: Download documents and articles.

```rust
pub struct Fetcher {
    pub fn fetch_url(&self, url: &str) -> PlatoResult<Vec<u8>>;
    pub fn fetch_article(&self, url: &str) -> PlatoResult<Article>;
}
```

### `crates/core/src/pocket` - Pocket Integration

**Purpose**: Pocket article service integration.

```rust
pub struct PocketClient {
    pub fn get_articles(&self) -> PlatoResult<Vec<Article>>;
    pub fn add_article(&self, url: &str, tags: Vec<String>) -> PlatoResult<()>;
    pub fn archive(&self, article_id: &str) -> PlatoResult<()>;
}
```

### `crates/core/src/instapaper` - Instapaper Integration

**Purpose**: Instapaper service integration.

Similar API to Pocket.

### `crates/opds` - OPDS Catalog Support

**Purpose**: Open Publication Distribution System (OPDS) support.

```rust
pub struct OpdsClient {
    pub fn list_catalogs(&self) -> PlatoResult<Vec<Catalog>>;
    pub fn list_books(&self, catalog: &str) -> PlatoResult<Vec<Book>>;
    pub fn download(&self, book: &Book) -> PlatoResult<PathBuf>;
}
```

---

## Plugins & Extensions

### `crates/plugin` - Plugin System

**Purpose**: Extensibility through plugins.

#### Plugin Trait

```rust
pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn execute(&self, args: Vec<String>) -> PlatoResult<String>;
}
```

#### Plugin Loading

```rust
pub struct PluginManager {
    pub fn load(&mut self, path: &Path) -> PlatoResult<()>;
    pub fn unload(&mut self, name: &str) -> PlatoResult<()>;
    pub fn call(&self, name: &str, args: Vec<String>) -> PlatoResult<String>;
    pub fn list(&self) -> Vec<String>;
}
```

---

## Platform-Specific APIs

### `crates/plato-android` - Android Integration

**Purpose**: Android native bridge for Plato.

Features:
- JNI bindings to Android APIs
- Text-to-speech integration
- File access via SAF (Storage Access Framework)
- Notification support

### `crates/plato-ios` - iOS Integration

**Purpose**: iOS native bridge for Plato.

Features:
- Swift interop via objc
- iOS text-to-speech
- iCloud Drive integration
- App lifecycle management

### `crates/mobile_optimizations` - Mobile Tuning

**Purpose**: Platform-independent mobile optimizations.

```rust
pub fn memory_config() -> MemoryConfig;
pub fn network_config() -> NetworkConfig;
pub fn power_config() -> PowerConfig;
pub fn storage_config() -> StorageConfig;
pub fn animation_config() -> AnimationConfig;
pub fn recommended_thread_pool_size() -> usize;
pub fn recommended_io_buffer_size() -> usize;

pub fn is_mobile_platform() -> bool;
```

---

## Utilities & Helpers

### `crates/helpers` - Common Utilities

**Purpose**: Cross-cutting utility functions.

#### Categories

- Path manipulation
- File operations
- String utilities
- Date/time formatting
- Number formatting

### `crates/utils` - Low-level Utilities

**Purpose**: Low-level utility functions.

- Memory utilities
- Byte array operations
- Bit manipulation
- Performance timing

### `crates/validation` - Input Validation

**Purpose**: Input validation for settings and user input.

```rust
pub fn validate_email(email: &str) -> bool;
pub fn validate_url(url: &str) -> bool;
pub fn validate_font_size(size: f32) -> bool;
pub fn validate_language_code(code: &str) -> bool;
```

### `crates/reading_time` - Reading Time Estimation

**Purpose**: Estimate reading time for documents.

```rust
pub fn estimate_from_word_count(words: usize, speed: ReadingSpeed) -> Duration;
pub fn estimate_from_page_count(pages: usize, speed: ReadingSpeed) -> Duration;
pub fn format_duration(duration: Duration) -> String;

pub enum ReadingSpeed {
    Slow,      // 200 wpm
    Average,   // 250 wpm
    Fast,      // 300 wpm
}
```

### `crates/accessibility` - Accessibility Features

**Purpose**: Accessibility support for users with disabilities.

#### Features

- **Bionic reading** - Highlight word parts for better focus
- **Auto-pace** - Automatic page turning based on reading speed
- **Dyslexia fonts** - OpenDyslexic and similar fonts
- **High contrast** - Improved visibility
- **Screen reader support** - Integration with accessibility APIs

```rust
pub struct BionicReading {
    pub fn format(text: &str) -> String;  // Bold first parts of words
}
```

### `crates/tts` - Text-to-Speech

**Purpose**: Document narration on supported platforms.

#### Platform Support

- **Desktop** - Via system TTS (Linux, macOS, Windows)
- **Android** - Via Android TTS
- **Kobo** - Not supported (no audio hardware)
- **iOS** - Via AVFoundation

#### TTS Interface

```rust
pub struct TextToSpeech {
    pub fn speak(&self, text: &str, voice: Voice) -> PlatoResult<()>;
    pub fn stop(&self) -> PlatoResult<()>;
    pub fn pause(&self) -> PlatoResult<()>;
    pub fn resume(&self) -> PlatoResult<()>;
    pub fn is_speaking(&self) -> bool;
}

pub struct Voice {
    pub lang: String,
    pub rate: f32,      // 0.5 - 2.0
    pub pitch: f32,     // 0.5 - 2.0
    pub volume: f32,    // 0.0 - 1.0
}
```

#### Feature Flags

```toml
[features]
tts = ["tts"]           # Desktop TTS
tts-android = []        # Android TTS
```

### `crates/core/src/tts_desktop` - Desktop TTS (Linux/macOS/Windows)

**Purpose**: Text-to-speech using system TTS engines.

### `crates/core/src/tts_android` - Android TTS

**Purpose**: JNI bindings to Android TextToSpeech API.

### `crates/i18n` - Internationalization

**Purpose**: Multi-language support.

```rust
pub fn translate(key: &str, lang: &str) -> String;
pub fn format_number(n: f64, lang: &str) -> String;
pub fn format_date(date: DateTime<Utc>, lang: &str) -> String;
```

### `crates/cover_editor` - Cover Art Editing

**Purpose**: Edit and generate cover art.

```rust
pub struct CoverEditor {
    pub fn edit(image_path: &Path) -> PlatoResult<Pixmap>;
    pub fn generate_placeholder(title: &str, author: &str) -> Pixmap;
    pub fn extract_from_document(doc: &dyn Document) -> Option<Pixmap>;
}
```

### `crates/core/src/cover_editor` - Cover Processing

**Purpose**: Extract and process cover images from documents.

### `crates/rar` - RAR Archive Support

**Purpose**: Read RAR (comic archive) support.

```rust
pub fn list_files(path: &Path) -> PlatoResult<Vec<String>>;
pub fn extract(path: &Path, file: &str, dest: &Path) -> PlatoResult<()>;
```

### `crates/searcher` (if present) - Full-Text Search

**Purpose**: Indexed full-text search across library.

```rust
pub struct SearchIndex {
    pub fn build(&mut self, library: &Library) -> PlatoResult<()>;
    pub fn search(&self, query: &str) -> Vec<SearchResult>;
}
```

### `crates/ai` - AI Integration

**Purpose**: AI-powered features (summarization, recommendation).

```rust
pub struct AiEngine {
    pub fn summarize(text: &str) -> PlatoResult<String>;
    pub fn recommend_books(&self, preferences: &str) -> Vec<Book>;
    pub fn generate_metadata(text: &str) -> Metadata;
}
```

---

## Testing

### `crates/core/src/test_mocks` - Mock Implementations

**Purpose**: Mock implementations for unit testing.

```rust
pub struct MockFramebuffer;
pub struct MockBattery;
pub struct MockFrontlight;
pub struct MockLightSensor;
pub struct MockDocument;

impl Display for MockFramebuffer { /* ... */ }
impl Battery for MockBattery { /* ... */ }
// etc.
```

#### Example Test

```rust
#[cfg(test)]
mod tests {
    use crate::test_mocks::*;
    
    #[test]
    fn test_rendering() {
        let mut fb = MockFramebuffer::new();
        let rect = Rectangle::full();
        // Test rendering without hardware
    }
}
```

---

## Building & Compilation

### Cargo Features

```toml
[features]
default = []
android = []
ios = []
desktop = []
tts = []
tts-android = ["android"]
```

### Build Commands

```bash
# Host development (x86_64)
cargo build --target x86_64-unknown-linux-gnu

# Kobo ARM 32-bit (primary)
cargo build --profile release-arm -p plato

# Kobo ARM 64-bit
cargo build --target aarch64-unknown-linux-gnu --profile release-arm64

# Android
cargo build --target aarch64-unknown-linux-gnu-android

# iOS
cargo build --target aarch64-apple-ios
```

### Clippy Linting

```bash
cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings
```

---

## Common Patterns

### Error Handling Pattern

```rust
use plato_core::{PlatoResult, PlatoError};

fn operation() -> PlatoResult<String> {
    let file = std::fs::read_to_string("file.txt")
        .map_err(|e| PlatoError::Io(e))?;
    Ok(file)
}
```

### Device-Aware Code

```rust
use plato_core::CURRENT_DEVICE;

fn setup() {
    let device = &*CURRENT_DEVICE;
    match device.model() {
        plato_core::device::Model::Clara2E => {
            // Clara 2E specific logic
        }
        _ => {
            // Generic fallback
        }
    }
}
```

### View Implementation

```rust
use plato_core::view::{View, Event, Hub};

struct MyView { /* ... */ }

impl View for MyView {
    fn draw(&mut self, fb: &mut Display, rect: Rectangle) -> PlatoResult<()> {
        // Draw content
        Ok(())
    }
    
    fn handle(&mut self, event: &Event, hub: &Hub) -> PlatoResult<bool> {
        match event {
            Event::Touch(e) => { /* ... */ Ok(true) },
            _ => Ok(false)
        }
    }
}
```

---

## Resources

- **AGENTS.md** - Development guidelines
- **CONTRIBUTING.md** - Contribution guide
- **README.md** - Project overview
- **Cargo.toml** - Dependency declarations
- **Examples** - See `examples/` directory

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.9.45+ | May 2026 | Pure Rust PDF (pdfpurr), iOS support, AI integration |
| 0.9.0   | 2023 | Major architecture refactor, modular crates |
| 0.8.0   | 2021 | First public release |

---

**Last Updated**: May 5, 2026  
**Maintainers**: Plato Contributors  
**License**: AGPLv3

