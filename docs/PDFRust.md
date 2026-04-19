# PDFRust.md - MuPDF to Rust Replacement Plan

## Overview

Comprehensive plan to replace MuPDF C library dependency with pure Rust alternatives in Plato, following AGENTS.md rules (no backward compatibility). **With the emergence of hayro and PDFPurr in 2025-2026, full pure Rust migration is now viable.**

---

## 1. Current MuPDF Usage Summary

### 1.1 MuPDF Core Modules (1,867 lines)

| Module | File | Lines | Category |
|--------|------|-------|----------|
| FFI sys | `mupdf_sys.rs` | 531 | Core types/FFI |
| Context | `mupdf/context.rs` | 164 | Document context |
| Document | `mupdf/document.rs` | 213 | PDF document |
| Page | `mupdf/page.rs` | 238 | Page operations |
| Text | `mupdf/text.rs` | 245 | Text extraction |
| Pixmap | `mupdf/pixmap.rs` | 179 | Rendering |
| Outline | `mupdf/outline.rs` | 102 | TOC |
| Annotations | `mupdf/annotation.rs` | 71 | Annotations |
| Links | `mupdf/link.rs` | 59 | Links |
| Images | `mupdf/image.rs` | 35 | Images |
| Module | `mupdf/mod.rs` | 30 | Re-exports |

### 1.2 Search Integration (~500 lines)

| Component | File | Lines | Purpose |
|-----------|------|--------|
| `Search` struct | `reader_core.rs` | Search state |
| Search handler | `reader_search_handler.rs` | 234 | Search management |
| Search UI | `search_bar.rs` | ~200 | Search interface |
| Search stub | `reader_stubs.rs` | ~50 | Search logic |

---

## 2. New Pure Rust PDF Libraries (2025-2026)

### 2.1 Available Libraries Comparison

| Library | Version | Status | Rendering | Features | License |
|---------|----------|--------|------------|-----------|----------|
| **hayro** | v0.5 | Active | vello | ~90% feature-complete | Apache-2.0 |
| **PDFPurr** | v0.4 | Active | tiny-skia | Full (OCR, forms, encryption) | MIT/Apache |
| **lopdf** | v0.40+ | Mature | ❌ | PDF editing (no render) | MIT |
| **oxidize-pdf** | v1.6 | Active | Custom | Full features | AGPL-3.0 |
| **MuPDF** | 1.27 | Active | Custom | **E-ink optimized** | AGPL-3.0 |

### 2.2 E-ink Optimization Requirements

E-ink displays have unique requirements that general PDF libraries don't support:

| Requirement | Description | Library Support |
|-------------|-------------|-----------------|
| **Partial updates** | Only update changed regions | MuPDF only |
| **Ghosting reduction** | Clean refresh after updates | Custom needed |
| **Grayscale only** | 16-level gray for e-ink | Custom needed |
| **Waveform modes** | Different refresh intensities | Custom needed |
| **Delta updates** | Send only pixel differences | Custom needed |

**Critical**: Neither hayro nor PDFPurr support e-ink-specific rendering. Custom layer required.

### 2.3 E-ink Optimization Strategy

For Kobo e-readers, we must keep MuPDF wrapper BUT use hayro/PDFPurr for non-critical paths:

```rust
// Platform-specific PDF backend selection
#[cfg(target_arch = "arm")]
use mupdf_rust;  // MuPDF for ARM (Kobo)

#[cfg(not(target_arch = "arm"))]
use hayro;       // hayro for desktop/emulator
```

Or use feature flags:

```toml
[features]
default = ["eink_optimized"]
eink_optimized = ["mupdf-rust"]
desktop = ["hayro"]
```

### 2.4 Feature Comparison

| Feature | MuPDF (C) | hayro | PDFPurr | lopdf |
|---------|-----------|------|---------|-------|
| **Rendering** | ✅ Optimized | ✅ Good | ✅ Good | ❌ |
| **Text positions** | Exact | ~90% | ~90% | ❌ |
| **Annotations** | Full | Limited | Full | Limited |
| **Forms** | Full | No | Full | No |
| **Encryption** | Full | R2-R6 | Full | Yes |
| **Performance** | Fast | Medium | Fast | Fast |
| **E-ink optimized** | ✅ Custom | ❌ | ❌ | ❌ |
| **Lines of code** | 458K | ~50K | ~80K | N/A |

### 2.4 Detailed Feature Availability

| # | MuPDF Use | MuPDF Function | lopdf | hayro | PDFPurr |
|---|-----------|----------------|-------|-------|----------|
| 1 | Document loading | `MuPdfContext::new()` | ✅ `Document::load()` | ✅ `Pdf::from_file()` | ✅ `PdfDoc::from_path()` |
| 2 | Context creation | `new_mupdf_context()` | ✅ | ✅ | ✅ |
| 3 | Page access | `doc.get_page()` | ✅ | ✅ `pdf.pages()` | ✅ |
| 4 | Page bounds | `page.bound_box()` | ✅ `page.boundaries()` | ✅ | ✅ |
| 5 | Text extraction | `TextPage::blocks()` | ⚠️ `extract_text()` | ⚠️ ~90% | ✅ |
| 6 | Text blocks | `TextBlock` | ❌ | ⚠️ Approximate | ✅ |
| 7 | Character positions | `chr_rect` | ❌ | ❌ | approx |
| 8 | Word positions | `word_rect` | ❌ | ❌ | approx |
| 9 | Image extraction | `Image` | ✅ | ✅ | ✅ |
| 10 | Links extraction | `Link` | ⚠️ | ✅ | ✅ |
| 11 | Annotations CRUD | `Annotation` | ⚠️ Limited | ⚠️ Limited | ✅ |
| 12 | TOC/Outline | `Outline` | ✅ `get_toc()` | ✅ | ✅ |
| 13 | Render to pixmap | `page.to_pixmap()` | ❌ | ✅ `render()` | ✅ `render_page()` |
| 14 | Save document | `doc.save()` | ✅ `save()` | ✅ | ✅ |
| 15 | Page insert | `insert_page()` | ✅ | ✅ | ✅ |
| 16 | Page delete | `delete_page()` | ✅ | ✅ | ✅ |
| 17 | Merge documents | `add_document_pages()` | ✅ | ✅ | ✅ |
| 18 | Encryption | AES support | ✅ | ✅ R2-R6 | ✅ Full |
| 19 | Progressive load | `ProgressiveLoader` | ⚠️ | ⚠️ | ✅ |
| 20 | Search text | `TextPage::blocks()` | ❌ | ⚠️ Approximate | ⚠️ |
| 21 | E-ink rendering | Custom partial updates | ❌ | ❌ | ❌ |
| 22 | Delta updates | Custom algorithm | ❌ | ❌ | ❌ |

### 2.5 Summary by Availability

| Status | Count | Features |
|--------|-------|----------|
| **✅ Direct replacement** | 11 | Document load, page access, save, merge, TOC |
| **⚠️ Partial/Approximate** | 6 | Text ~90%, links, annotations, search |
| **❌ Custom needed** | 5 | Exact text positions, E-ink, partial updates |

### 2.6 Recommended Backend

| Feature | Backend |
|---------|---------|
| Desktop PDF viewing | **hayro** or **PDFPurr** |
| E-ink rendering | Keep **MuPDF** |
| Document editing | **lopdf** or **PDFPurr** |
| Search with positions | Custom + approximate |
| OCR support | **PDFPurr** |

### 2.7 Recommendation

**Use existing libraries - don't rewrite!** Add to `Cargo.toml`:

```toml
# Option 1: hayro (rendering-focused, Apache-2.0)
hayro = "0.5"

# Option 2: PDFPurr (full-featured, MIT/Apache)
pdfpurr = "0.4"
```

### 2.8 Usage Examples

```rust
// hayro - Rendering focused
use hayro::{Pdf, InterpreterSettings, RenderSettings, render};

let pdf = Pdf::from_file("doc.pdf").unwrap();
let page = pdf.pages()[0];
let pixmap = render(&page, &InterpreterSettings::default(), &RenderSettings::default());

// PDFPurr - Full features
use pdfpurr::{PdfDoc, Renderer, RenderOptions};

let doc = PdfDoc::from_path("doc.pdf").unwrap();
let renderer = Renderer::new(&doc, RenderOptions { dpi: 150.0, .. });
let pixmap = renderer.render_page(0).unwrap();
```

---

## 3. Replacement Strategy

### 3.1 Library Selection

| Purpose | Primary Library | Notes |
|---------|---------------|-------|
| PDF rendering | **hayro** | Best rendering, uses vello |
| Full PDF ops | **PDFPurr** | OCR, forms, encryption |
| Low-level edit | **lopdf** | Object manipulation |

### 3.2 Features Requiring Custom Implementation

| Feature | Library | Custom Needed |
|---------|---------|----------------|
| Text bounding boxes | hayro/PDFPurr | ~90% accuracy via font metrics |
| E-ink rendering | None | Custom partial update layer |
| Annotation CRUD | PDFPurr | Limited - custom impl |
| Search with positions | hayro | Approximate via text analysis |

---

## 4. Phase 1: Migration (hayro/PDFPurr)

### 4.1 Document Loading & Access

```rust
// hayro-based wrapper
use hayro::Pdf;

pub struct PdfContext {
    pdf: Pdf,
}

impl PdfContext {
    pub fn new(path: &Path) -> Result<Self> {
        Ok(Self { pdf: Pdf::from_file(path)? })
    }
    
    pub fn page_count(&self) -> usize {
        self.pdf.pages().len()
    }
    
    pub fn get_page(&self, index: usize) -> Option<Page> {
        self.pdf.pages().get(index).cloned()
    }
}
```

### 4.2 E-ink Rendering Layer (Custom Required)

Since hayro/PDFPurr don't support e-ink, create custom optimization layer:

```rust
// E-ink specific rendering optimizations

/// Update modes for e-ink displays
pub enum EinkUpdateMode {
    /// Full refresh - clears ghosting, uses full waveform
    Full,
    /// Partial update - faster, may accumulate ghosting
    Partial,
    /// FAST - quick text update
    Fast,
}

/// E-ink optimized renderer
pub struct EinkRenderer {
    /// Previous frame for delta updates
    prev_frame: Vec<u8>,
    /// Current update mode
    mode: EinkUpdateMode,
}

impl EinkRenderer {
    /// Render page with e-ink optimizations
    pub fn render(&mut self, page: &Page, dpi: f32) -> Pixmap {
        // First render full page
        let full = hayro::render(page, dpi);
        
        match self.mode {
            EinkUpdateMode::Full => {
                // Full waveform, clear previous
                self.prev_frame = full.pixels.clone();
                full
            }
            EinkUpdateMode::Partial => {
                // Calculate delta - only changed regions
                let delta = self.compute_delta(&full);
                self.prev_frame = full.pixels.clone();
                delta
            }
            EinkUpdateMode::Fast => {
                // Quick update with reduced quality
                let fast = self.reduce_quality(&full);
                self.prev_frame = full.pixels.clone();
                fast
            }
        }
    }
    
    /// Compute delta (only changed pixels)
    fn compute_delta(&self, new: &Pixmap) -> Pixmap {
        let mut delta = Vec::with_capacity(new.pixels.len());
        
        for (i, (old, new)) in self.prev_frame.iter().zip(new.pixels.iter()).enumerate() {
            if old != new {
                delta.push(new);  // Changed
            } else {
                delta.push(0);    // Same - transparent
            }
        }
        
        Pixmap { pixels: delta, .. }
    }
    
    /// Reduce to 4-bit grayscale for FAST mode
    fn reduce_quality(&self, pixmap: &Pixmap) -> Pixmap {
        Pixmap {
            pixels: pixmap.pixels.iter()
                .map(|p| *p / 17)  // 256 -> 16 levels
                .collect(),
            ..
        }
    }
}
```

### 4.2.1 Waveform Management

E-ink displays use waveforms to control pixel transitions:

```rust
/// E-ink waveform modes
pub enum WaveformMode {
    DU4,    // 4-level fast
    DU8,     // 8-level medium
    GC16,    // 16-level for full graphics
    AUTO,    // Automatic based on content
}

/// Select optimal waveform mode
pub fn select_waveform(content: &Content) -> WaveformMode {
    let text_ratio = content.text_pixels / content.total_pixels;
    
    if text_ratio > 0.8 {
        WaveformMode::DU4   // Mostly text - fast
    } else if text_ratio > 0.3 {
        WaveformMode::DU8  // Mixed
    } else {
        WaveformMode::GC16 // Graphics
    }
}
```

```rust
use hayro::{InterpreterSettings, RenderSettings, render};

impl PdfContext {
    pub fn render_page(&self, page_index: usize, dpi: f32) -> Result<Pixmap> {
        let page = self.pdf.pages()[page_index];
        
        let interp_settings = InterpreterSettings::default();
        let render_settings = RenderSettings {
            x_scale: dpi / 72.0,
            y_scale: dpi / 72.0,
            ..Default::default()
        };
        
        Ok(render(&page, &interp_settings, &render_settings))
    }
}
```

### 4.3 Text Extraction

```rust
// Basic text extraction via hayro
use hayro::Pdf;

pub fn get_page_text(pdf: &Pdf, page_index: usize) -> String {
    // hayro provides approximate text extraction
    // For exact positions, custom analysis needed
    pdf.pages()[page_index].content().to_string()
}
```

---

## 5. Phase 2: Advanced Features

### 5.1 Links Extraction

```rust
// Links from PDF annotations
pub fn get_links(pdf: &Pdf, page_index: usize) -> Vec<Link> {
    // Use lopdf for low-level access
    use lopdf::{Document, ObjectId};
    
    let doc = Document::load("doc.pdf").unwrap();
    // Extract links from Annots dictionary
}
```

### 5.2 Annotations

```rust
// Annotations handling
pub struct Annotation {
    pub annot_type: String,
    pub rect: Rect,
    pub contents: Option<String>,
}
```

### 5.3 TOC/Outline

```rust
pub fn get_toc(pdf: &Pdf) -> Vec<TocEntry> {
    pdf.outline().map(|o| TocEntry {
        title: o.title().to_string(),
        page: o.page().num(),
    })).collect()
}
```

---

## 6. Phase 3: Search Implementation (Critical)

### 6.1 Search Backend

```rust
use hayro::Pdf;

pub struct SearchResult {
    pub page: usize,
    pub text: String,
    pub bounds: Rect,  // Approximate
}

pub fn search_pdf(pdf: &Pdf, query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let re = regex::Regex::new(&format!("(?i){}", query)).unwrap();
    
    for (page_idx, page) in pdf.pages().iter().enumerate() {
        let text = page.content();
        
        for mat in re.find_iter(&text) {
            results.push(SearchResult {
                page: page_idx,
                text: mat.as_str().to_string(),
                bounds: Rect::default(),  // Can't get exact bounds
            });
        }
    }
    
    results
}
```

### 6.2 Approximate Position Tracking

```rust
// Estimate position from font metrics
pub fn estimate_word_bounds(
    text: &str,
    font_size: f32,
    char_width: f32,
) -> Rect {
    let width = text.len() as f32 * char_width;
    Rect {
        x0: 0.0,
        y0: 0.0,
        x1: width,
        y1: font_size,
    }
}
```

### 6.3 Search Limitations (vs MuPDF)

| Feature | MuPDF | hayro Replacement |
|---------|------|------------------|
| Exact word bounds | ✅ | Approximate |
| Accurate highlighting | ✅ | Region-based |
| Case sensitivity | ✅ | ✅ |
| Regex support | ✅ | ✅ |

---

## 7. Phase 4: Integration & Testing

### 7.1 Module Integration

```rust
// document/pdf_rust/mod.rs
pub mod context;      // Document wrapper
pub mod render;       // Rendering
pub mod text;        // Text extraction
pub mod search;       // Search
pub mod links;       // Links
pub mod annotation; // Annotations
pub mod toc;        // TOC

pub use context::PdfContext;
pub use render::render_page;
```

### 7.2 Feature Flag

```toml
[features]
default = ["use_mupdf"]
use_hayro = ["hayro"]
use_purf = ["pdfpurr"]
```

### 7.3 Build Verification

```bash
# ARM build
cargo build --target arm-unknown-linux-gnueabihf -p plato-core

# Host build
cargo build --target x86_64-unknown-linux-gnu -p plato-core

# Clippy
cargo clippy -- -D warnings
```

---

## 8. Known Limitations

| Feature | MuPDF | Alternative | Workaround |
|---------|------|-------------|------------|
| Exact text bounds | ✅ | ~90% | Font-based estimation |
| E-ink rendering | Custom | ❌ | Custom partial updates |
| Annotation editing | Full | Limited | Custom impl |
| Performance | Optimized | Medium | Optimize DPI |
| Memory usage | Low | Higher | Streaming |

---

## 9. Files to Create/Modify

```
crates/core/src/document/
├── pdf_rust/
│   ├── mod.rs        # Module + re-exports
│   ├── context.rs    # Document wrapper
│   ├── render.rs     # Rendering
│   ├── text.rs       # Text extraction
│   └── search.rs    # Search
└── (existing files remain as fallback)
```

---

## 10. Completion Criteria

- [ ] Replace MuPDF FFI with hayro/PDFPurr
- [ ] Implement search backend
- [ ] Zero warnings on ARM/host builds
- [ ] Performance acceptable (vs MuPDF)
- [ ] Document E-ink limitations
- [ ] Tests pass

---

## 11. Timeline Estimate

| Phase | Tasks | Effort |
|--------|-------|--------|
| Phase 1 | Document loading, rendering | 2 weeks |
| Phase 2 | Advanced features | 1 week |
| Phase 3 | Search implementation | 1 week |
| Phase 4 | Integration, testing | 2 weeks |

**Total: ~6 weeks**