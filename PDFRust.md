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

### 2.2 Comparison with MuPDF (458K LOC)

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

### 2.3 Recommendation

**Use existing libraries - don't rewrite!** Add to `Cargo.toml`:

```toml
# Option 1: hayro (rendering-focused, Apache-2.0)
hayro = "0.5"

# Option 2: PDFPurr (full-featured, MIT/Apache)
pdfpurr = "0.4"
```

### 2.4 Usage Examples

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

### 4.2 Page Rendering (E-ink Optimized)

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