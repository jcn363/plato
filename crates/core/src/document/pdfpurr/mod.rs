//! Minimal PDFPurr wrapper types for compatibility
//!
//! PDFPurr's API is different from MuPDF, so we provide minimal type definitions
//! that match the expected interface for pdf.rs.

use std::path::Path;
use anyhow::{Result, bail};

/// Stub MuPDF Context type
pub struct MuPdfContext {
    _private: (),
}

impl MuPdfContext {
    pub fn new() -> Result<Self> {
        Ok(MuPdfContext { _private: () })
    }

    pub fn set_user_css(&self, _css: &str) {
        // PDFPurr doesn't need user CSS
    }

    pub fn device_gray(&self) -> () {
        // Stub
    }

    pub fn device_rgb(&self) -> () {
        // Stub
    }

    pub fn open_document<P: AsRef<Path>>(&self, _path: P) -> Option<PdfPurrDocument> {
        Some(PdfPurrDocument { _private: () })
    }

    pub fn open_document_memory(&self, _magic: &str, _buf: &[u8]) -> Option<PdfPurrDocument> {
        Some(PdfPurrDocument { _private: () })
    }
}

/// Stub Document type - uses actual PDFPurr Document internally
pub struct Document {
    // Placeholder - will be implemented with actual PDFPurr integration
    _private: (),
}

/// Type alias for compatibility
pub type PdfPurrDocument = Document;

impl Document {
    pub fn open<P: AsRef<Path>>(_path: P) -> Result<Self> {
        Ok(Document { _private: () })
    }

    pub fn from_bytes(_data: &[u8]) -> Result<Self> {
        Ok(Document { _private: () })
    }

    pub fn object_count(&self) -> usize {
        0 // Placeholder
    }

    pub fn load_page(&self, _index: i32) -> Result<Page> {
        Ok(Page { _private: () })
    }

    pub fn is_reflowable(&self) -> bool {
        false
    }

    pub fn layout(&mut self, _width: f32, _height: f32) {
        // Stub
    }

    pub fn title(&self) -> Option<String> {
        None
    }

    pub fn author(&self) -> Option<String> {
        None
    }

    pub fn lookup_metadata(&self, _key: &str) -> Option<String> {
        None
    }

    pub fn metadata(&self) -> Option<String> {
        None
    }

    pub fn page_count(&self) -> usize {
        0 // Placeholder
    }

    pub fn load_outline(&self) -> Option<Outline> {
        Some(Outline { _private: () })
    }

    pub fn needs_password(&self) -> bool {
        false
    }
}

/// Stub Page type
pub struct Page {
    _private: (),
}

impl Page {
    pub fn to_text_page(&self, _options: Option<&()>) -> Option<TextPage> {
        Some(TextPage { _private: () })
    }

    pub fn load_links(&self) -> Option<Link> {
        Some(Link { _private: () })
    }

    pub fn render_pixmap(&self, _matrix: f32, _color_space: PixmapFormat, _flags: i32) -> Result<PdfPurrPixmap> {
        Ok(PdfPurrPixmap { _private: () })
    }

    pub fn dims(&self) -> (f32, f32) {
        (600.0, 800.0) // Placeholder
    }

    pub fn width(&self) -> f32 {
        600.0
    }

    pub fn height(&self) -> f32 {
        800.0
    }

    pub fn media_box(&self) -> FzRect {
        FzRect {
            x0: 0.0,
            y0: 0.0,
            x1: 600.0,
            y1: 800.0,
        }
    }

    pub fn search(&self, _needle: &str) -> Option<Vec<FzQuad>> {
        Some(Vec::new())
    }

    pub fn images(&self) -> Option<Vec<FzRect>> {
        Some(Vec::new())
    }

    pub fn char_count(&self) -> usize {
        0
    }
}

/// Stub TextPage type
pub struct TextPage {
    _private: (),
}

impl TextPage {
    pub fn blocks(&self) -> Vec<TextBlock> {
        Vec::new()
    }
}

/// Stub TextBlock type
pub struct TextBlock {
    _private: (),
}

impl TextBlock {
    pub fn kind(&self) -> i32 {
        0
    }

    pub fn bbox(&self) -> FzRect {
        FzRect::default()
    }

    pub fn lines(&self) -> Vec<TextLine> {
        Vec::new()
    }

    pub fn chars(&self) -> Vec<TextChar> {
        Vec::new()
    }
}

/// Stub TextLine type
pub struct TextLine {
    _private: (),
}

impl TextLine {
    pub fn bbox(&self) -> FzRect {
        FzRect::default()
    }

    pub fn chars(&self) -> Vec<TextChar> {
        Vec::new()
    }
}

/// Stub TextChar type
pub struct TextChar {
    _private: (),
}

impl TextChar {
    pub fn char_code(&self) -> i32 {
        0
    }

    pub fn quad(&self) -> FzQuad {
        FzQuad::default()
    }

    pub fn origin(&self) -> i32 {
        0
    }
}

/// Stub Link type
pub struct Link {
    _private: (),
}

impl Link {
    pub fn uri(&self) -> String {
        String::new()
    }

    pub fn rect(&self) -> FzRect {
        FzRect::default()
    }

    pub fn next(&self) -> Option<Link> {
        None
    }
}

/// Stub Outline type
pub struct Outline {
    _private: (),
}

impl Outline {
    pub fn clone_outline(&self) -> Outline {
        Outline { _private: () }
    }

    pub fn page(&self) -> FzLocation {
        FzLocation { chapter: 0, page: 0 }
    }

    pub fn uri(&self) -> Option<String> {
        None
    }

    pub fn next(&self) -> Option<Outline> {
        None
    }

    pub fn title(&self) -> String {
        String::new()
    }

    pub fn down(&self) -> Option<Outline> {
        None
    }
}

/// Stub FzRect type
#[derive(Debug, Clone, Default)]
pub struct FzRect {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl FzRect {
    pub fn default() -> Self {
        FzRect {
            x0: 0.0,
            y0: 0.0,
            x1: 0.0,
            y1: 0.0,
        }
    }
}

// Implement From for Boundary compatibility
impl From<FzRect> for crate::geom::Boundary {
    fn from(rect: FzRect) -> Self {
        crate::geom::Boundary {
            min: crate::geom::Vec2::new(rect.x0, rect.y0),
            max: crate::geom::Vec2::new(rect.x1, rect.y1),
        }
    }
}

/// Stub FzPoint type
#[derive(Debug, Clone, Default)]
pub struct FzPoint {
    pub x: f32,
    pub y: f32,
}

/// Stub FzQuad type
#[derive(Debug, Clone, Default)]
pub struct FzQuad {
    pub ul: FzPoint,
    pub ur: FzPoint,
    pub ll: FzPoint,
    pub lr: FzPoint,
}

/// Stub FzLocation type
#[derive(Debug, Clone, Default)]
pub struct FzLocation {
    pub chapter: i32,
    pub page: i32,
}

/// Stub PixmapFormat type
#[derive(Debug, Clone, Copy)]
pub enum PixmapFormat {
    Grayscale,
    RGB,
}

/// Stub Pixmap type (renamed to avoid conflict with crate::framebuffer::Pixmap)
pub struct PdfPurrPixmap {
    _private: (),
}

/// Stub rect_from_quad function
pub fn rect_from_quad(_quad: FzQuad) -> FzRect {
    FzRect::default()
}

/// Stub union_rect function
pub fn union_rect(_a: FzRect, _b: FzRect) -> FzRect {
    FzRect::default()
}

/// Stub scale function
pub fn scale(_x: f32, _y: f32) -> f32 {
    1.0
}

/// Stub FZ_PAGE_BLOCK_IMAGE constant
pub const FZ_PAGE_BLOCK_IMAGE: i32 = 2;
