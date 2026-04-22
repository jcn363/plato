// Temporarily commented out due to pdfpurr API changes
// mod annotation;
// mod document;
// mod image;
// mod link;
mod outline;
// mod page;
// mod pixmap;
// mod text;

// pub use annotation::Annotation;
// pub use document::Document;
// pub use image::Image;
// pub use link::Link;
pub use outline::Outline;
// pub use page::Page;
// pub use pixmap::{Pixmap, PixmapFormat};
// pub use text::{FzPoint, FzQuad, TextBlock, TextBlockIter, TextChar, TextCharIter, TextLine, TextLineIter, TextPage, FZ_PAGE_BLOCK_IMAGE, FZ_PAGE_BLOCK_TEXT};

// Re-export common types for compatibility
pub use crate::geom::{Boundary, Vec2};

// PDFPurr doesn't have a separate context like MuPDF
// Document management is handled directly through the Document type
pub type PdfContext = ();

// Stub definitions for missing pdfpurr types
pub const FZ_PAGE_BLOCK_IMAGE: i32 = 2;

#[derive(Debug, Clone, Default)]
pub struct FzQuad {
    pub ul: FzPoint,
    pub ur: FzPoint,
    pub ll: FzPoint,
    pub lr: FzPoint,
}

#[derive(Debug, Clone, Default)]
pub struct FzPoint {
    pub x: f32,
    pub y: f32,
}

/// Rectangle type for PDF operations
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

/// Convert a quad to a rectangle
pub fn rect_from_quad(quad: FzQuad) -> FzRect {
    FzRect {
        x0: quad.ul.x.min(quad.ll.x).min(quad.ur.x).min(quad.lr.x),
        y0: quad.ul.y.min(quad.ll.y).min(quad.ur.y).min(quad.lr.y),
        x1: quad.ul.x.max(quad.ll.x).max(quad.ur.x).max(quad.lr.x),
        y1: quad.ul.y.max(quad.ll.y).max(quad.ur.y).max(quad.lr.y),
    }
}

/// Union two rectangles
pub fn union_rect(a: FzRect, b: FzRect) -> FzRect {
    FzRect {
        x0: a.x0.min(b.x0),
        y0: a.y0.min(b.y0),
        x1: a.x1.max(b.x1),
        y1: a.y1.max(b.y1),
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
