//! Compatibility types for PDFPurr PDF rendering library.
//!
//! These types provide API compatibility during the migration from MuPDF to PDFPurr.
//! They are used for coordinate conversion and PDF page layout calculations.

/// Rectangle type for PDF coordinate representation (compatible with MuPDF fz_rect).
#[derive(Debug, Clone, Copy, Default)]
pub struct FzRect {
    /// The x-coordinate of the bottom-left corner.
    pub x0: f32,
    /// The y-coordinate of the bottom-left corner.
    pub y0: f32,
    /// The x-coordinate of the top-right corner.
    pub x1: f32,
    /// The y-coordinate of the top-right corner.
    pub y1: f32,
}

impl FzRect {
    #[expect(
        clippy::should_implement_trait,
        reason = "Custom default() provides zero-initialized rectangle specific to PDF coordinate system"
    )]
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

/// Point type for PDF coordinate representation (compatible with MuPDF fz_point).
#[derive(Debug, Clone, Copy, Default)]
pub struct FzPoint {
    /// The x-coordinate.
    pub x: f32,
    /// The y-coordinate.
    pub y: f32,
}

/// Quadrilateral type for PDF text selection (compatible with MuPDF fz_quad).
#[derive(Debug, Clone, Copy, Default)]
pub struct FzQuad {
    /// The upper-left corner.
    pub ul: FzPoint,
    /// The upper-right corner.
    pub ur: FzPoint,
    /// The lower-left corner.
    pub ll: FzPoint,
    /// The lower-right corner.
    pub lr: FzPoint,
}

/// Location type for PDF chapter/page navigation (compatible with MuPDF fz_location).
#[derive(Debug, Clone, Default)]
pub struct FzLocation {
    /// The chapter index.
    pub chapter: i32,
    /// The page index.
    pub page: i32,
}

/// Pixel format for rendered output buffers.
#[derive(Debug, Clone, Copy)]
pub enum PixmapFormat {
    /// Grayscale color format.
    Grayscale,
    /// RGB color format.
    RGB,
}

/// Pixmap wrapper for PDFPurr rendering output.
#[derive(Debug, Clone)]
pub struct PdfPurrPixmap {
    inner: tiny_skia::Pixmap,
}

impl PdfPurrPixmap {
    /// Creates a new PdfPurrPixmap from a tiny_skia Pixmap.
    pub fn new(inner: tiny_skia::Pixmap) -> Self {
        Self { inner }
    }

    /// Returns the raw pixel data.
    pub fn data(&self) -> &[u8] {
        self.inner.data()
    }

    /// Returns the width of the pixmap.
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    /// Returns the height of the pixmap.
    pub fn height(&self) -> u32 {
        self.inner.height()
    }
}

/// Image block constant.
pub const FZ_PAGE_BLOCK_IMAGE: i32 = 2;

/// Convert quad to rect.
pub fn rect_from_quad(quad: FzQuad) -> FzRect {
    let min_x = quad.ul.x.min(quad.ll.x).min(quad.ur.x).min(quad.lr.x);
    let max_x = quad.ul.x.max(quad.ll.x).max(quad.ur.x).max(quad.lr.x);
    let min_y = quad.ul.y.min(quad.ll.y).min(quad.ur.y).min(quad.lr.y);
    let max_y = quad.ul.y.max(quad.ll.y).max(quad.ur.y).max(quad.lr.y);

    FzRect {
        x0: min_x,
        y0: min_y,
        x1: max_x,
        y1: max_y,
    }
}

/// Union two rects.
pub fn union_rect(a: FzRect, b: FzRect) -> FzRect {
    FzRect {
        x0: a.x0.min(b.x0),
        y0: a.y0.min(b.y0),
        x1: a.x1.max(b.x1),
        y1: a.y1.max(b.y1),
    }
}

/// Scale factor calculation for coordinate transformations.
pub fn scale(x: f32, _y: f32) -> f32 {
    x
}
