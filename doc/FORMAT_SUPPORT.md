# Format Support for Plato

> **Last Updated**: 2026-04-22
> **Related Documents**: [PDF_FEATURES.md](./PDF_FEATURES.md) | [BUILD.md](./BUILD.md)

This document catalogs the document and image format support in Plato, detailing implementation status, technical decisions, and dependencies.

## Quick Reference

 | Format Type | Format          | Status          | Library/Crate  | Notes                               |
 |-------------|-----------------|-----------------|----------------|-------------------------------------|
 | Document    | PDF             | ✅ Implemented  | PDFPurr, lopdf | Full PDF rendering and manipulation |
 | Document    | EPUB            | ✅ Implemented  | Custom         | HTML/CSS rendering engine           |
 | Document    | HTML            | ✅ Implemented  | Custom         | HTML/CSS rendering engine           |
 | Document    | DJVU            | ✅ Implemented  | djvu-rs        | Basic page navigation               |
 | Image       | PNG             | ✅ Implemented  | image crate    | Standard format support             |
 | Image       | JPEG            | ✅ Implemented  | image crate    | Standard format support             |
 | Image       | GIF             | ✅ Implemented  | image crate    | Standard format support             |
 | Image       | BMP             | ✅ Implemented  | image crate    | Standard format support             |
 | Image       | WebP            | ✅ Implemented  | image crate    | Standard format support             |
 | Image       | TGA             | ✅ Implemented  | image crate    | Standard format support             |
 | Image       | JPEG 2000 (JP2) | 🚧 Placeholder  | openjp2        | Requires openjp2 integration        |

## Document Formats

### PDF

**Status**: ✅ IMPLEMENTED - Full PDF support via PDFPurr

**Implementation Details**:

- Rendering: PDFPurr (pure Rust PDF library)
- Manipulation: lopdf (pure Rust PDF operations)
- Features: Annotations, redaction, page manipulation, progressive loading, resource extraction

**Dependencies**:

- `pdfpurr` - Pure Rust PDF rendering
- `lopdf` - Pure Rust PDF manipulation

**File Extensions**: `.pdf`

**See Also**: [PDF_FEATURES.md](./PDF_FEATURES.md) for detailed PDF feature documentation

### EPUB

**Status**: ✅ IMPLEMENTED - Custom EPUB rendering engine

**Implementation Details**:

- Custom HTML/CSS rendering engine optimized for e-ink
- NCX/Navigation parsing for table of contents
- Support for embedded fonts and stylesheets

**File Extensions**: `.epub`

### HTML

**Status**: ✅ IMPLEMENTED - Custom HTML rendering engine

**Implementation Details**:

- Custom HTML/CSS rendering engine optimized for e-ink
- DOM parsing, layout, text shaping, line breaking
- Support for inline CSS and external stylesheets

**File Extensions**: `.html`, `.htm`

### DJVU

**Status**: ✅ IMPLEMENTED - Basic DJVU support via djvu-rs

**Implementation Details**:

- Module: `document/djvu.rs`
- Library: `djvu-rs` (Rust bindings for DjVuLibre)
- Features: File opening, page dimension extraction, page count
- Rendering: Stub implementation (requires further integration with rendering pipeline)

**Current Limitations**:

- Text extraction not implemented
- Pixmap rendering not implemented (placeholder)
- Table of contents not supported (DJVU typically doesn't have TOC)
- Page navigation works via Document trait

**Dependencies**:

- `djvu-rs` - Rust bindings for DjVuLibre

**File Extensions**: `.djvu`, `.djv`

**Usage Example**:

```rust
use crate::document::DjvuDocument;

let doc = DjvuDocument::new(&path)?;
let page_count = doc.pages_count();
let dims = doc.dims(0)?;
```

**Future Enhancements**:

- Full page rendering integration
- Text layer extraction
- Hyperlink support
- Metadata extraction

## Image Formats

### Standard Formats

**Status**: ✅ IMPLEMENTED - Full support via image crate

**Supported Formats**:

- PNG (Portable Network Graphics)
- JPEG (Joint Photographic Experts Group)
- GIF (Graphics Interchange Format)
- BMP (Bitmap)
- WebP (Web Picture)
- TGA (Truevision TGA)

**Implementation**:

- Library: `image` crate (version 0.25.10)
- Features: PNG, JPEG, GIF, BMP, WebP, TGA
- Used in: cover_editor.rs, framebuffer/image.rs

**Usage**:

```rust
use image;

let img = image::open(path)?;
let rgba = img.to_rgba8();
```

### JPEG 2000 (JP2)

**Status**: 🚧 PLACEHOLDER - Module created, requires openjp2 integration

**Implementation Details**:

- Module: `image_formats/jp2.rs`
- Library: `openjp2` (Rust bindings for OpenJPEG)
- Current Status: Placeholder implementation
- Reason: The `image` crate does not have a `jp2` feature, so direct JPEG 2000 support requires the `openjp2` crate

**Current Limitations**:

- Decoding not yet implemented (returns error)
- Requires openjp2 integration for full functionality
- File detection works (is_jp2 function)

**Dependencies**:

- `openjp2` - Rust bindings for OpenJPEG library (already in Cargo.toml)

**File Extensions**: `.jp2`, `.jpx`, `.j2k`

**Usage Example**:

```rust
use crate::image_formats::jp2::{is_jp2, load_jp2};

if is_jp2(&path) {
    let image = load_jp2(&path)?;
}
```

**Future Work**:

- Integrate openjp2::decode for JPEG 2000 decoding
- Convert decoded data to DynamicImage for rendering
- Add to image loading workflow in cover_editor.rs and framebuffer/image.rs

## Technical Architecture

**Document Processing Stack**:

```text
Document Open → file_kind() → Format Dispatch
                            ├─→ EPUB → EpubDocument
                            ├─→ HTML → HtmlDocument
                            ├─→ PDF  → PdfOpener → PdfDocument
                            └─→ DJVU → DjvuDocument
```

**Image Processing Stack**:

```text
Image Open → Format Detection → Image Crate
                              ├─→ PNG, JPEG, GIF, etc. (supported)
                              └─→ JP2 (placeholder, requires openjp2)
```

**Dependencies**:

| Feature          | Dependency | Purpose                         |
|------------------|------------|---------------------------------|
| PDF Rendering    | pdfpurr    | Pure Rust PDF rendering         |
| PDF Manipulation | lopdf      | Pure Rust PDF operations        |
| DJVU Support     | djvu-rs    | Rust bindings for DjVuLibre     |
| JPEG 2000        | openjp2    | Rust bindings for OpenJPEG      |
| Standard Images  | image      | Rust image processing library   |
| Compression      | bzip2      | BZIP2 compression/decompression |
| XML Processing   | quick-xml  | Enhanced XML parsing            |

## Design Philosophy

- **Pure Rust**: Prefer Rust-native libraries over C bindings where possible
- **Memory Efficiency**: Optimize for Kobo's 256MB RAM constraint
- **E-ink Optimization**: Prioritize e-ink display characteristics
- **Modular Design**: Keep format-specific code in separate modules
- **Progressive Enhancement**: Implement basic support first, add advanced features incrementally

## Future Format Support

Potential formats for future implementation:

1. **CBZ/CBR** - Comic book archives (ZIP/RAR based)
2. **MOBI** - Mobipocket format (via PDFPurr conversion)
3. **FB2** - FictionBook format (XML-based)
4. **DOCX** - Word documents (via pandoc conversion)
5. **TIFF** - Tagged Image File Format (via image crate or libtiff)

Priority should be based on user demand and implementation complexity.
