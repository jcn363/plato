use super::mupdf;

use super::{chapter, chapter_relative};
use super::{BoundedText, Document, Location, TocEntry};
use crate::framebuffer::Pixmap;
use crate::geom::{Boundary, CycleDir};
use std::char;
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

impl From<mupdf::FzRect> for Boundary {
    fn from(rect: mupdf::FzRect) -> Boundary {
        Boundary {
            min: vec2!(rect.x0, rect.y0),
            max: vec2!(rect.x1, rect.y1),
        }
    }
}

pub struct PdfOpener {
    ctx: mupdf::MuPdfContext,
}

pub struct PdfDocument {
    ctx: mupdf::MuPdfContext,
    doc: mupdf::Document,
}

#[allow(dead_code)]
pub struct PdfPage<'a> {
    page: mupdf::Page,
    index: usize,
    _doc: &'a PdfDocument,
}

impl PdfOpener {
    pub fn new() -> Option<PdfOpener> {
        mupdf::MuPdfContext::new().ok().map(|ctx| PdfOpener { ctx })
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> Option<PdfDocument> {
        self.ctx.open_document(path).map(|doc| PdfDocument {
            ctx: self.ctx.clone(),
            doc,
        })
    }

    pub fn open_memory(&self, magic: &str, buf: &[u8]) -> Option<PdfDocument> {
        self.ctx
            .open_document_memory(magic, buf)
            .map(|doc| PdfDocument {
                ctx: self.ctx.clone(),
                doc,
            })
    }

    pub fn load_user_stylesheet(&mut self) {
        if let Ok(content) = std::fs::read_to_string(USER_STYLESHEET).map_err(|e| {
            if e.kind() != std::io::ErrorKind::NotFound {
                crate::log_error!("{:#}", e)
            }
        }) {
            self.ctx.set_user_css(&content);
        }
    }
}

unsafe impl Send for PdfDocument {}
unsafe impl Sync for PdfDocument {}

impl PdfDocument {
    pub fn page(&self, index: usize) -> Option<PdfPage<'_>> {
        self.doc.load_page(index as i32).ok().map(|page| PdfPage {
            page,
            index,
            _doc: self,
        })
    }

    fn walk_toc(outline: &mupdf::Outline, index: &mut usize) -> Vec<TocEntry> {
        let mut vec = Vec::new();
        let mut current: Option<mupdf::Outline> = Some(outline.clone_outline());

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

            vec.push(TocEntry {
                title,
                location,
                index: current_index,
                children,
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

    fn set_ignore_document_css(&mut self, ignore: bool) {
        self.ctx.set_use_document_css(!ignore);
    }
}

impl<'a> PdfPage<'a> {
    pub fn images(&self) -> Option<Vec<Boundary>> {
        let opts = mupdf::FzTextOptions {
            flags: mupdf::FZ_TEXT_PRESERVE_IMAGES,
            scale: 1.0,
            clip: mupdf::FzRect::default(),
        };
        let text_page = self.page.to_text_page(Some(&opts))?;
        let mut images = Vec::new();

        for block in text_page.blocks() {
            if block.kind() == mupdf::FZ_PAGE_BLOCK_IMAGE {
                let bnd: Boundary = block.bbox().into();
                images.retain(|img: &Boundary| !img.overlaps(&bnd));
                images.push(bnd);
            }
        }
        Some(images)
    }

    pub fn lines(&self) -> Option<Vec<BoundedText>> {
        let text_page = self.page.to_text_page(None)?;
        let mut lines = Vec::new();

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
        let mut words = Vec::new();

        for block in text_page.blocks() {
            for line in block.lines() {
                let mut current_word = String::new();
                let mut word_rect = mupdf::FzRect::default();

                for text_char in line.chars() {
                    if let Some(c) = char::from_u32(text_char.char_code() as u32) {
                        if c.is_whitespace() {
                            if !current_word.is_empty() {
                                let bounds: Boundary = word_rect.into();
                                words.push(BoundedText {
                                    text: current_word.clone(),
                                    rect: bounds,
                                    location: bounds.min.into(),
                                });
                                current_word.clear();
                                word_rect = mupdf::FzRect::default();
                            }
                        } else {
                            let quad = text_char.quad();
                            let ctx = self._doc.ctx.as_ptr();
                            let chr_rect = mupdf::rect_from_quad(ctx, quad);
                            word_rect = mupdf::union_rect(ctx, word_rect, chr_rect);
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
        let mut current: Option<mupdf::Link> = Some(first_link);

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

    pub fn pixmap(&self, scale: f32, color_samples: usize) -> Option<Pixmap> {
        let matrix = mupdf::scale(scale, scale);
        let color_space = if color_samples == 1 {
            self._doc.ctx.device_gray()
        } else {
            self._doc.ctx.device_rgb()
        };
        self.page.render_pixmap(matrix, color_space, 0).ok()
    }

    pub fn boundary_box(&self) -> Option<Boundary> {
        let ctx = self._doc.ctx.as_ptr();
        let rect = mupdf::FzRect::default();
        let dev = mupdf::new_bbox_device(ctx, rect)?;
        self.page.run(dev, mupdf::IDENTITY);
        mupdf::close_device(ctx, dev);
        mupdf::drop_device(ctx, dev);
        Some(rect.into())
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
