# Plato API Integration Guide

Practical examples and integration patterns for working with Plato.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Document Handling](#document-handling)
3. [Display & Rendering](#display--rendering)
4. [Input Handling](#input-handling)
5. [UI Development](#ui-development)
6. [Settings Management](#settings-management)
7. [Library Integration](#library-integration)
8. [Advanced Patterns](#advanced-patterns)

---

## Getting Started

### Project Setup

Create a new Rust project:

```bash
cargo new my-plato-app
cd my-plato-app
```

Add Plato dependencies to `Cargo.toml`:

```toml
[dependencies]
plato-core = { path = "../plato/crates/core" }
plato-error = { path = "../plato/crates/error" }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

### Basic Device Detection

```rust
use plato_core::CURRENT_DEVICE;

fn main() {
    let device = &*CURRENT_DEVICE;
    
    println!("Device Model: {}", device.model());
    println!("Resolution: {:?}", device.dims());
    println!("DPI: {}", device.dpi());
    println!("Has natural light: {}", device.has_natural_light());
    println!("Touch protocol: {:?}", device.proto());
}
```

### Error Handling

```rust
use plato_core::{PlatoResult, PlatoError};

fn read_document(path: &str) -> PlatoResult<Vec<u8>> {
    std::fs::read(path)
        .map_err(|e| PlatoError::Io(e))
}

fn main() -> PlatoResult<()> {
    let data = read_document("document.pdf")?;
    println!("Loaded {} bytes", data.len());
    Ok(())
}
```

---

## Document Handling

### Loading and Rendering Documents

```rust
use plato_core::document::Document;
use plato_core::{PlatoResult, Rectangle};
use std::path::Path;

fn open_document(path: &Path) -> PlatoResult<Box<dyn Document>> {
    let file = std::fs::read(path)?;
    
    match path.extension().and_then(|s| s.to_str()) {
        Some("pdf") => {
            let doc = plato_core::document::PdfDocument::new(&file)?;
            Ok(Box::new(doc))
        }
        Some("epub") => {
            let doc = plato_core::document::EpubDocument::new(&file)?;
            Ok(Box::new(doc))
        }
        _ => Err(plato_core::PlatoError::InvalidFileFormat(
            format!("{:?}", path.extension()),
            None,
        ))
    }
}

fn render_page(doc: &mut dyn Document, page: usize) -> PlatoResult<()> {
    let (width, height) = doc.page_dims(page);
    println!("Page {} dimensions: {}x{}", page, width, height);
    
    let rect = Rectangle::new(
        plato_core::geom::Point::new(0, 0),
        plato_core::geom::Point::new(width as i32, height as i32),
    );
    
    let pixmap = doc.render(page, rect)?;
    println!("Rendered page {} to pixmap", page);
    
    Ok(())
}

fn main() -> plato_core::PlatoResult<()> {
    let mut doc = open_document(Path::new("test.pdf"))?;
    render_page(doc.as_mut(), 0)?;
    Ok(())
}
```

### Text Extraction

```rust
use plato_core::document::Document;

fn extract_text(doc: &dyn Document, page: usize) -> plato_core::PlatoResult<String> {
    // Document trait provides text extraction if supported
    // Implementation varies by document format
    let outline = doc.outline();
    if let Some(items) = outline {
        for item in items {
            println!("TOC: {}", item.title);
        }
    }
    
    Ok(String::new())  // Placeholder
}
```

### PDF-Specific Operations

```rust
use plato_core::PlatoResult;

fn pdf_operations() -> PlatoResult<()> {
    let pdf = plato_core::document::PdfDocument::new(&std::fs::read("test.pdf")?)?;
    
    println!("Pages: {}", pdf.pages_count());
    println!("Is reflowable: {}", pdf.is_reflowable());
    
    // Check for annotations
    if let Some(annotations) = pdf.annotations(0) {
        for note in annotations {
            println!("Annotation: {}", note.content);
        }
    }
    
    Ok(())
}
```

### EPUB Handling

```rust
use plato_core::document::EpubDocument;
use plato_core::PlatoResult;

fn epub_operations() -> PlatoResult<()> {
    let epub = EpubDocument::new(&std::fs::read("book.epub")?)?;
    
    // EPUB provides better metadata access
    if let Some(metadata) = epub.metadata() {
        println!("Title: {}", metadata.title);
        println!("Author: {}", metadata.author);
        println!("Language: {:?}", metadata.language);
    }
    
    // EPUBs are reflowable - can adapt to different page sizes
    println!("Reflowable: {}", epub.is_reflowable());
    
    Ok(())
}
```

---

## Display & Rendering

### Framebuffer Access

```rust
use plato_core::framebuffer::{Display, KoboFramebuffer2, UpdateMode};
use plato_core::{PlatoResult, geom::Rectangle, geom::Point};

fn basic_framebuffer() -> PlatoResult<()> {
    let mut fb = KoboFramebuffer2::new()?;
    
    // Get screen dimensions
    let (width, height) = fb.dims();
    println!("Framebuffer: {}x{}", width, height);
    
    // Create a drawing area
    let rect = Rectangle::new(
        Point::new(100, 100),
        Point::new(200, 200),
    );
    
    // Update specific region
    fb.update_region(&rect, UpdateMode::Partial)?;
    
    Ok(())
}
```

### Display Update Modes

```rust
use plato_core::framebuffer::UpdateMode;

fn update_strategies() {
    // Full update: Complete refresh, highest quality, slower
    let mode = UpdateMode::Full;
    // Use for: Large content changes, full page turns
    
    // Partial update: Fast refresh, some ghosting, suitable for text
    let mode = UpdateMode::Partial;
    // Use for: Minor updates, highlighting, menu changes
    
    // Fast update: Very fast, lower quality, may have artifacts
    let mode = UpdateMode::Fast;
    // Use for: Animations, frequent updates
}
```

### Creating a Pixmap

```rust
use plato_core::framebuffer::Pixmap;
use plato_core::color::Color;

fn create_pixmap() {
    let width = 600u32;
    let height = 800u32;
    
    let mut pixmap = Pixmap::new(width, height);
    
    // Fill with white
    pixmap.draw_rect(
        &pixmap.rect(),
        &Color::white(),
    );
    
    // Draw a black box
    let box_rect = plato_core::geom::Rectangle::new(
        plato_core::geom::Point::new(10, 10),
        plato_core::geom::Point::new(100, 100),
    );
    pixmap.draw_rect(&box_rect, &Color::black());
}
```

### Rendering with Frontlight

```rust
use plato_core::frontlight::Frontlight;

fn adjust_display(fb: &mut impl Frontlight) -> plato_core::PlatoResult<()> {
    // Get current intensity
    let intensity = fb.intensity();
    println!("Current frontlight: {}", intensity * 100.0);
    
    // Set intensity (0.0 = off, 1.0 = maximum)
    fb.set_intensity(0.5)?;
    
    // Check for natural light (warm/cool color temperature)
    if fb.is_natural() {
        if let Some(temp) = fb.temperature() {
            println!("Color temperature: {}K", temp);
        }
    }
    
    Ok(())
}
```

---

## Input Handling

### Touch Events

```rust
use plato_core::input::{TouchProto, FingerStatus};
use plato_core::PlatoResult;

fn process_touch() -> PlatoResult<()> {
    // Detect touch protocol
    let proto = plato_core::CURRENT_DEVICE.proto();
    println!("Touch protocol: {:?}", proto);
    
    match proto {
        TouchProto::A => println!("Kobo Gen 1 touch"),
        TouchProto::B => println!("Kobo Gen 2 touch"),
        TouchProto::C => println!("Kobo Gen 3+ touch"),
    }
    
    Ok(())
}
```

### Gesture Recognition

```rust
use plato_core::gesture::{GestureRec, GestureEvent};
use plato_core::input::Contact;
use std::time::SystemTime;

fn recognize_gestures() -> plato_core::PlatoResult<()> {
    let mut recognizer = GestureRec::new();
    
    // Simulate touch contacts
    let contact = Contact {
        id: 0,
        x: 300.0,
        y: 400.0,
        pressure: 100,
    };
    
    let time = SystemTime::now();
    
    if let Some(gesture) = recognizer.push(contact, time)? {
        match gesture {
            GestureEvent::Tap => println!("User tapped"),
            GestureEvent::DoubleTap => println!("User double-tapped"),
            GestureEvent::Swipe(dir) => println!("User swiped: {:?}", dir),
            GestureEvent::LongPress => println!("User long-pressed"),
            _ => println!("Other gesture: {:?}", gesture),
        }
    }
    
    Ok(())
}
```

### Button Events

```rust
use plato_core::input::InputEvent;

fn handle_button_events() {
    let device = &*plato_core::CURRENT_DEVICE;
    
    if device.has_page_turn_buttons() {
        println!("Device has page-turn buttons");
        // Handle physical button events
    }
    
    // Button events are delivered via InputEvent::Button
}
```

---

## UI Development

### Custom View Implementation

```rust
use plato_core::view::{View, Event, Hub};
use plato_core::framebuffer::Display;
use plato_core::{PlatoResult, geom::Rectangle};

pub struct CustomView {
    rect: Rectangle,
    data: String,
}

impl CustomView {
    pub fn new(rect: Rectangle) -> Self {
        Self {
            rect,
            data: "Custom View".to_string(),
        }
    }
}

impl View for CustomView {
    fn draw(&mut self, fb: &mut Display, rect: Rectangle) -> PlatoResult<()> {
        // Draw the view content
        // 1. Fill background
        // 2. Draw text/graphics
        // 3. Update framebuffer region
        Ok(())
    }
    
    fn handle(&mut self, event: &Event, hub: &Hub) -> PlatoResult<bool> {
        match event {
            Event::Touch(touch_event) => {
                // Check if touch is within our bounds
                if self.rect.contains(touch_event.point) {
                    println!("Custom view was touched!");
                    // Optionally post event to hub
                    hub.post(/* event */);
                    return Ok(true);
                }
                Ok(false)
            }
            Event::Gesture(gesture) => {
                println!("Custom view received gesture: {:?}", gesture);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
    
    fn rect(&self) -> Rectangle {
        self.rect
    }
}
```

### Event Hub Communication

```rust
use plato_core::view::{Hub, Event};

fn communicate_between_views(hub: &Hub) {
    // Post custom event for other views to handle
    let custom_event = Event::Custom(/* your data */);
    hub.post(custom_event);
    
    // Views can subscribe to event types
}
```

### Layout Container

```rust
use plato_core::view::{View, Rectangle};

struct VerticalStack {
    rect: Rectangle,
    views: Vec<Box<dyn View>>,
    spacing: i32,
}

impl VerticalStack {
    pub fn new(rect: Rectangle, spacing: i32) -> Self {
        Self {
            rect,
            views: Vec::new(),
            spacing,
        }
    }
    
    pub fn add_view(&mut self, view: Box<dyn View>) {
        self.views.push(view);
    }
    
    fn layout_views(&mut self) {
        let mut y = self.rect.top();
        let width = self.rect.width();
        
        for view in &mut self.views {
            let view_height = 100;  // Example
            let rect = Rectangle::new(
                plato_core::geom::Point::new(self.rect.left(), y),
                plato_core::geom::Point::new(self.rect.left() + width, y + view_height),
            );
            // Update view position
            y += view_height + self.spacing;
        }
    }
}
```

---

## Settings Management

### Loading Settings

```rust
use plato_core::settings::Settings;
use plato_core::PlatoResult;

fn load_configuration() -> PlatoResult<()> {
    // Load from default location
    let settings = Settings::load()?;
    
    println!("Frontlight: {}", settings.display.frontlight_level);
    println!("Font family: {}", settings.reading.font_family);
    println!("Font size: {}", settings.reading.font_size);
    println!("Dark mode: {}", settings.interface.dark_mode);
    
    Ok(())
}
```

### Modifying Settings

```rust
use plato_core::settings::Settings;
use plato_core::PlatoResult;

fn customize_settings() -> PlatoResult<()> {
    let mut settings = Settings::load()?;
    
    // Adjust display settings
    settings.display.frontlight_level = 0.7;
    settings.display.invert_colors = true;
    
    // Customize reading settings
    settings.reading.font_size = 14.0;
    settings.reading.line_height = 1.5;
    settings.reading.margin = 20;
    
    // Save changes
    settings.save()?;
    
    Ok(())
}
```

### Custom Configuration Files

```rust
use plato_core::config::Config;
use plato_core::PlatoResult;
use std::path::Path;

fn load_custom_config(path: &Path) -> PlatoResult<()> {
    // Load TOML configuration file
    let config = Config::load_from_path(path)?;
    
    // Access configuration values
    if let Some(value) = config.get("setting.name") {
        println!("Setting value: {}", value);
    }
    
    Ok(())
}
```

---

## Library Integration

### Working with Document Library

```rust
use plato_core::library::Library;
use plato_core::PlatoResult;

fn browse_library() -> PlatoResult<()> {
    let library = Library::new()?;
    
    // List all books
    let books = library.books();
    for book in books {
        println!("{} by {}", book.title, book.author);
    }
    
    // Search for books
    let results = library.search("rust");
    println!("Found {} books about Rust", results.len());
    
    Ok(())
}
```

### Adding Documents to Library

```rust
use plato_core::library::Library;
use plato_core::PlatoResult;
use std::path::Path;

fn add_to_library() -> PlatoResult<()> {
    let mut library = Library::new()?;
    
    // Add a document to the library
    let path = Path::new("/books/mybook.pdf");
    library.add_book(path)?;
    
    println!("Book added to library");
    
    Ok(())
}
```

### Working with Metadata

```rust
use plato_core::metadata::Metadata;
use plato_core::PlatoResult;
use std::path::Path;

fn extract_metadata(path: &Path) -> PlatoResult<Metadata> {
    // Extract metadata from document
    let doc = std::fs::read(path)?;
    let metadata = Metadata::from_document(&doc, path)?;
    
    println!("Title: {}", metadata.title);
    println!("Author: {}", metadata.author);
    println!("Pages: {:?}", metadata.pages);
    
    Ok(metadata)
}
```

---

## Advanced Patterns

### Platform-Specific Code

```rust
use plato_core::is_mobile_platform;

#[cfg(target_os = "android")]
fn platform_specific() {
    println!("Running on Android");
    // Use Android-specific APIs
}

#[cfg(target_os = "ios")]
fn platform_specific() {
    println!("Running on iOS");
    // Use iOS-specific APIs
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn platform_specific() {
    println!("Running on desktop");
}

fn main() {
    if is_mobile_platform() {
        println!("Mobile platform detected");
        // Apply mobile optimizations
    }
}
```

### Memory-Efficient Document Processing

```rust
use plato_core::document::Document;
use plato_core::mobile_optimizations::memory_config;
use plato_core::PlatoResult;

fn process_large_document(doc: &mut dyn Document) -> PlatoResult<()> {
    let config = memory_config();
    println!("Recommended buffer: {} bytes", config.max_buffer_size);
    
    // Process in chunks to avoid memory issues
    let pages = doc.pages_count();
    for page in 0..pages {
        let (_w, _h) = doc.page_dims(page);
        
        // Render with appropriate buffer size
        // Clear rendered page from memory before next iteration
    }
    
    Ok(())
}
```

### Battery-Aware Operations

```rust
use plato_core::battery::Battery;
use plato_core::CURRENT_DEVICE;

fn battery_aware_sync(battery: &dyn Battery) -> plato_core::PlatoResult<()> {
    match battery.capacity_level() {
        0..=10 => {
            println!("Critical battery - skipping sync");
            return Ok(());
        }
        11..=30 => {
            println!("Low battery - minimal sync only");
            // Do essential sync only
        }
        _ => {
            println!("Battery OK - full sync");
            // Perform full sync
        }
    }
    Ok(())
}
```

### Async Document Loading

```rust
use plato_core::document::Document;
use tokio::task;

async fn load_document_async(path: String) -> plato_core::PlatoResult<Box<dyn Document>> {
    task::spawn_blocking(move || {
        let data = std::fs::read(&path)?;
        match path.ends_with(".pdf") {
            true => Ok(Box::new(plato_core::document::PdfDocument::new(&data)?)),
            false => Err(plato_core::PlatoError::InvalidFileFormat(path, None)),
        }
    })
    .await
    .unwrap()
}
```

### Caching Thumbnails

```rust
use plato_core::thumbnail::Thumbnail;
use plato_core::document::Document;
use plato_core::PlatoResult;

fn cache_thumbnails(doc: &mut dyn Document) -> PlatoResult<()> {
    let cache_dir = Thumbnail::cache_dir();
    println!("Cache directory: {:?}", cache_dir);
    
    // Generate and cache thumbnail for first page
    let thumb = Thumbnail::generate(doc)?;
    println!("Generated thumbnail: {}x{}", thumb.width(), thumb.height());
    
    // Save to cache
    // Implementation depends on Thumbnail API
    
    Ok(())
}
```

### Text-to-Speech Integration

```rust
#[cfg(feature = "tts")]
use plato_core::tts::{TextToSpeech, Voice};
#[cfg(feature = "tts")]
use plato_core::PlatoResult;

#[cfg(feature = "tts")]
fn narrate_text() -> PlatoResult<()> {
    let tts = TextToSpeech::new()?;
    
    let voice = Voice {
        lang: "en-US".to_string(),
        rate: 1.0,
        pitch: 1.0,
        volume: 1.0,
    };
    
    tts.speak("The quick brown fox jumps over the lazy dog", voice)?;
    
    while tts.is_speaking() {
        // Wait for narration
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    Ok(())
}
```

### Pocket Integration

```rust
use plato_core::pocket::PocketClient;
use plato_core::PlatoResult;

fn sync_pocket_articles() -> PlatoResult<()> {
    let mut pocket = PocketClient::authenticate("your-token")?;
    
    // Fetch all articles
    let articles = pocket.get_articles()?;
    println!("Found {} articles", articles.len());
    
    for article in articles {
        println!("- {}: {}", article.title, article.url);
    }
    
    // Archive an article
    pocket.archive(&articles[0].id)?;
    
    Ok(())
}
```

---

## Best Practices

### 1. Always Use Error Handling

```rust
// ✓ Good
fn operation() -> plato_core::PlatoResult<()> {
    let data = std::fs::read("file")?;
    Ok(())
}

// ✗ Bad
fn operation() {
    let data = std::fs::read("file").unwrap();  // Panic!
}
```

### 2. Respect Device Capabilities

```rust
// ✓ Good
fn init_display() -> plato_core::PlatoResult<()> {
    let device = &*plato_core::CURRENT_DEVICE;
    
    if device.has_natural_light() {
        // Use warm light feature
    }
    
    Ok(())
}
```

### 3. Memory Efficiency

```rust
// ✓ Good - pre-allocate known size
let mut buffer = Vec::with_capacity(1024);

// ✗ Bad - repeated allocations
let mut buffer = Vec::new();
for _ in 0..1000 {
    buffer.push(0u8);  // Reallocates multiple times
}
```

### 4. Platform Checks

```rust
// ✓ Good - use runtime check
if plato_core::is_mobile_platform() {
    apply_mobile_optimizations();
}

// ✗ Bad - assumes platform
#[cfg(not(target_os = "android"))]
fn operation() {
    // May not work on iOS
}
```

---

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| `InvalidFileFormat` | Check file extension and magic bytes |
| `MemoryError` | Reduce buffer size, use streaming |
| `Battery` error | Device battery API not available on this platform |
| Touch not working | Wrong `TouchProto` for device model |
| Rendering artifacts | Use `UpdateMode::Full` instead of `Partial` |

---

## Resources

- **API Reference**: See `docs/API.md`
- **Examples**: Check `examples/` directory
- **AGENTS.md**: Development guidelines
- **CONTRIBUTING.md**: Contribution guidelines

