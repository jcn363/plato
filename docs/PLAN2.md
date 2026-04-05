# Plan2: Replacing MuPDF with Pure Rust PDF Engine (No External Libraries)

## Executive Summary

This document outlines an ambitious plan to replace MuPDF with a completely custom, pure Rust PDF engine implemented from scratch without any external PDF libraries. This is a research and development project that will take significant time but provides full control over the PDF implementation.

**Important Warning**: This plan is HIGH RISK and HIGH EFFORT. It requires implementing the entire PDF specification (ISO 32000) from scratch. Consider PLAN.md (using lopdf/pdf_oxide) for a more practical approach.

---

## 1. Why No External Libraries?

### 1.1 Motivations

| Motivation | Description |
|------------|-------------|
| Full control | Complete ownership of implementation |
| No dependencies | Zero external crates for PDF functionality |
| Learning | Deep understanding of PDF format |
| Optimization | Custom optimizations for embedded devices |
| Flexibility | Modify任何aspect without library constraints |
| No FFI | Eliminate all unsafe FFI code entirely |

### 1.2 Challenges

| Challenge | Impact |
|-----------|--------|
| PDF specification complexity | ~1000 pages to implement fully |
| Rendering is extremely complex | Requires graphics engine |
| Time investment | 2-4 years for full implementation |
| Edge cases | Many PDF edge cases to handle |
| Font handling | Requires font rasterization |

---

## 2. Scope: What to Implement

### 2.1 Feature Parity Target

| Feature | Priority | Complexity | Notes |
|---------|----------|------------|-------|
| PDF parsing (file/memory) | P0 | High | Core foundation |
| Xref table | P0 | Medium | Document structure |
| Object reading | P0 | High | All PDF object types |
| Page loading | P0 | High | Page tree traversal |
| Text extraction | P1 | Very High | Content stream parsing |
| Rendering to bitmap | P1 | Extremely High | Graphics pipeline |
| Metadata | P1 | Low | Title, author, etc. |
| Links | P1 | Medium | Link annotations |
| Password protection | P2 | High | Encryption |
| PDF manipulation | P2 | High | Edit operations |
| Annotations | P2 | High | Read/create |
| Redaction | P3 | Very High | Complex content mod |
| EPUB reflow | P3 | Extremely High | HTML/CSS layout |

### 2.2 Minimum Viable Product (MVP)

For Phase 1, target:
1. Parse standard PDFs (non-encrypted)
2. Extract text content
3. Basic metadata
4. Page count and dimensions

---

## 3. PDF Specification Overview

### 3.1 Document Structure

```
PDF File Structure:
┌─────────────────────────────────────────┐
│ Header (%PDF-1.x)                       │
├─────────────────────────────────────────┤
│ Body                                   │
│   ├─ Object 1 (Catalog)                │
│   ├─ Object 2 (Pages)                  │
│   ├─ Object 3 (Page)                   │
│   ├─ ...                                │
│   └─ Object N                           │
├─────────────────────────────────────────┤
│ Xref Table                              │
│   (byte offset for each object)         │
├─────────────────────────────────────────┤
│ Trailer                                 │
│   (points to Catalog & Xref)            │
└─────────────────────────────────────────┘
```

### 3.2 PDF Object Types

| Type | Syntax | Example |
|------|--------|---------|
| Null | `null` | `null` |
| Boolean | `true` / `false` | `true` |
| Number | `123` / `45.67` | `42.5` |
| String | `(hello)` / `<hex>` | `(Test)` |
| Name | `/Name` | `/Type` |
| Array | `[1 2 3]` | `[ /Foo /Bar ]` |
| Dictionary | `<< >>` | `<< /Key value >>` |
| Stream | `stream...endstream` | Content streams |
| Reference | `1 0 R` | `5 0 R` |

### 3.3 Key PDF Concepts

| Concept | Description |
|---------|-------------|
| Catalog | Root object: `/Type /Catalog` |
| Pages | Page tree: `/Type /Pages` |
| Page | Individual page: `/Type /Page` |
| Content Stream | Page drawing commands |
| Resources | Fonts, images, patterns |
| Xref | Cross-reference table |
| Encrypt | Encryption dictionary |

---

## 4. Implementation Architecture

### 4.1 Module Structure

```
crates/core/src/document/pdf_engine/
├── mod.rs              # Main entry point
├── parser/
│   ├── mod.rs
│   ├── lexer.rs        # Tokenizer
│   ├── tokenizer.rs    # PDF token extraction
│   ├── object.rs       # Object parsing
│   ├── xref.rs         # Xref table parser
│   └── stream.rs       # Stream decompression
├── objects/
│   ├── mod.rs
│   ├── value.rs        # PdfValue enum
│   ├── dict.rs         # Dictionary handling
│   ├── array.rs        # Array handling
│   └── reference.rs    # Object references
├── document/
│   ├── mod.rs
│   ├── catalog.rs      # Document catalog
│   ├── page_tree.rs    # Page tree navigation
│   └── page.rs         # Page object
├── content/
│   ├── mod.rs
│   ├── interpreter.rs  # Execute content stream
│   ├── operators.rs    # Graphics operators
│   └── text.rs         # Text extraction
├── render/
│   ├── mod.rs
│   ├── device.rs       # Render device
│   ├── text_render.rs  # Text rasterization
│   └── graphics.rs     # Graphics pipeline
├── font/
│   ├── mod.rs
│   ├── builtin.rs      # Built-in fonts
│   └── subfont.rs      # Font subsetting
└── security/
    ├── mod.rs
    ├── encryption.rs   # PDF encryption
    └── auth.rs        # Password handling
```

### 4.2 Core Data Types

```rust
// Object/value types
pub enum PdfValue {
    Null,
    Bool(bool),
    Integer(i64),
    Real(f64),
    String(Vec<u8>),
    Name(Vec<u8>),
    Array(Vec<PdfValue>),
    Dict(PdfDict),
    Stream(PdfStream),
    Reference(ObjId),
}

pub struct PdfDict {
    entries: BTreeMap<Vec<u8>, PdfValue>,
}

pub struct PdfStream {
    dict: PdfDict,
    data: Vec<u8>,
}

pub struct ObjId {
    pub num: u32,
    pub gen: u16,
}

pub struct PdfDocument {
    version: f32,
    objects: BTreeMap<ObjId, PdfValue>,
    xref: XrefTable,
    catalog: PdfValue,
}
```

---

## 5. Implementation Phases

### Phase 1: Foundation (Months 1-4)

**Goal**: Parse basic PDFs, extract metadata, count pages

#### 1.1 Lexer/Tokenizer (Weeks 1-3)

```rust
// Token types
pub enum Token {
    Comment(Vec<u8>),
    Integer(i64),
    Real(f64),
    String(Vec<u8>),
    HexString(Vec<u8>),
    Name(Vec<u8>),
    ArrayOpen,
    ArrayClose,
    DictOpen,
    DictClose,
    Stream,
    EndStream,
    Obj,
    EndObj,
    R,  // Reference
    True,
    False,
    Null,
    Eof,
}

pub struct Lexer {
    input: Vec<u8>,
    position: usize,
}

impl Lexer {
    pub fn next_token(&mut self) -> Result<Token, PdfError> {
        // Skip whitespace
        // Parse next token based on first character
    }
}
```

#### 1.2 Object Parser (Weeks 4-6)

```rust
pub struct ObjectParser<'a> {
    lexer: &'a mut Lexer,
}

impl<'a> ObjectParser<'a> {
    pub fn parse(&mut self) -> Result<PdfValue, PdfError> {
        let token = self.lexer.peek()?;
        match token {
            Token::Integer => self.parse_number(),
            Token::String => self.parse_string(),
            Token::Name => self.parse_name(),
            Token::ArrayOpen => self.parse_array(),
            Token::DictOpen => self.parse_dict(),
            Token::Reference => self.parse_reference(),
            // ...
        }
    }
}
```

#### 1.3 Xref Table (Weeks 7-8)

```rust
pub struct XrefEntry {
    pub offset: u64,
    pub generation: u16,
    pub in_use: bool,
}

pub struct XrefTable {
    pub entries: Vec<XrefEntry>,
    pub start_object: u32,
}

impl XrefTable {
    pub fn parse(lexer: &mut Lexer) -> Result<XrefTable, PdfError> {
        // Parse xref table
        // Handle both original and hybrid PDFs
    }
}
```

#### 1.4 Document Structure (Weeks 9-12)

```rust
pub struct PdfEngine {
    document: PdfDocument,
}

impl PdfEngine {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<PdfEngine, PdfError> {
        // 1. Read entire file
        // 2. Find trailer
        // 3. Parse xref table
        // 4. Load objects
        // 5. Get catalog
    }

    pub fn pages_count(&self) -> usize {
        // Navigate to Pages tree
    }

    pub fn page_dimensions(&self, index: usize) -> Option<(f32, f32)> {
        // Get page /MediaBox
    }
}
```

#### 1.5 Phase 1 Deliverables

- [ ] Lexer for PDF tokens
- [ ] Object parser (all PDF object types)
- [ ] Xref table parser
- [ ] Document loader
- [ ] Page count
- [ ] Page dimensions
- [ ] Basic metadata (title, author)

**Testing**: Load 50+ PDFs, verify page count matches

---

### Phase 2: Text Extraction (Months 5-10)

**Goal**: Extract text from PDF content streams

#### 2.1 Content Stream Parsing (Weeks 13-18)

```rust
// Graphics operators (subset)
#[derive(Debug)]
pub enum Operator {
    // Graphics state
    q,  // push
    Q,  // pop
    cm, // concatMatrix
    
    // Path construction
    m,  // moveto
    l,  // lineto
    c,  // curveto
    re, // rectangle
    
    // Path painting
    S,  // stroke
    f,  // fill
    B,  // fill+stroke
    
    // Text
    Tj, // show text
    Tj_, // show text (spaced)
    Tf, // set font
    Td, // text translate
    TD, // text move
    Tm, // text matrix
    ET, // end text
    BT, // begin text
    
    // Images
    Do, // paint XObject
}

pub struct ContentInterpreter {
    stack: Vec<f64>,
    text_state: TextState,
    graphics_state: GraphicsState,
}

pub struct GraphicsState {
    pub ctm: Matrix,      // Current transformation matrix
    pub line_width: f64,
    pub stroke_color: Color,
    pub fill_color: Color,
    // ...
}

pub struct TextState {
    pub font: Option<FontRef>,
    pub font_size: f64,
    pub text_matrix: Matrix,
    pub line_matrix: Matrix,
}
```

#### 2.2 Text Extraction (Weeks 19-24)

```rust
pub struct TextExtractor {
    current_text: String,
    positions: Vec<TextPosition>,
}

impl ContentInterpreter for TextExtractor {
    fn op_BT(&mut self) { /* Start text block */ }
    
    fn op_Tj(&mut self, args: &[f64]) {
        // Extract text from string argument
        // Store position for word detection
    }
    
    fn op_ET(&mut self) {
        // End text block, finalize text
    }
}

pub struct TextPosition {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
```

#### 2.3 Font Handling (Weeks 25-30)

```rust
pub enum FontType {
    Type0,       // Type 0 (composite)
    Type1,       // Type 1
    Type2,       // Type 2 (TrueType)
    Type3,       // Type 3 (custom)
    TrueType,
}

pub trait FontBackend {
    fn glyph_id(&self, c: char) -> Option<u16>;
    fn width(&self, glyph_id: u16) -> f64;
    fn rasterize(&self, glyph_id: u16, size: f64) -> RasterizedGlyph;
}

pub struct BuiltinFont {
    pub name: String,
    pub encoding: Vec<u8>,  // 256 entries
    widths: Vec<f64>,
}

impl BuiltinFont {
    pub fn new(subtype: &str) -> Option<BuiltinFont> {
        // Implement: Times-Roman, Helvetica, Courier, Symbol, ZapfDingbats
    }
}
```

#### 2.4 Phase 2 Deliverables

- [ ] Content stream parser
- [ ] Graphics state machine
- [ ] Text extraction
- [ ] Basic font handling
- [ ] Word/line detection
- [ ] Link extraction

**Testing**: Compare text extraction with MuPDF output on 100+ PDFs

---

### Phase 3: Rendering (Months 11-18)

**Goal**: Render pages to pixmaps

#### 3.1 Graphics Pipeline (Weeks 31-38)

```rust
pub struct RenderDevice {
    pub pixmap: Pixmap,
    pub ctm: Matrix,
    clip_rect: Option<Rect>,
}

impl RenderDevice {
    pub fn new(width: u32, height: u32, samples: usize) -> Self {
        // Initialize pixmap buffer
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        // Clipping check
        // Write to pixmap
    }

    pub fn draw_image(&mut self, image: &PdfImage, matrix: Matrix) {
        // Apply transformation
        // Resample if needed
        // Write to pixmap
    }

    pub fn stroke_path(&mut self, path: &Path, style: &StrokeStyle) {
        // Bresenham or anti-aliased line drawing
    }
}
```

#### 3.2 Text Rendering (Weeks 39-46)

```rust
pub struct TextRenderer {
    font_cache: HashMap<FontRef, Box<dyn FontBackend>>,
}

impl TextRenderer {
    pub fn render_text(
        &mut self,
        text: &str,
        font: &FontRef,
        size: f64,
        matrix: Matrix,
        device: &mut RenderDevice,
    ) {
        for c in text.chars() {
            let gid = font.glyph_id(c)?;
            let glyph = font.rasterize(gid, size);
            
            // Transform position
            let pos = matrix * Point::new(0.0, 0.0);
            
            // Blit glyph to pixmap (with subpixel positioning)
            device.draw_glyph(&glyph, pos, font.color());
        }
    }
}
```

#### 3.3 Image Handling (Weeks 47-52)

```rust
pub enum ImageFormat {
    JPEG,
    PNG,
    CCITT,  // Fax
    JBIG2,
    DCT,
}

pub struct PdfImage {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub data: Vec<u8>,
    pub color_space: ColorSpace,
}

impl PdfImage {
    pub fn from_xobject(obj: &PdfDict) -> Result<PdfImage, PdfError> {
        // Check /Subtype (Image)
        // Get /Width, /Height
        // Get /ColorSpace
        // Decode based on /Filter
    }
}
```

#### 3.4 Phase 3 Deliverables

- [ ] Render device
- [ ] Path rendering (fill/stroke)
- [ ] Text rendering with fonts
- [ ] Image handling (JPEG, PNG)
- [ ] Matrix transformations
- [ ] Page rendering to pixmap

**Testing**: Visual comparison with MuPDF output, performance benchmarks

---

### Phase 4: Advanced Features (Months 19-24)

#### 4.1 PDF Manipulation (Weeks 53-60)

```rust
pub struct PdfEditor {
    document: PdfDocument,
}

impl PdfEditor {
    pub fn delete_page(&mut self, index: usize) -> Result<(), PdfError> {
        // 1. Remove page from page tree
        // 2. Remove page object
        // 3. Update page count
    }

    pub fn insert_page(&mut self, after: usize, page: Page) -> Result<(), PdfError> {
        // 1. Create page object
        // 2. Insert into page tree
        // 3. Update parent /Count
    }

    pub fn rotate_page(&mut self, index: usize, degrees: i32) -> Result<(), PdfError> {
        // Update /Rotate entry
    }

    pub fn save(&mut self, path: &Path) -> Result<(), PdfError> {
        // 1. Rebuild xref table
        // 2. Write objects
        // 3. Write xref
        // 4. Write trailer
    }
}
```

#### 4.2 Annotations (Weeks 61-68)

```rust
pub enum AnnotationType {
    Text,
    Link,
    FreeText,
    Line,
    Square,
    Circle,
    Polygon,
    Highlight,
    Underline,
    StrikeOut,
    Ink,
    Popup,
}

pub struct Annotation {
    pub annot_type: AnnotationType,
    pub rect: Rect,
    pub contents: Option<String>,
    pub color: Option<Color>,
}
```

#### 4.3 Encryption (Weeks 69-76)

```rust
pub struct PdfEncryption {
    pub algorithm: EncryptionAlgorithm,
    pub key: Vec<u8>,
}

pub enum EncryptionAlgorithm {
    None,
    RC4_40,
    RC4_128,
    AES_128,
    AES_256,
}

impl PdfEncryption {
    pub fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        // Decrypt based on algorithm
    }
}
```

#### 4.4 Phase 4 Deliverables

- [ ] Page deletion/insertion
- [ ] Page rotation
- [ ] Document saving
- [ ] Basic annotations
- [ ] Password protection
- [ ] Linearization

---

### Phase 5: Optimization & Cleanup (Months 25-30)

#### 5.1 Performance Optimization

- Progressive loading (parse xref first)
- Lazy object loading
- Page caching
- Memory pooling

#### 5.2 Edge Case Handling

- Corrupted PDFs
- Non-standard encodings
- Incremental updates
- Large files (>100MB)

#### 5.3 Final Deliverables

- Full feature parity with current MuPDF usage
- Complete test suite
- Performance benchmarks
- Documentation

---

## 6. Technical Details

### 6.1 Stream Compression

```rust
pub enum Filter {
    ASCII85Decode,
    ASCIIHexDecode,
    FlateDecode,
    LZWDecode,
    DCTDecode,
    JPXDecode,
    CCITTFaxDecode,
    RunLengthDecode,
}

impl Filter {
    pub fn decode(&self, data: &[u8], params: Option<&PdfDict>) -> Result<Vec<u8>, PdfError> {
        match self {
            Filter::FlateDecode => Ok(decompress_flate(data)?),
            Filter::ASCII85Decode => Ok(decode_ascii85(data)?),
            // ...
        }
    }
}
```

### 6.2 Color Spaces

```rust
pub enum ColorSpace {
    DeviceGray,
    DeviceRGB,
    DeviceCMYK,
    CalGray,
    CalRGB,
    Lab,
    ICCBased,
    Indexed,
    Pattern,
    Separation,
}

impl ColorSpace {
    pub fn to_rgb(&self, components: &[f64]) -> [u8; 3] {
        match self {
            ColorSpace::DeviceGray => {
                let v = (components[0] * 255.0) as u8;
                [v, v, v]
            }
            ColorSpace::DeviceRGB => [
                (components[0] * 255.0) as u8,
                (components[1] * 255.0) as u8,
                (components[2] * 255.0) as u8,
            ],
            // ...
        }
    }
}
```

### 6.3 Matrix Operations

```rust
#[derive(Clone, Copy, Debug)]
pub struct Matrix {
    pub a: f64, pub b: f64,
    pub c: f64, pub d: f64,
    pub e: f64, pub f: f64,
}

impl Matrix {
    pub fn identity() -> Matrix {
        Matrix { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 }
    }

    pub fn scale(sx: f64, sy: f64) -> Matrix {
        Matrix { a: sx, b: 0.0, c: 0.0, d: sy, e: 0.0, f: 0.0 }
    }

    pub fn translate(tx: f64, ty: f64) -> Matrix {
        Matrix { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: tx, f: ty }
    }

    pub fn rotate(angle: f64) -> Matrix {
        let cos = angle.cos();
        let sin = angle.sin();
        Matrix { a: cos, b: sin, c: -sin, d: cos, e: 0.0, f: 0.0 }
    }

    pub fn multiply(&self, other: &Matrix) -> Matrix {
        Matrix {
            a: self.a * other.a + self.b * other.c,
            b: self.a * other.b + self.b * other.d,
            c: self.c * other.a + self.d * other.c,
            d: self.c * other.b + self.d * other.d,
            e: self.e * other.a + self.f * other.c + other.e,
            f: self.e * other.b + self.f * other.d + other.f,
        }
    }

    pub fn transform_point(&self, x: f64, y: f64) -> (f64, f64) {
        (self.a * x + self.c * y + self.e, self.b * x + self.d * y + self.f)
    }
}
```

---

## 7. Testing Strategy

### 7.1 Unit Tests

```rust
#[cfg(test)]
mod lexer_tests {
    #[test]
    fn test_integer_token() { }
    #[test]
    fn test_string_token() { }
    #[test]
    fn test_name_token() { }
}

#[cfg(test)]
mod matrix_tests {
    #[test]
    fn test_identity() { }
    #[test]
    fn test_multiply() { }
    #[test]
    fn test_transform() { }
}
```

### 7.2 Integration Tests

| Test Set | Count | Purpose |
|----------|-------|---------|
| Basic PDFs | 100 | Parse, page count |
| Text extraction | 50 | Compare with MuPDF |
| Rendering | 50 | Visual comparison |
| Edge cases | 20 | Corrupted, encrypted |
| Performance | 10 | Timing benchmarks |

### 7.3 Regression Tests

- Compare text output with MuPDF
- Compare rendered images pixel-by-pixel (allow tolerance)
- Verify page count matches across 500+ PDFs

---

## 8. Risk Analysis

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Specification gaps | High | High | Study Adobe PDF reference |
| Font complexity | Very High | High | Implement subset, defer full |
| Rendering quality | High | Medium | Allow minor differences |
| Performance | Medium | High | Profile and optimize |
| Edge cases | High | Medium | Extensive testing |
| Timeline | High | High | Buffer time per phase |

---

## 9. Timeline Summary

| Phase | Duration | Focus | Key Deliverables |
|-------|----------|-------|------------------|
| Phase 1 | 4 months | Foundation | Parsing, page count, metadata |
| Phase 2 | 6 months | Text extraction | Words, lines, fonts |
| Phase 3 | 8 months | Rendering | Pixmap output |
| Phase 4 | 6 months | Advanced | Editing, encryption |
| Phase 5 | 6 months | Cleanup | Optimization, docs |
| **Total** | **30 months** | | **~2.5 years** |

---

## 10. Resources Required

### 10.1 Team

| Role | Count | Duration |
|------|-------|----------|
| Lead Developer | 1 | 30 months |
| Senior Engineer (review) | 1 | 20% time |
| QA Engineer | 1 | 30% time |

### 10.2 References

1. **ISO 32000-1:2008** — PDF 1.7 specification (official)
2. **PDF Reference 1.7** — Adobe's documentation
3. **PDF.js source** — Reference implementation
4. **MuPDF source** — Reference implementation

---

## 11. Comparison with PLAN.md

| Aspect | PLAN.md (Libraries) | PLAN2.md (From Scratch) |
|--------|---------------------|-------------------------|
| Timeline | 4.5 months | 30 months |
| Dependencies | lopdf, pdf_oxide | None |
| Risk | Low | Very High |
| Effort | 1 developer | 1+ developers |
| Full control | Partial | Complete |
| FFI | Kept (rendering) | Eliminated |
| Complexity | Manageable | Extremely High |

---

## 12. Decision Recommendation

**DO NOT pursue PLAN2.md for production use.**

PLAN2.md should only be considered if:
1. This is a research/learning project
2. You have 2-3 years of dedicated resources
3. Full control over PDF implementation is critical
4. You accept high risk of feature gaps

**For production**: Use PLAN.md (lopdf + pdf_oxide hybrid approach).

---

## 13. Summary

Implementing a PDF engine from scratch in pure Rust is a massive undertaking requiring:
- 30+ months of development
- Deep PDF specification knowledge
- Custom graphics rendering
- Font rasterization
- Extensive testing

This plan is documented for completeness but is not recommended for practical implementation. The hybrid approach in PLAN.md provides 80% of the benefits with 10% of the effort.