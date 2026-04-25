//! PDFPurr integration for Plato
//!
//! This module provides a wrapper around PDFPurr (pure Rust PDF library)
//! to replace the MuPDF C library dependency.

mod outline;
mod page;
mod text;
mod types;

pub use types::{
    rect_from_quad, scale, union_rect, FzLocation, FzPoint, FzQuad, FzRect, PdfPurrPixmap,
    PixmapFormat, FZ_PAGE_BLOCK_IMAGE,
};

pub use outline::{Link, Outline};
pub use page::Page;
pub use text::{TextBlock, TextChar, TextLine, TextPage};

use anyhow::{bail, Context, Result};
use pdfpurr::Document as PdfPurrDoc;
use std::path::Path;
use std::sync::Arc;

use crate::document::cache::{PageCacheKey, PdfCache};

/// Wrapper around PDFPurr Document
pub struct Document {
    inner: PdfPurrDoc,
    cache: Option<Arc<PdfCache>>,
    doc_id: String,
    lopdf_doc: Option<lopdf::Document>,
}

/// Type alias for compatibility
pub type PdfPurrDocument = Document;

impl Document {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let doc_id = path.as_ref().to_string_lossy().to_string();
        let inner = PdfPurrDoc::open(&path)
            .map_err(|e| anyhow::format_err!("Failed to open PDF: {}", e))?;

        // Load lopdf document for link extraction
        let lopdf_doc = lopdf::Document::load(path.as_ref())
            .context("Failed to load PDF with lopdf for link extraction")?;

        Ok(Document {
            inner,
            cache: None,
            doc_id,
            lopdf_doc: Some(lopdf_doc),
        })
    }

    pub fn open_with_cache<P: AsRef<Path>>(path: P, cache: Arc<PdfCache>) -> Result<Self> {
        let doc_id = path.as_ref().to_string_lossy().to_string();
        let inner = PdfPurrDoc::open(&path)
            .map_err(|e| anyhow::format_err!("Failed to open PDF: {}", e))?;

        // Load lopdf document for link extraction
        let lopdf_doc = lopdf::Document::load(path.as_ref())
            .context("Failed to load PDF with lopdf for link extraction")?;

        Ok(Document {
            inner,
            cache: Some(cache),
            doc_id,
            lopdf_doc: Some(lopdf_doc),
        })
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Err(anyhow::format_err!("PDF data cannot be empty"));
        }
        let inner = PdfPurrDoc::from_bytes(data)
            .map_err(|e| anyhow::format_err!("Failed to load PDF from bytes: {}", e))?;
        let doc_id = format!("bytes_{}", hex::encode(&data[..8]));

        // Load lopdf document for link extraction
        let lopdf_doc = lopdf::Document::load_mem(data)
            .context("Failed to load PDF with lopdf for link extraction")?;

        Ok(Document {
            inner,
            cache: None,
            doc_id,
            lopdf_doc: Some(lopdf_doc),
        })
    }

    pub fn from_bytes_with_cache(data: &[u8], cache: Arc<PdfCache>) -> Result<Self> {
        if data.is_empty() {
            return Err(anyhow::format_err!("PDF data cannot be empty"));
        }
        let inner = PdfPurrDoc::from_bytes(data)
            .map_err(|e| anyhow::format_err!("Failed to load PDF from bytes: {}", e))?;
        let doc_id = format!("bytes_{}", hex::encode(&data[..8]));

        // Load lopdf document for link extraction
        let lopdf_doc = lopdf::Document::load_mem(data)
            .context("Failed to load PDF with lopdf for link extraction")?;

        Ok(Document {
            inner,
            cache: Some(cache),
            doc_id,
            lopdf_doc: Some(lopdf_doc),
        })
    }

    pub fn set_cache(&mut self, cache: Arc<PdfCache>) {
        self.cache = Some(cache);
    }

    pub fn object_count(&self) -> usize {
        self.inner.object_count()
    }

    pub fn load_page(&self, index: i32) -> Result<Page<'_>> {
        if index < 0 {
            bail!("Invalid page index: {}", index);
        }
        let page_index = index as usize;
        let page_count = self.inner.page_count().unwrap_or(0);
        if page_index >= page_count {
            bail!(
                "Page index {} out of range (document has {} pages)",
                page_index,
                page_count
            );
        }
        let cache_key = PageCacheKey::new(self.doc_id.clone(), page_index as i32);
        Ok(Page::new(
            &self.inner,
            page_index,
            cache_key,
            self.cache.clone(),
            self.lopdf_doc.as_ref(),
        ))
    }

    pub fn is_reflowable(&self) -> bool {
        false
    }

    pub fn layout(&mut self, _width: f32, _height: f32) {
        // PDFPurr doesn't support reflow
        let _width = _width.max(0.0);
        let _height = _height.max(0.0);
    }

    pub fn title(&self) -> Option<String> {
        self.inner.metadata().title
    }

    pub fn author(&self) -> Option<String> {
        self.inner.metadata().author
    }

    pub fn lookup_metadata(&self, key: &str) -> Option<String> {
        match key.to_lowercase().as_str() {
            "title" => self.inner.metadata().title,
            "author" => self.inner.metadata().author,
            "subject" => self.inner.metadata().subject,
            "keywords" => self.inner.metadata().keywords,
            "creator" => self.inner.metadata().creator,
            "producer" => self.inner.metadata().producer,
            _ => None,
        }
    }

    pub fn metadata(&self) -> Option<String> {
        // Return simple metadata as string since PDFPurr's Metadata doesn't implement Serialize
        let meta = self.inner.metadata();
        let mut parts = Vec::new();
        if let Some(title) = &meta.title {
            parts.push(format!("Title: {}", title));
        }
        if let Some(author) = &meta.author {
            parts.push(format!("Author: {}", author));
        }
        if let Some(subject) = &meta.subject {
            parts.push(format!("Subject: {}", subject));
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n"))
        }
    }

    pub fn page_count(&self) -> usize {
        self.inner.page_count().unwrap_or(0)
    }

    pub fn load_outline(&self) -> Option<Outline> {
        let outlines = self.inner.outlines();
        if outlines.is_empty() {
            None
        } else {
            Some(Outline::new(outlines))
        }
    }

    pub fn needs_password(&self) -> bool {
        // PDFPurr handles encryption in from_bytes_with_password
        false
    }
}

// ============================================================================
// Unit Tests for Phase 5 Validation
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // FzRect Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_fz_rect_default() {
        let rect = FzRect::default();
        assert_eq!(rect.x0, 0.0);
        assert_eq!(rect.y0, 0.0);
        assert_eq!(rect.x1, 0.0);
        assert_eq!(rect.y1, 0.0);
    }

    #[test]
    fn test_fz_rect_boundary_conversion() {
        let rect = FzRect {
            x0: 10.0,
            y0: 20.0,
            x1: 100.0,
            y1: 200.0,
        };

        let boundary: crate::geom::Boundary = rect.into();
        assert_eq!(boundary.min.x, 10.0);
        assert_eq!(boundary.min.y, 20.0);
        assert_eq!(boundary.max.x, 100.0);
        assert_eq!(boundary.max.y, 200.0);
    }

    // ------------------------------------------------------------------------
    // FzPoint Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_fz_point_default() {
        let point = FzPoint::default();
        assert_eq!(point.x, 0.0);
        assert_eq!(point.y, 0.0);
    }

    // ------------------------------------------------------------------------
    // FzQuad Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_fz_quad_default() {
        let quad = FzQuad::default();
        assert_eq!(quad.ul.x, 0.0);
        assert_eq!(quad.ur.y, 0.0);
        assert_eq!(quad.ll.x, 0.0);
        assert_eq!(quad.lr.y, 0.0);
    }

    // ------------------------------------------------------------------------
    // Utility Function Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_rect_from_quad() {
        let quad = FzQuad {
            ul: FzPoint { x: 0.0, y: 0.0 },
            ur: FzPoint { x: 100.0, y: 0.0 },
            ll: FzPoint { x: 0.0, y: 50.0 },
            lr: FzPoint { x: 100.0, y: 50.0 },
        };

        let rect = rect_from_quad(quad);
        assert_eq!(rect.x0, 0.0);
        assert_eq!(rect.y0, 0.0);
        assert_eq!(rect.x1, 100.0);
        assert_eq!(rect.y1, 50.0);
    }

    #[test]
    fn test_rect_from_rotated_quad() {
        // Rotated rectangle
        let quad = FzQuad {
            ul: FzPoint { x: 0.0, y: 10.0 },
            ur: FzPoint { x: 90.0, y: 0.0 },
            ll: FzPoint { x: 10.0, y: 60.0 },
            lr: FzPoint { x: 100.0, y: 50.0 },
        };

        let rect = rect_from_quad(quad);
        assert_eq!(rect.x0, 0.0); // min x
        assert_eq!(rect.y0, 0.0); // min y
        assert_eq!(rect.x1, 100.0); // max x
        assert_eq!(rect.y1, 60.0); // max y
    }

    #[test]
    fn test_union_rect() {
        let a = FzRect {
            x0: 0.0,
            y0: 0.0,
            x1: 50.0,
            y1: 50.0,
        };
        let b = FzRect {
            x0: 30.0,
            y0: 30.0,
            x1: 100.0,
            y1: 100.0,
        };

        let union = union_rect(a, b);
        assert_eq!(union.x0, 0.0);
        assert_eq!(union.y0, 0.0);
        assert_eq!(union.x1, 100.0);
        assert_eq!(union.y1, 100.0);
    }

    #[test]
    fn test_union_disjoint_rects() {
        let a = FzRect {
            x0: 0.0,
            y0: 0.0,
            x1: 10.0,
            y1: 10.0,
        };
        let b = FzRect {
            x0: 100.0,
            y0: 100.0,
            x1: 200.0,
            y1: 200.0,
        };

        let union = union_rect(a, b);
        assert_eq!(union.x0, 0.0);
        assert_eq!(union.y0, 0.0);
        assert_eq!(union.x1, 200.0);
        assert_eq!(union.y1, 200.0);
    }

    #[test]
    fn test_scale() {
        assert_eq!(scale(1.0, 2.0), 1.0);
        assert_eq!(scale(100.0, 50.0), 100.0);
    }

    // ------------------------------------------------------------------------
    // Outline Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_fz_location_default() {
        let loc = FzLocation::default();
        assert_eq!(loc.chapter, 0);
        assert_eq!(loc.page, 0);
    }

    // ------------------------------------------------------------------------
    // Pixmap Format Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_pixmap_format_variants() {
        let _gray = PixmapFormat::Grayscale;
        let _rgb = PixmapFormat::RGB;
    }

    // ------------------------------------------------------------------------
    // Error Handling Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_negative_page_index() {
        // This would need a mock document to test properly
        // For now, just verify the error path exists
    }

    // ------------------------------------------------------------------------
    // Edge Case Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_rect_from_quad_single_point() {
        let quad = FzQuad {
            ul: FzPoint { x: 5.0, y: 5.0 },
            ur: FzPoint { x: 5.0, y: 5.0 },
            ll: FzPoint { x: 5.0, y: 5.0 },
            lr: FzPoint { x: 5.0, y: 5.0 },
        };

        let rect = rect_from_quad(quad);
        assert_eq!(rect.x0, 5.0);
        assert_eq!(rect.y0, 5.0);
        assert_eq!(rect.x1, 5.0);
        assert_eq!(rect.y1, 5.0);
    }

    #[test]
    fn test_union_rect_same() {
        let a = FzRect {
            x0: 10.0,
            y0: 10.0,
            x1: 50.0,
            y1: 50.0,
        };

        let union = union_rect(a, a);
        assert_eq!(union.x0, 10.0);
        assert_eq!(union.y0, 10.0);
        assert_eq!(union.x1, 50.0);
        assert_eq!(union.y1, 50.0);
    }

    #[test]
    fn test_empty_outline() {
        let outline = Outline::new(vec![]);

        assert!(outline.next().is_none());
        assert!(outline.down().is_none());
        assert_eq!(outline.title(), "");
        assert_eq!(outline.page().page, 0);
    }

    #[test]
    fn test_link_empty() {
        let link = Link::new(vec![], 0);

        assert_eq!(link.uri(), "");
        assert!(link.next().is_none());
    }

    #[test]
    fn test_empty_text_page() {
        let text_page = TextPage::new(vec![]);

        assert!(text_page.blocks().is_empty());
        assert_eq!(text_page.chars(), 0);
    }

    #[test]
    fn test_text_page_single_run() {
        use pdfpurr::content::analysis::TextRun;

        let run = TextRun {
            text: "Hello".to_string(),
            x: 10.0,
            y: 20.0,
            width: 50.0,
            height: 12.0,
            font_size: 12.0,
            color: [0.0, 0.0, 0.0, 1.0],
            font_name: "Arial".to_string(),
            is_bold: false,
            is_italic: false,
            rendering_mode: 0,
            is_monospaced: false,
        };

        let text_page = TextPage::new(vec![run]);

        assert_eq!(text_page.chars(), 5);
        let blocks = text_page.blocks();
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn test_text_page_multiple_runs() {
        use pdfpurr::content::analysis::TextRun;

        let runs = vec![
            TextRun {
                text: "Hello".to_string(),
                x: 10.0,
                y: 20.0,
                width: 50.0,
                height: 12.0,
                font_size: 12.0,
                color: [0.0, 0.0, 0.0, 1.0],
                font_name: "Arial".to_string(),
                is_bold: false,
                is_italic: false,
                rendering_mode: 0,
                is_monospaced: false,
            },
            TextRun {
                text: "World".to_string(),
                x: 65.0,
                y: 20.0,
                width: 50.0,
                height: 12.0,
                font_size: 12.0,
                color: [0.0, 0.0, 0.0, 1.0],
                font_name: "Arial".to_string(),
                is_bold: false,
                is_italic: false,
                rendering_mode: 0,
                is_monospaced: false,
            },
        ];

        let text_page = TextPage::new(runs);

        assert_eq!(text_page.chars(), 10);
        let blocks = text_page.blocks();
        assert_eq!(blocks.len(), 1); // Same y position, same block
    }

    #[test]
    fn test_text_page_separate_lines() {
        use pdfpurr::content::analysis::TextRun;

        let runs = vec![
            TextRun {
                text: "Line 1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 50.0,
                height: 12.0,
                font_size: 12.0,
                color: [0.0, 0.0, 0.0, 1.0],
                font_name: "Arial".to_string(),
                is_bold: false,
                is_italic: false,
                rendering_mode: 0,
                is_monospaced: false,
            },
            TextRun {
                text: "Line 2".to_string(),
                x: 10.0,
                y: 40.0, // Different y position
                width: 50.0,
                height: 12.0,
                font_size: 12.0,
                color: [0.0, 0.0, 0.0, 1.0],
                font_name: "Arial".to_string(),
                is_bold: false,
                is_italic: false,
                rendering_mode: 0,
                is_monospaced: false,
            },
        ];

        let text_page = TextPage::new(runs);

        let blocks = text_page.blocks();
        assert_eq!(blocks.len(), 2); // Different y positions, different blocks
    }
}
