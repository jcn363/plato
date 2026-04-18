# PDFRust.md - MuPDF to Rust Replacement Plan

## Overview

Comprehensive plan to replace MuPDF C library dependency with pure Rust alternatives in Plato, following AGENTS.md rules (no backward compatibility).

## 1. Current MuPDF Usage Summary

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
| **Total** | | **1,867** | |

### Search Integration

| Component | File | Lines | Purpose |
|-----------|------|--------|
| `Search` struct | `reader_core.rs` | Search state |
| Search handler | `reader_search_handler.rs` | 234 | Search management |
| Search UI | `search_bar.rs` | ~200 | Search interface |
| Search stub | `reader_stubs.rs` | ~50 | Search logic |

**Total search code**: ~500 lines

---

## 2. Replacement Strategy

### Primary Libraries

| Purpose | Library | Version | License |
|---------|---------|---------|----------|
| PDF parsing/editing | `lopdf` | 0.40+ | MIT |
| Rendering | `printpdf` | 0.7+ | MIT/Apache |
| Text extraction | `pdf-extract` | 0.4+ | Apache |

### Fallback for Complex Features

For features not available in pure Rust, create custom implementations:

1. **Text bounding boxes** - Custom text analysis
2. **E-ink rendering** - Custom partial update logic
3. **Annotation handling** - Custom implementation
4. **Search positions** - Custom position tracking

---

## 3. Phase 1: Foundation Replacements

### 3.1.1 Replace FFI Types (mupdf_sys.rs)

**Current**: Raw C FFI bindings for MuPDF

**Target**: Pure Rust type definitions compatible with lopdf

```rust
// NEW: crates/core/src/document/pdf_rust/mod.rs
// Replaces mupdf_sys.rs

use lopdf::Object;

/// PDF Document wrapper
pub struct PdfContext(lopdf::Document);

/// Page wrapper  
pub struct PdfPage {
    page: lopdf::Object,
    doc: lopdf::ObjectId,
}

/// Bounds rectangle
#[derive(Clone, Copy, Debug)]
pub struct PdfRect {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}
```

**Task**: Create `document/pdf_rust/mod.rs` with type aliases and conversion traits

**AGENTS.md Compliance**:
- New file under 1000 lines ✅
- Add to module hierarchy ✅

---

### 3.1.2 Replace Document Context (mupdf/context.rs)

**Current**: `MuPdfContext::new()`, matrix operations

**Target**: lopdf Document wrapper

```rust
// Implementation pattern
use lopdf::Document;

pub struct PdfContext {
    doc: Document,
}

impl PdfContext {
    pub fn new(path: &Path) -> Result<Self, Error> {
        Ok(Self { doc: Document::load(path)? })
    }
    
    pub fn save(&mut self, path: &Path) -> Result<(), Error> {
        self.doc.save(path)
    }
}
```

**API Mapping**:

| MuPDF | Rust (lopdf) |
|-------|-------------|
| `MuPdfContext::new()` | `PdfContext::new()` |
| `ctx.save_document()` | `doc.save()` |
| `ctx.new_document()` | `Document::new()` |

**Tasks**:
1. Create `document/pdf_rust/context.rs`
2. Implement document lifecycle
3. Add save/export functionality

---

### 3.1.3 Replace Document Operations (mupdf/document.rs)

**Current**: Document-level operations (pages, metadata, outline)

**Target**: lopdf Document + custom helpers

```rust
// Implementation pattern
use lopdf::{Document, ObjectId};

pub struct PdfDocument {
    doc: Document,
}

impl PdfDocument {
    pub fn page_count(&self) -> usize {
        self.doc.get_pages().len()
    }
    
    pub fn get_page(&self, index: usize) -> Option<Page> {
        let pages = self.doc.get_pages();
        let (id, _) = pages.get(&index)?;
        Some(Page { page_id: *id, doc: &self.doc })
    }
    
    pub fn get_toc(&self) -> Vec<TocEntry> {
        // Extract from document outline
        self.doc.get_toc().unwrap_or_default()
    }
}
```

**API Mapping**:

| MuPDF | Rust (lopdf) | Notes |
|-------|--------------|-------|
| `doc.page_count()` | `page_count()` | Direct |
| `doc.get_page_size()` | `page.boundaries().MediaBox` | Different API |
| `doc.walk_toc()` | `get_toc()` | Simplified |
| `doc.is_encrypted()` | `is_encrypted()` | Via lopdf |

**Tasks**:
1. Create `document/pdf_rust/document.rs`
2. Implement page access
3. Add TOC extraction
4. Add metadata handling

---

## 4. Phase 2: Page Operations

### 4.2.1 Replace Page Operations (mupdf/page.rs)

**Current**: Page bounds, rotation, rendering

**Target**: printpdf + custom implementation

```rust
use lopdf::ObjectId;

pub struct PageRef<'a> {
    page_id: ObjectId,
    doc: &'a Document,
}

impl PageRef<'_> {
    pub fn media_box(&self) -> PdfRect {
        let page = self.doc.get_object(self.page_id).unwrap();
        // Extract MediaBox from page dictionary
    }
    
    pub fn rotation(&self) -> i32 {
        let page = self.doc.get_object(self.page_id).unwrap();
        // Extract Rotate from page dictionary
        page.get(b"Rotate").and_then(|r| r.as_integer().ok()).unwrap_or(0)
    }
}
```

**API Mapping**:

| MuPDF | Rust | Notes |
|-------|------|-------|
| `page.bound_box()` | `media_box()` | Custom extraction |
| `page.rotation()` | `rotation()` | Via dict |
| `page.resources()` | `resources()` | Complex |
| `page.run(device, matrix)` | **N/A** | Use printpdf |

**Tasks**:
1. Create `document/pdf_rust/page.rs`
2. Implement bounds extraction
3. Implement rotation handling

---

### 4.2.2 Replace Text Extraction (mupdf/text.rs)

**Current**: Full text extraction with word/character positions

**Target**: Custom implementation using lopdf + text analysis

```rust
use lopdf::{Document, ObjectId};

/// Text block from PDF
#[derive(Debug, Clone)]
pub struct TextBlock {
    pub text: String,
    pub bounds: PdfRect,
    pub block_type: BlockType,
}

/// Block type enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType {
    Text,
    Image,
    Other,
}

/// Page text with positions
pub struct TextPage {
    blocks: Vec<TextBlock>,
}

impl TextPage {
    /// Extract text with positions from a page
    pub fn from_page(doc: &Document, page_id: ObjectId) -> Result<Self> {
        let mut blocks = Vec::new();
        
        // Get page content stream
        let page = doc.get_object(page_id)?;
        let contents = page.get(b"Contents")?;
        
        // Parse content stream for text operators
        // This is complex - requires custom PDF content parsing
        // Alternative: use pdf-extract crate
        
        Ok(Self { blocks })
    }
    
    /// Get all text blocks as strings
    pub fn blocks(&self) -> &[TextBlock] {
        &self.blocks
    }
}
```

**Critical Note**: Text with accurate bounding boxes is NOT available in any pure Rust library. Custom implementation required.

**Tasks**:
1. Create `document/pdf_rust/text.rs`
2. Implement basic text extraction from lopdf
3. Note: Accurate word/character positions require MuPDF

---

## 5. Phase 3: Rendering

### 5.3.1 Replace Pixmap Rendering (mupdf/pixmap.rs)

**Current**: Page to Pixmap rendering for e-ink display

**Target**: printpdf or custom e-ink optimized renderer

```rust
use printpdf::*;

pub struct Pixmap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA pixels
}

impl Pixmap {
    /// Render page to RGBA pixmap
    pub fn from_page(page: &PageRef, dpi: f32) -> Result<Self> {
        let media_box = page.media_box();
        let width = ((media_box.width() * dpi) / 72.0) as u32;
        let height = ((media_box.height() * dpi) / 72.0) as u32;
        
        // Use printpdf for rendering
        let mut pdf-doc = PdfDocument::new();
        // ... rendering code
        
        Ok(Pixmap { width, height, data })
    }
}
```

**Critical Note**: E-ink optimized rendering (partial updates, ghosting reduction) requires custom implementation.

**Tasks**:
1. Create `document/pdf_rust/pixmap.rs`
2. Integrate printpdf for basic rendering
3. Add e-ink optimization layer (custom)

---

## 6. Phase 4: Advanced Features

### 6.4.1 Replace Links (mupdf/link.rs)

**Current**: PDF internal/external links

**Target**: Custom extraction from lopdf

```rust
use lopdf::Object;

pub struct Link {
    pub dest: LinkDest,
    pub rect: PdfRect,
}

pub enum LinkDest {
    Page(usize),  // Internal link: page number
    External(String), // External: URL
}

impl Link {
    pub fn from_page(doc: &Document, page_id: ObjectId) -> Result<Vec<Self>> {
        let mut links = Vec::new();
        // Extract from Annots dictionary or Links dictionary
        // Complex - requires PDF structure knowledge
        
        Ok(links)
    }
}
```

**Tasks**:
1. Create `document/pdf_rust/link.rs`
2. Implement link extraction (limited)

---

### 6.4.2 Replace Annotations (mupdf/annotation.rs)

**Current**: Full annotation CRUD

**Target**: Custom implementation (lopdf limited support)

```rust
pub struct Annotation {
    pub annot_type: AnnotType,
    pub rect: PdfRect,
    pub contents: Option<String>,
}

pub enum AnnotType {
    Highlight,
    Underline,
    Text,
    // ... other types
}
```

**Tasks**:
1. Create `document/pdf_rust/annotation.rs`
2. Note: Limited annotation editing available

---

### 6.4.3 Replace Outline/TOC (mupdf/outline.rs)

**Current**: Navigate TOC structure

**Target**: Via lopdf

```rust
use lopdf::Object;

pub struct Outline {
    pub title: String,
    pub dest: Option<OutlineDest>,
    pub children: Vec<Outline>,
}

impl Outline {
    pub fn from_doc(doc: &Document) -> Result<Vec<Self>> {
        // lopdf provides get_toc()
        doc.get_toc()
    }
}
```

**Tasks**:
1. Create `document/pdf_rust/outline.rs`
2. Map to lopdf TOC API

---

### 6.4.4 Replace Image Extraction (mupdf/image.rs)

**Current**: Extract embedded images

**Target**: Via lopdf + image crate

```rust
use lopdf::Object;

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub color_space: ColorSpace,
    pub data: Vec<u8>,
}

impl Image {
    pub fn from_page(doc: &Document, page_id: ObjectId) -> Result<Vec<Self>> {
        // Extract XObject images from page resources
        // Complex - requires stream parsing
        Ok(vec![])
    }
}
```

**Tasks**:
1. Create `document/pdf_rust/image.rs`
2. Implement image extraction

---

## 7. Phase 5: Integration

### 5.1 Create Unified Module

```rust
// crates/core/src/document/pdf_rust/mod.rs

pub mod context;
pub mod document;
pub mod page;
pub mod text;
pub mod pixmap;
pub mod link;
pub mod annotation;
pub mod outline;
pub mod image;

pub use context::PdfContext;
pub use document::PdfDocument;
pub use page::PageRef;
pub use text::{TextBlock, TextPage};
pub use pixmap::Pixmap;
```

### 5.2 Drop-in Replacement API

```rust
// Create trait for abstraction
pub trait PdfBackend {
    fn open(path: &Path) -> Result<Self>
    where Self: Sized;
    fn save(&mut self, path: &Path) -> Result()>;
    fn page_count(&self) -> usize;
    // ... etc
}
```

### 5.3 Feature Flag

```toml
# Cargo.toml
[features]
default = ["use_mupdf"]
use_lopdf = ["lopdf", "printpdf"]
```

---

## 8. Phase 5b: Search Feature (CRITICAL)

### Current Search Implementation

| Component | File | Lines | Purpose |
|-----------|------|-------|---------|
| Search struct | `reader_core.rs` | ~15 | Search state |
| Search handler | `reader_search_handler.rs` | 234 | Search management |
| Search bar UI | `search_bar.rs` | ~200 | Search interface |
| Search results UI | `reader_stubs.rs` | ~50 | Result display |

### MuPDF Search Usage

The current search relies on MuPDF's text extraction with positions:

```rust
// Current: Via MuPDF text extraction with bounding boxes
// mupdf/text.rs provides:
// - TextPage with word/character positions
// - TextBlock with exact bounds
// Search iterates through all characters to find matches
```

### Replacement Requirements

| Feature | MuPDF | lopdf | printpdf | Custom |
|---------|------|------|---------|--------|
| Text content | ✅ | ✅ | ✅ | ✅ |
| Word positions | ✅ | ❌ | ❌ | ❌ |
| Case-sensitive | ✅ | ✅ | ✅ | ✅ |
| Regex support | ✅ | ✅ | ✅ | ✅ |
| Whole word | ✅ | Via regex | Via regex | ✅ |
| Search direction | ✅ | Manual | Manual | ✅ |
| Highlight regions | ✅ | ❌ | ❌ | Custom |

### Search Backend Interface

```rust
// crates/core/src/document/pdf_rust/search.rs

use lopdf::Document;
use std::collections::VecDeque;

/// Search result with page location
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub page: usize,
    pub text: String,
    pub bounds: PdfRect,
}

/// PDF backend trait for search
pub trait PdfSearchBackend {
    /// Search for text in a specific page
    fn search_page(&self, page: usize, query: &str) -> Vec<SearchResult>;
    
    /// Search all pages
    fn search_all(&self, query: &str) -> Vec<SearchResult>;
    
    /// Get page text content
    fn get_page_text(&self, page: usize) -> Result<String>;
}
```

### Implementation Strategy

```rust
// Implementation using lopdf + regex

pub struct LopfdfSearch;

impl LopfdfSearch {
    pub fn search_page(&self, doc: &Document, page: usize, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        // Get page text
        let text = match doc.extract_text(&[page as u32]) {
            Ok(t) => t,
            Err(_) => return results,
        };
        
        // Use regex for flexible search
        let re = regex::Regex::new(&format!("(?i){}", regex::escape(query))).unwrap();
        
        // Find all matches - but NO positions available
        // This is the fundamental limitation
        for mat in re.find_iter(&text) {
            results.push(SearchResult {
                page,
                text: mat.as_str().to_string(),
                bounds: PdfRect::default(), // CANNOT get bounds without MuPDF
            });
        }
        
        results
    }
}
```

### Advanced Search with Custom Position Tracking

To replace MuPDF search, we need custom text analysis:

```rust
use lopdf::ObjectId;

/// Custom text word with position tracking
pub struct WordPosition {
    pub text: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub rect: PdfRect,
}

/// Extract words with approximate positions from content stream
pub fn extract_words(doc: &Document, page_id: ObjectId) -> Vec<WordPosition> {
    let mut words = Vec::new();
    
    // Get page content stream
    let page = doc.get_object(page_id).unwrap();
    let contents = page.get(b"Contents").unwrap();
    
    // Parse PDF text showing operators (Tj, TJ)
    // This is complex but possible:
    // 1. Parse content stream as operations
    // 2. Extract text and matrix from Tm (text matrix)
    // 3. Calculate approximate word positions from matrices
    
    // Simplified approach: Use font metrics for position estimation
    // Warning: Less accurate than MuPDF
    
    words
}
```

### Search Result Highlighting

The key challenge: **highlighting search results requires bounding boxes**.

| Approach | Accuracy | Complexity |
|----------|----------|------------|
| Character-based | High | Very High |
| Word-based | Medium | High |
| Line-based | Low | Medium |
| Page-based | None | Low |

**Recommendation**: Use "approximate" highlighting with text position estimation.

```rust
/// Estimate word position from text content (approximate)
pub fn estimate_word_bounds(
    text: &str,
    page_width: f32,
    font_size: f32,
    char_width_avg: f32,
) -> PdfRect {
    let text_width = text.len() as f32 * char_width_avg;
    PdfRect {
        x0: 0.0,
        y0: 0.0,
        x1: text_width,
        y1: font_size,
    }
}
```

### Search UI Integration

Update search handler to work with new backend:

```rust
// Update reader_search_handler.rs to use pdf_rust backend

pub fn search_all(&mut self, query: &str, backend: &dyn PdfSearchBackend) {
    self.search_results.clear();
    
    // Search each page
    for page in 0..backend.page_count() {
        let results = backend.search_page(page, query);
        self.search_results.extend(results);
    }
    
    // Update UI state
    if self.search_results.is_empty() {
        // Show "No results"
    } else {
        self.current_result_index = 0;
        // Navigate to first result
    }
}
```

### Feature Flag

```toml
# Cargo.toml - Select search implementation
[features]
default = ["use_mupdf_search"]
use_rust_search = ["lopdf", "regex"]
```

### Tasks for Search

1. Create `document/pdf_rust/search.rs`
   - Implement `SearchResult` struct
   - Implement `PdfSearchBackend` trait
   - Implement basic text search via lopdf

2. Create word position estimator
   - Parse content streams
   - Estimate positions from font metrics
   - Document accuracy limitations

3. Update search handler
   - Switch between backends via feature flag
   - Handle "no results" gracefully

4. Update UI
   - Show approximate highlights
   - Add warning for "position unavailable"

### Search Limitations (vs MuPDF)

| Feature | MuPDF | lopdf Replacement |
|---------|-------|-------------------|
| Exact word bounds | ✅ | ❌ (estimated) |
| Character bounds | ✅ | ❌ |
| Accurate highlighting | ✅ | Approximate |
| Performance | Fast | Medium |
| Unicode support | Full | Limited |

---

## 9. Implementation Order

| Phase | Module | Priority | Complexity |
|------|--------|----------|-------------|
| 1.1 | mod.rs + types | **P0** | Low |
| 1.2 | context.rs | **P0** | Low |
| 1.3 | document.rs | **P0** | Medium |
| 2.1 | page.rs | **P1** | Medium |
| 2.2 | text.rs | **P1** | High |
| 3.1 | pixmap.rs | **P0** | High |
| 4.1 | link.rs | **P2** | Medium |
| 4.2 | annotation.rs | **P2** | High |
| 4.3 | outline.rs | **P2** | Low |
| 4.4 | image.rs | **P2** | Medium |
| 4.5 | search.rs | **P0** | High |
| 5 | Integration | **P1** | Medium |

---

## 10. Known Limitations

| Feature | MuPDF | Replacement | Status |
|--------|------|-------------|--------|
| Text + bbox | ✅ | Custom | Needs dev |
| E-ink render | ✅ | Custom | Needs dev |
| Annot CRUD | ✅ | Limited | Partial |
| Forms | ✅ | ❌ | Not planned |
| Search | ✅ | ❌ | Not planned |
| Font subsetting | ✅ | ❌ | Not planned |

---

## 11. Files to Create

```
crates/core/src/document/pdf_rust/
├── mod.rs          # Module root + types (NEW)
├── context.rs     # Document context (NEW)
├── document.rs   # Document operations (NEW)
├── page.rs       # Page operations (NEW)
├── text.rs      # Text extraction (NEW)
├── pixmap.rs   # Rendering (NEW)
├── link.rs      # Links (NEW)
├── annotation.rs # Annotations (NEW)
├── outline.rs   # TOC (NEW)
├── image.rs     # Images (NEW)
└── search.rs   # Search functionality (NEW)
```

---

## 12. Test Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_open_pdf() {
        let doc = PdfContext::new("test.pdf").expect("Failed to open");
        assert!(doc.page_count() > 0);
    }
    
    #[test]
    fn test_render_page() {
        let ctx = PdfContext::new("test.pdf").expect("Failed to open");
        let pixmap = ctx.render_page(0, 150.0).expect("Failed to render");
        assert!(!pixmap.data.is_empty());
    }
}
```

---

## 13. Build Verification

```bash
# Build for ARM
cargo build --target arm-unknown-linux-gnueabihf -p plato-core

# Build for host
cargo build --target x86_64-unknown-linux-gnu -p plato-core

# Clippy
cargo clippy -- -D warnings
```

---

## 14. Completion Criteria

- [ ] All MuPDF usages replaced in code
- [ ] Zero warnings on ARM and host builds
- [ ] Tests pass for document operations
- [ ] PDF rendering functional (basic)
- [ ] Performance acceptable vs MuPDF baseline