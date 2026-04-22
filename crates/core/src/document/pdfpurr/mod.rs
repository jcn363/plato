//! PDFPurr integration for Plato
//!
//! This module provides a wrapper around PDFPurr (pure Rust PDF library)
//! to replace the MuPDF C library dependency.

use std::path::Path;
use anyhow::{Result, bail};
use pdfpurr::Document as PdfPurrDoc;
use pdfpurr::rendering::{Renderer, RenderOptions};
use pdfpurr::content::analysis::TextRun;

/// Wrapper around PDFPurr Document
pub struct Document {
    inner: PdfPurrDoc,
}

/// Type alias for compatibility
pub type PdfPurrDocument = Document;

impl Document {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let inner = PdfPurrDoc::open(path)
            .map_err(|e| anyhow::format_err!("Failed to open PDF: {}", e))?;
        Ok(Document { inner })
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let inner = PdfPurrDoc::from_bytes(data)
            .map_err(|e| anyhow::format_err!("Failed to load PDF from bytes: {}", e))?;
        Ok(Document { inner })
    }

    pub fn object_count(&self) -> usize {
        self.inner.object_count()
    }

    pub fn load_page(&self, index: i32) -> Result<Page> {
        if index < 0 {
            bail!("Invalid page index: {}", index);
        }
        let page_index = index as usize;
        let page_count = self.inner.page_count().unwrap_or(0);
        if page_index >= page_count {
            bail!("Page index {} out of range (document has {} pages)", page_index, page_count);
        }
        Ok(Page {
            doc: &self.inner,
            index: page_index,
        })
    }

    pub fn is_reflowable(&self) -> bool {
        false
    }

    pub fn layout(&mut self, _width: f32, _height: f32) {
        // PDFPurr doesn't support reflow
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
            Some(Outline { outlines })
        }
    }

    pub fn needs_password(&self) -> bool {
        // PDFPurr handles encryption in from_bytes_with_password
        false
    }
}

/// Wrapper around PDFPurr page
pub struct Page<'a> {
    doc: &'a PdfPurrDoc,
    index: usize,
}

impl<'a> Page<'a> {
    pub fn to_text_page(&self, _options: Option<&()>) -> Option<TextPage> {
        // Extract text runs from PDFPurr
        self.doc.extract_text_runs(self.index).ok().map(|runs| TextPage { runs })
    }

    pub fn load_links(&self) -> Option<Link> {
        // TODO: Need to get page dictionary first, then call page_annotations
        // For now, return empty link list
        Some(Link { annots: Vec::new(), index: 0 })
    }

    pub fn render_pixmap(&self, _matrix: f32, _color_space: PixmapFormat, _flags: i32) -> Result<PdfPurrPixmap> {
        let options = RenderOptions {
            dpi: 72.0 * _matrix as f64,
            background: [255, 255, 255, 255],
        };
        let renderer = Renderer::new(self.doc, options);
        let pixmap = renderer.render_page(self.index)
            .map_err(|e| anyhow::format_err!("Failed to render page: {}", e))?;
        Ok(PdfPurrPixmap { inner: pixmap })
    }

    pub fn dims(&self) -> (f32, f32) {
        // Get page dimensions from PDFPurr
        // PDFPurr stores page dimensions in the page dictionary
        // For now, use default dimensions - this should be improved
        // by accessing the PDF's MediaBox directly
        (600.0, 800.0)
    }

    pub fn width(&self) -> f32 {
        self.dims().0
    }

    pub fn height(&self) -> f32 {
        self.dims().1
    }

    pub fn media_box(&self) -> FzRect {
        let (width, height) = self.dims();
        FzRect {
            x0: 0.0,
            y0: 0.0,
            x1: width,
            y1: height,
        }
    }

    pub fn search(&self, needle: &str) -> Option<Vec<FzQuad>> {
        // Basic search implementation using PDFPurr text extraction
        let text_runs = self.doc.extract_text_runs(self.index).ok()?;
        let text: String = text_runs.iter().map(|r| r.text.as_str()).collect();
        
        if text.contains(needle) {
            // Return page-level quad if text is found
            // Full implementation would need character-level position tracking
            Some(vec![FzQuad {
                ul: FzPoint { x: 0.0, y: 0.0 },
                ur: FzPoint { x: 600.0, y: 0.0 },
                ll: FzPoint { x: 0.0, y: 800.0 },
                lr: FzPoint { x: 600.0, y: 800.0 },
            }])
        } else {
            Some(Vec::new())
        }
    }

    pub fn images(&self) -> Option<Vec<FzRect>> {
        // PDFPurr doesn't have a direct image extraction API in version 0.4.0
        // This would require accessing the PDF's XObject dictionary directly
        // This is a Phase 4 feature - for now return empty list
        Some(Vec::new())
    }

    pub fn char_count(&self) -> usize {
        self.to_text_page(None).map(|tp| tp.chars()).unwrap_or(0)
    }
}

/// Text page wrapper for PDFPurr text runs
pub struct TextPage {
    runs: Vec<TextRun>,
}

impl TextPage {
    pub fn blocks(&self) -> Vec<TextBlock> {
        // Convert TextRuns to TextBlocks for compatibility
        if self.runs.is_empty() {
            return Vec::new();
        }
        
        let mut blocks = Vec::new();
        let mut current_block_runs = Vec::new();
        let mut last_y = self.runs[0].y;
        
        for run in &self.runs {
            // Group runs by vertical position (lines)
            if (run.y - last_y).abs() > run.height {
                if !current_block_runs.is_empty() {
                    let bbox = self.bbox_from_runs(&current_block_runs);
                    blocks.push(TextBlock {
                        runs: current_block_runs.clone(),
                        kind: 0, // Text block
                        bbox,
                    });
                    current_block_runs.clear();
                }
            }
            current_block_runs.push(run.clone());
            last_y = run.y;
        }
        
        if !current_block_runs.is_empty() {
            let bbox = self.bbox_from_runs(&current_block_runs);
            blocks.push(TextBlock {
                runs: current_block_runs,
                kind: 0,
                bbox,
            });
        }
        
        blocks
    }
    
    fn bbox_from_runs(&self, runs: &[TextRun]) -> FzRect {
        if runs.is_empty() {
            return FzRect::default();
        }
        
        let min_x = runs.iter().map(|r| r.x).fold(f64::INFINITY, f64::min);
        let max_x = runs.iter().map(|r| r.x + r.width).fold(f64::NEG_INFINITY, f64::max);
        let min_y = runs.iter().map(|r| r.y).fold(f64::INFINITY, f64::min);
        let max_y = runs.iter().map(|r| r.y + r.height).fold(f64::NEG_INFINITY, f64::max);
        
        FzRect {
            x0: min_x as f32,
            y0: min_y as f32,
            x1: max_x as f32,
            y1: max_y as f32,
        }
    }
    
    pub fn chars(&self) -> usize {
        self.runs.iter().map(|r| r.text.chars().count()).sum()
    }
}

/// Text block wrapper
pub struct TextBlock {
    runs: Vec<TextRun>,
    kind: i32,
    bbox: FzRect,
}

impl TextBlock {
    pub fn kind(&self) -> i32 {
        self.kind
    }

    pub fn bbox(&self) -> FzRect {
        self.bbox
    }

    pub fn lines(&self) -> Vec<TextLine> {
        // Convert runs to lines
        self.runs.chunks(10).map(|chunk| TextLine {
            runs: chunk.to_vec(),
            bbox: self.bbox_from_runs(chunk),
        }).collect()
    }
    
    pub fn chars(&self) -> Vec<TextChar> {
        self.runs.iter().flat_map(|run| {
            run.text.chars().enumerate().map(move |(i, c)| {
                let char_x = run.x + (i as f64 * run.width / run.text.len() as f64);
                TextChar {
                    char_code: c as u32 as i32,
                    quad: FzQuad {
                        ul: FzPoint { x: char_x as f32, y: run.y as f32 },
                        ur: FzPoint { x: (char_x + run.width / run.text.len() as f64) as f32, y: run.y as f32 },
                        ll: FzPoint { x: char_x as f32, y: (run.y + run.height) as f32 },
                        lr: FzPoint { x: (char_x + run.width / run.text.len() as f64) as f32, y: (run.y + run.height) as f32 },
                    },
                    origin: 0,
                }
            })
        }).collect()
    }
    
    fn bbox_from_runs(&self, runs: &[TextRun]) -> FzRect {
        if runs.is_empty() {
            return FzRect::default();
        }
        
        let min_x = runs.iter().map(|r| r.x).fold(f64::INFINITY, f64::min);
        let max_x = runs.iter().map(|r| r.x + r.width).fold(f64::NEG_INFINITY, f64::max);
        let min_y = runs.iter().map(|r| r.y).fold(f64::INFINITY, f64::min);
        let max_y = runs.iter().map(|r| r.y + r.height).fold(f64::NEG_INFINITY, f64::max);
        
        FzRect {
            x0: min_x as f32,
            y0: min_y as f32,
            x1: max_x as f32,
            y1: max_y as f32,
        }
    }
}

/// Text line wrapper
pub struct TextLine {
    runs: Vec<TextRun>,
    bbox: FzRect,
}

impl TextLine {
    pub fn bbox(&self) -> FzRect {
        self.bbox
    }

    pub fn chars(&self) -> Vec<TextChar> {
        self.runs.iter().flat_map(|run| {
            run.text.chars().enumerate().map(move |(i, c)| {
                let char_x = run.x + (i as f64 * run.width / run.text.len() as f64);
                TextChar {
                    char_code: c as u32 as i32,
                    quad: FzQuad {
                        ul: FzPoint { x: char_x as f32, y: run.y as f32 },
                        ur: FzPoint { x: (char_x + run.width / run.text.len() as f64) as f32, y: run.y as f32 },
                        ll: FzPoint { x: char_x as f32, y: (run.y + run.height) as f32 },
                        lr: FzPoint { x: (char_x + run.width / run.text.len() as f64) as f32, y: (run.y + run.height) as f32 },
                    },
                    origin: 0,
                }
            })
        }).collect()
    }
}

/// Text character wrapper
pub struct TextChar {
    pub char_code: i32,
    pub quad: FzQuad,
    pub origin: i32,
}

/// Link wrapper for PDFPurr annotations
pub struct Link {
    annots: Vec<pdfpurr::structure::Annotation>,
    index: usize,
}

impl Link {
    pub fn uri(&self) -> String {
        if self.index < self.annots.len() {
            if let Some(uri) = &self.annots[self.index].uri {
                return uri.clone();
            }
        }
        String::new()
    }

    pub fn rect(&self) -> FzRect {
        if self.index < self.annots.len() {
            let rect = self.annots[self.index].rect;
            FzRect {
                x0: rect[0] as f32,
                y0: rect[1] as f32,
                x1: rect[2] as f32,
                y1: rect[3] as f32,
            }
        } else {
            FzRect::default()
        }
    }

    pub fn next(&self) -> Option<Link> {
        if self.index + 1 < self.annots.len() {
            Some(Link {
                annots: self.annots.clone(),
                index: self.index + 1,
            })
        } else {
            None
        }
    }
}

/// Outline wrapper for PDFPurr outlines
pub struct Outline {
    outlines: Vec<pdfpurr::structure::Outline>,
}

impl Outline {
    pub fn clone_outline(&self) -> Outline {
        Outline {
            outlines: self.outlines.clone(),
        }
    }

    pub fn page(&self) -> FzLocation {
        if let Some(first) = self.outlines.first() {
            FzLocation {
                chapter: 0,
                page: first.page.unwrap_or(0) as i32,
            }
        } else {
            FzLocation { chapter: 0, page: 0 }
        }
    }

    pub fn uri(&self) -> Option<String> {
        self.outlines.first().and_then(|o| o.uri.clone())
    }

    pub fn next(&self) -> Option<Outline> {
        if self.outlines.len() > 1 {
            Some(Outline {
                outlines: self.outlines[1..].to_vec(),
            })
        } else {
            None
        }
    }

    pub fn title(&self) -> String {
        self.outlines.first().map(|o| o.title.clone()).unwrap_or_default()
    }

    pub fn down(&self) -> Option<Outline> {
        self.outlines.first().and_then(|o| {
            if !o.children.is_empty() {
                Some(Outline {
                    outlines: o.children.clone(),
                })
            } else {
                None
            }
        })
    }
}

/// Stub FzRect type
#[derive(Debug, Clone, Copy, Default)]
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
#[derive(Debug, Clone, Copy, Default)]
pub struct FzPoint {
    pub x: f32,
    pub y: f32,
}

/// Stub FzQuad type
#[derive(Debug, Clone, Copy, Default)]
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

/// Pixmap wrapper for PDFPurr rendering output
pub struct PdfPurrPixmap {
    inner: tiny_skia::Pixmap,
}

impl PdfPurrPixmap {
    pub fn data(&self) -> &[u8] {
        self.inner.data()
    }
    
    pub fn width(&self) -> u32 {
        self.inner.width()
    }
    
    pub fn height(&self) -> u32 {
        self.inner.height()
    }
}

/// Convert quad to rect
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

/// Union two rects
pub fn union_rect(a: FzRect, b: FzRect) -> FzRect {
    FzRect {
        x0: a.x0.min(b.x0),
        y0: a.y0.min(b.y0),
        x1: a.x1.max(b.x1),
        y1: a.y1.max(b.y1),
    }
}

/// Scale function (simplified)
pub fn scale(x: f32, _y: f32) -> f32 {
    x
}

/// Image block constant
pub const FZ_PAGE_BLOCK_IMAGE: i32 = 2;
