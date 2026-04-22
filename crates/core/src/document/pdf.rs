//! PDF Document Handling
//!
//! This module provides PDF document loading, rendering, and manipulation via PDFPurr (pure Rust).
//!
//! ## Architecture
//!
//! - **PdfOpener**: Creates and manages PDF document instances
//! - **PdfDocument**: Represents an opened PDF with page access
//! - **PdfPage**: Provides rendering and text extraction for individual pages
//!
//! ## Features
//!
//! - Page rendering to pixmaps
//! - Text extraction and search
//! - Table of contents extraction
//! - Auto-margin detection for scanned documents
//! - Link and annotation handling
//! - Encryption support (RC4, AES-128/256)
//! - PDF manipulation via lopdf

use super::pdfpurr::{
    Document as PdfPurrDocument, MuPdfContext, Page, Link, Outline, FzRect, FzQuad, FzPoint,
    rect_from_quad, union_rect, PixmapFormat, FZ_PAGE_BLOCK_IMAGE,
};

use super::{chapter, chapter_relative};
use super::{BoundedText, Document, Location, TocEntry};
use crate::framebuffer::Pixmap;
use crate::geom::{Boundary, CycleDir};
use std::path::Path;

const USER_STYLESHEET: &str = "css/html-user.css";

fn auto_detect_margins(pixmap: &Pixmap, threshold: u8) -> (f32, f32, f32, f32) {
    let width = pixmap.width as usize;
    let height = pixmap.height as usize;
    let samples = pixmap.samples;
    let data = &pixmap.data;

    let is_blank = |x: usize, y: usize| -> bool {
        let addr = samples * (y * width + x);
        if samples == 1 {
            data[addr] > threshold
        } else {
            data[addr] > threshold && data[addr + 1] > threshold && data[addr + 2] > threshold
        }
    };

    let mut top = 0;
    'top_loop: for y in 0..height {
        for x in 0..width {
            if !is_blank(x, y) {
                top = y;
                break 'top_loop;
            }
        }
    }

    let mut bottom = height;
    'bottom_loop: for y in (0..height).rev() {
        for x in 0..width {
            if !is_blank(x, y) {
                bottom = y + 1;
                break 'bottom_loop;
            }
        }
    }

    let mut left = 0;
    'left_loop: for x in 0..width {
        for y in top..bottom {
            if !is_blank(x, y) {
                left = x;
                break 'left_loop;
            }
        }
    }

    let mut right = width;
    'right_loop: for x in (0..width).rev() {
        for y in top..bottom {
            if !is_blank(x, y) {
                right = x + 1;
                break 'right_loop;
            }
        }
    }

    let content_left = left as f32 / width as f32;
    let content_right = right as f32 / width as f32;
    let content_top = top as f32 / height as f32;
    let content_bottom = bottom as f32 / height as f32;

    let margin_left = content_left;
    let margin_right = 1.0 - content_right;
    let margin_top = content_top;
    let margin_bottom = 1.0 - content_bottom;

    (margin_left, margin_top, margin_right, margin_bottom)
}


/// PDF document opener.
pub struct PdfOpener {
    ctx: MuPdfContext,
}

/// PDF document instance with page access.
pub struct PdfDocument {
    doc: PdfPurrDocument,
}

/// PDF page for rendering and text extraction.
pub struct PdfPage<'a> {
    page: Page<'a>,
    _doc: &'a PdfDocument,
    page_num: usize,
}

impl PdfOpener {
    /// Creates a new PDF opener.
    pub fn new() -> Option<PdfOpener> {
        Some(PdfOpener { ctx: MuPdfContext::new().ok()? })
    }

    /// Opens a PDF file from the given path.
    ///
    /// # Arguments
    /// * `path` - Path to the PDF file
    ///
    /// # Returns
    /// None if the file cannot be opened or is not a valid PDF.
    pub fn open<P: AsRef<Path>>(&self, path: P) -> Option<PdfDocument> {
        PdfPurrDocument::open(path)
            .ok()
            .map(|doc| PdfDocument { doc })
    }

    /// Opens a PDF from memory buffer.
    ///
    /// # Arguments
    /// * `magic` - MIME type or file magic bytes (e.g., "application/pdf")
    /// * `buf` - PDF file content as bytes
    pub fn open_memory(&self, _magic: &str, buf: &[u8]) -> Option<PdfDocument> {
        PdfPurrDocument::from_bytes(buf)
            .ok()
            .map(|doc| PdfDocument { doc })
    }

    /// Loads user stylesheet from css/html-user.css if present.
    pub fn load_user_stylesheet(&mut self) {
        // PDFPurr doesn't need user CSS
        let _ = std::fs::read_to_string(USER_STYLESHEET).map_err(|e| {
            if e.kind() != std::io::ErrorKind::NotFound {
                crate::log_error!("{:#}", e)
            }
        });
    }
}

unsafe impl Send for PdfDocument {}
unsafe impl Sync for PdfDocument {}

impl PdfDocument {
    /// Loads a page by index (0-based).
    ///
    /// # Arguments
    /// * `index` - Page index
    ///
    /// # Returns
    /// None if the page doesn't exist or cannot be loaded.
    pub fn page(&self, index: usize) -> Option<PdfPage<'_>> {
        self.doc
            .load_page(index as i32)
            .ok()
            .map(|page| PdfPage { page, _doc: self, page_num: index })
    }

    fn walk_toc(outline: &Outline, index: &mut usize) -> Vec<TocEntry> {
        let mut vec = Vec::new();
        let mut current: Option<Outline> = Some(outline.clone_outline());

        while let Some(entry) = current {
            let page_loc = entry.page();
            let location = if page_loc.chapter >= 0 && page_loc.page >= 0 {
                Location::Exact((page_loc.chapter * 1000 + page_loc.page) as usize)
            } else if let Some(uri) = entry.uri() {
                Location::Uri(uri)
            } else {
                Location::Exact(0)
            };

            let title = entry.title();
            let current_index = *index;
            *index += 1;

            let children = entry
                .down()
                .map(|down| Self::walk_toc(&down, index))
                .unwrap_or_default();

            let page_num = if page_loc.chapter >= 0 && page_loc.page >= 0 {
                (page_loc.chapter * 1000 + page_loc.page) as usize
            } else {
                0
            };
            vec.push(TocEntry {
                title: title.to_string(),
                location,
                index: current_index,
                children,
                page: Some(page_num),
                level: 0,
            });

            current = entry.next();
        }
        vec
    }

    pub fn is_protected(&self) -> bool {
        self.doc.needs_password()
    }
}

impl Document for PdfDocument {
    fn dims(&self, index: usize) -> Option<(f32, f32)> {
        self.page(index).map(|page| page.dims())
    }

    fn pages_count(&self) -> usize {
        self.doc.page_count() as usize
    }

    fn resolve_location(&mut self, loc: Location) -> Option<usize> {
        if self.pages_count() == 0 {
            return None;
        }

        match loc {
            Location::Exact(index) => {
                if index >= self.pages_count() {
                    None
                } else {
                    Some(index)
                }
            }
            Location::Previous(index) => {
                if index > 0 {
                    Some(index - 1)
                } else {
                    None
                }
            }
            Location::Next(index) => {
                if index < self.pages_count() - 1 {
                    Some(index + 1)
                } else {
                    None
                }
            }
            Location::LocalUri(_index, _uri) => None,
            _ => None,
        }
    }

    fn pixmap(&mut self, loc: Location, scale: f32, samples: usize) -> Option<(Pixmap, usize)> {
        let index = self.resolve_location(loc)?;
        self.page(index)
            .and_then(|page| page.pixmap(scale, samples))
            .map(|pixmap| (pixmap, index))
    }

    fn toc(&mut self) -> Option<Vec<TocEntry>> {
        self.doc.load_outline().map(|outline| {
            let mut index = 0;
            PdfDocument::walk_toc(&outline, &mut index)
        })
    }

    fn chapter<'a>(&mut self, offset: usize, toc: &'a [TocEntry]) -> Option<(&'a TocEntry, f32)> {
        chapter(offset, self.pages_count(), toc)
    }

    fn chapter_relative<'a>(
        &mut self,
        offset: usize,
        dir: CycleDir,
        toc: &'a [TocEntry],
    ) -> Option<&'a TocEntry> {
        chapter_relative(offset, dir, toc)
    }

    fn metadata(&self, key: &str) -> Option<String> {
        self.doc.lookup_metadata(key)
    }

    fn words(&mut self, loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        let index = self.resolve_location(loc)?;
        self.page(index)
            .and_then(|page| page.words())
            .map(|words| (words, index))
    }

    fn lines(&mut self, loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        let index = self.resolve_location(loc)?;
        self.page(index)
            .and_then(|page| page.lines())
            .map(|lines| (lines, index))
    }

    fn images(&mut self, loc: Location) -> Option<(Vec<Boundary>, usize)> {
        let index = self.resolve_location(loc)?;
        self.page(index)
            .and_then(|page| page.images())
            .map(|images| (images, index))
    }

    fn links(&mut self, loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        let index = self.resolve_location(loc)?;
        self.page(index)
            .and_then(|page| page.links())
            .map(|links| (links, index))
    }

    fn title(&self) -> Option<String> {
        self.doc.title()
    }

    fn author(&self) -> Option<String> {
        self.doc.author()
    }

    fn is_reflowable(&self) -> bool {
        self.doc.is_reflowable()
    }

    fn auto_crop_margins(
        &mut self,
        color_samples: usize,
        threshold: u8,
    ) -> Option<(f32, f32, f32, f32)> {
        self.pixmap(Location::Exact(0), 1.0, color_samples)
            .map(|(pixmap, _)| auto_detect_margins(&pixmap, threshold))
    }

    fn layout(&mut self, width: u32, height: u32, _font_size: f32, _dpi: u16) {
        self.doc.layout(width as f32, height as f32);
    }

    fn set_font_family(&mut self, _family_name: &str, _search_path: &str) {
        // PDF documents use embedded fonts.
        // Font family changes are not supported for PDF format.
    }

    fn set_margin_width(&mut self, _width: i32) {
        // PDF documents have fixed page layouts defined by the document.
        // Margin changes are not supported for PDF format.
    }

    fn set_text_align(&mut self, _text_align: crate::metadata::TextAlign) {
        // PDF documents have fixed text positioning defined by the document.
        // Text alignment changes are not supported for PDF format.
    }

    fn set_line_height(&mut self, _line_height: f32) {
        // PDF documents have fixed line spacing defined by the document.
        // Line height changes are not supported for PDF format.
    }

    fn set_hyphen_penalty(&mut self, _hyphen_penalty: i32) {
        // PDF documents have fixed hyphenation defined by the document.
        // Hyphen penalty changes are not supported for PDF format.
    }

    fn set_stretch_tolerance(&mut self, _stretch_tolerance: f32) {
        // PDF documents have fixed text layout defined by the document.
        // Stretch tolerance changes are not supported for PDF format.
    }

    fn set_ignore_document_css(&mut self, _ignore: bool) {
        // PDFPurr doesn't use CSS
        // This is kept for API compatibility
    }
}

impl<'a> PdfPage<'a> {
    pub fn images(&self) -> Option<Vec<Boundary>> {
        let text_page = self.page.to_text_page(None)?;
        let mut images = Vec::with_capacity(16);

        for block in text_page.blocks() {
            if block.kind() == FZ_PAGE_BLOCK_IMAGE {
                let bnd: Boundary = block.bbox().into();
                images.retain(|img: &Boundary| !img.overlaps(&bnd));
                images.push(bnd);
            }
        }
        Some(images)
    }

    pub fn lines(&self) -> Option<Vec<BoundedText>> {
        let text_page = self.page.to_text_page(None)?;
        let mut lines = Vec::with_capacity(64);

        for block in text_page.blocks() {
            for line in block.lines() {
                let rect: Boundary = line.bbox().into();
                lines.push(BoundedText {
                    rect,
                    text: String::new(),
                    location: rect.min.into(),
                });
            }
        }
        Some(lines)
    }

    pub fn words(&self) -> Option<Vec<BoundedText>> {
        let text_page = self.page.to_text_page(None)?;
        let mut words = Vec::with_capacity(128);

        for block in text_page.blocks() {
            for line in block.lines() {
                let mut current_word = String::new();
                let mut word_rect = FzRect::default();

                for text_char in line.chars() {
                    if let Some(c) = std::char::from_u32(text_char.char_code as u32) {
                        if c.is_whitespace() {
                            if !current_word.is_empty() {
                                let bounds: Boundary = word_rect.into();
                                words.push(BoundedText {
                                    text: current_word.clone(),
                                    rect: bounds,
                                    location: bounds.min.into(),
                                });
                                current_word.clear();
                                word_rect = FzRect::default();
                            }
                        } else {
                            let quad = text_char.quad;
                            let pdfpurr_quad = FzQuad {
                                ul: FzPoint { x: quad.ul.x, y: quad.ul.y },
                                ur: FzPoint { x: quad.ur.x, y: quad.ur.y },
                                ll: FzPoint { x: quad.ll.x, y: quad.ll.y },
                                lr: FzPoint { x: quad.lr.x, y: quad.lr.y },
                            };
                            let chr_rect = rect_from_quad(pdfpurr_quad);
                            word_rect = union_rect(word_rect, chr_rect);
                            current_word.push(c);
                        }
                    }
                }

                if !current_word.is_empty() {
                    let bounds: Boundary = word_rect.into();
                    words.push(BoundedText {
                        text: current_word,
                        rect: bounds,
                        location: bounds.min.into(),
                    });
                }
            }
        }
        Some(words)
    }

    pub fn links(&self) -> Option<Vec<BoundedText>> {
        let first_link = self.page.load_links()?;
        let mut result = Vec::new();
        let mut current: Option<Link> = Some(first_link);

        while let Some(link) = current {
            let text = link.uri();
            let rect: Boundary = link.rect().into();
            result.push(BoundedText {
                text,
                rect,
                location: rect.min.into(),
            });
            current = link.next();
        }
        Some(result)
    }

    pub fn pixmap(&self, zoom: f32, color_samples: usize) -> Option<Pixmap> {
        let color_space = if color_samples == 1 {
            PixmapFormat::Grayscale
        } else {
            PixmapFormat::RGB
        };
        
        self.page.render_pixmap(zoom, color_space, 0).ok().and_then(|pdfpurr_pixmap| {
            let width = pdfpurr_pixmap.width();
            let height = pdfpurr_pixmap.height();
            let data = pdfpurr_pixmap.data();
            
            // Convert RGBA to grayscale if needed
            let samples = if color_samples == 1 { 1usize } else { 3usize };
            let mut pixmap_data = vec![0u8; (width * height * samples as u32) as usize];
            
            if color_samples == 1 {
                // Convert RGBA to grayscale using luminance formula
                for i in 0..(width * height) as usize {
                    let rgba_idx = i * 4;
                    let r = data[rgba_idx] as f32;
                    let g = data[rgba_idx + 1] as f32;
                    let b = data[rgba_idx + 2] as f32;
                    // Rec. 709 luma coefficients
                    let gray = (0.2126 * r + 0.7152 * g + 0.0722 * b) as u8;
                    pixmap_data[i] = gray;
                }
            } else {
                // RGB
                for i in 0..(width * height) as usize {
                    let rgba_idx = i * 4;
                    let rgb_idx = i * 3;
                    pixmap_data[rgb_idx] = data[rgba_idx];
                    pixmap_data[rgb_idx + 1] = data[rgba_idx + 1];
                    pixmap_data[rgb_idx + 2] = data[rgba_idx + 2];
                }
            }
            
            Some(Pixmap {
                width: width,
                height: height,
                samples: samples,
                data: pixmap_data,
                update_flag: false,
            })
        })
    }

    pub fn boundary_box(&self) -> Option<Boundary> {
        // PDFPurr doesn't have the same bbox device API
        // Return the full page bounds as a fallback
        let (width, height) = self.dims();
        Some(Boundary {
            min: vec2!(0.0, 0.0),
            max: vec2!(width, height),
        })
    }

    #[inline]
    pub fn dims(&self) -> (f32, f32) {
        self.page.dims()
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.page.width()
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.page.height()
    }
}
