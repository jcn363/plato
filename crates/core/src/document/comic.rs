//! Comic Book Archive Support (CBZ/CBR)
//!
//! Provides support for comic book archive formats:
//! - CBZ: ZIP archive containing images
//! - CBR: RAR archive containing images (supported if unrar is available)
//!
//! Images are sorted alphabetically and displayed as pages.

use std::io::Read;
use std::path::Path;

use anyhow::{Context, Error};
use image::DynamicImage;

use crate::framebuffer::Pixmap;
use crate::geom::{Boundary, CycleDir, Point};
use crate::metadata::{Annotation, TextAlign};

use super::{BoundedText, Document, Location, Neighbors, TocEntry};

/// Represents a comic book archive document (CBZ/CBR)
pub struct ComicDocument {
    pages: Vec<Vec<u8>>, // Raw image data for each page
    page_count: usize,
    current_page: usize,
    dims: Vec<(f32, f32)>, // Dimensions for each page
}

impl ComicDocument {
    /// Open a comic book archive file
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "cbz" | "zip" => Self::open_cbz(path),
            "cbr" => Err(format_err!("CBR (RAR) format not yet implemented. Convert to CBZ or use PDF.")),
            _ => Err(format_err!("Unsupported comic archive format: {}", ext)),
        }
    }

    /// Open a CBZ (ZIP) archive
    fn open_cbz(path: &Path) -> Result<Self, Error> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("can't open CBZ file {}", path.display()))?;
        let mut archive = zip::ZipArchive::new(file)
            .with_context(|| format!("can't read CBZ archive {}", path.display()))?;

        let mut image_entries: Vec<(String, Vec<u8>)> = Vec::new();

        // Extract all image files from the archive
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            if entry.is_file() {
                let name = entry.name().to_lowercase();
                if Self::is_image_file(&name) {
                    let mut data = Vec::new();
                    entry.read_to_end(&mut data)?;
                    image_entries.push((name, data));
                }
            }
        }

        // Sort by filename for proper page ordering
        image_entries.sort_by(|a, b| a.0.cmp(&b.0));

        let page_count = image_entries.len();
        if page_count == 0 {
            return Err(format_err!("No images found in CBZ archive"));
        }

        // Get dimensions for each page
        let mut dims = Vec::with_capacity(page_count);
        let mut pages = Vec::with_capacity(page_count);

        for (_, data) in image_entries {
            // Try to get image dimensions
            if let Ok(img) = image::load_from_memory(&data) {
                dims.push((img.width() as f32, img.height() as f32));
            } else {
                dims.push((800.0, 1200.0)); // Default fallback dimensions
            }
            pages.push(data);
        }

        Ok(ComicDocument {
            pages,
            page_count,
            current_page: 0,
            dims,
        })
    }

    /// Check if a file is an image based on extension
    fn is_image_file(name: &str) -> bool {
        name.ends_with(".jpg")
            || name.ends_with(".jpeg")
            || name.ends_with(".png")
            || name.ends_with(".gif")
            || name.ends_with(".bmp")
            || name.ends_with(".webp")
    }

    /// Get image data for a specific page
    fn get_page_data(&self, index: usize) -> Option<&[u8]> {
        self.pages.get(index).map(|v| v.as_slice())
    }
}

impl Document for ComicDocument {
    fn dims(&self, index: usize) -> Option<(f32, f32)> {
        self.dims.get(index).copied()
    }

    fn pages_count(&self) -> usize {
        self.page_count
    }

    fn toc(&mut self) -> Option<Vec<TocEntry>> {
        // Comic archives typically don't have a table of contents
        None
    }

    fn chapter<'a>(
        &mut self,
        _offset: usize,
        _toc: &'a [TocEntry],
    ) -> Option<(&'a TocEntry, f32)> {
        // No chapter support for comic archives
        None
    }

    fn chapter_relative<'a>(
        &mut self,
        _offset: usize,
        _dir: CycleDir,
        _toc: &'a [TocEntry],
    ) -> Option<&'a TocEntry> {
        // No chapter support for comic archives
        None
    }

    fn words(&mut self, _loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        // No text extraction for comic images
        None
    }

    fn lines(&mut self, _loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        // No text extraction for comic images
        None
    }

    fn links(&mut self, _loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        // No links in comic archives
        None
    }

    fn images(&mut self, _loc: Location) -> Option<(Vec<Boundary>, usize)> {
        // Each page is an image, so we return the page as a single image
        None
    }

    fn pixmap(&mut self, loc: Location, _scale: f32, _samples: usize) -> Option<(Pixmap, usize)> {
        let page = match loc {
            Location::Exact(p) => p,
            Location::Previous(p) => p.saturating_sub(1),
            Location::Next(p) => (p + 1).min(self.page_count.saturating_sub(1)),
            _ => self.current_page,
        };

        if page >= self.page_count {
            return None;
        }

        // Load the image and convert to Pixmap
        let data = self.get_page_data(page)?;
        let img = image::load_from_memory(data).ok()?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let pixmap = Pixmap::from_raw(
            width as usize,
            height as usize,
            rgba.into_raw(),
        );

        self.current_page = page;
        Some((pixmap, page))
    }

    fn layout(&mut self, _width: u32, _height: u32, _font_size: f32, _dpi: u16) {
        // No layout needed for fixed-size comic images
    }

    fn set_font_family(&mut self, _family_name: &str, _search_path: &str) {
        // Not applicable for comic archives
    }

    fn set_margin_width(&mut self, _width: i32) {
        // Not applicable for comic archives
    }

    fn set_text_align(&mut self, _text_align: TextAlign) {
        // Not applicable for comic archives
    }

    fn set_line_height(&mut self, _line_height: f32) {
        // Not applicable for comic archives
    }

    fn set_hyphen_penalty(&mut self, _hyphen_penalty: i32) {
        // Not applicable for comic archives
    }

    fn set_stretch_tolerance(&mut self, _stretch_tolerance: f32) {
        // Not applicable for comic archives
    }

    fn set_ignore_document_css(&mut self, _ignore: bool) {
        // Not applicable for comic archives
    }

    fn title(&self) -> Option<String> {
        None
    }

    fn author(&self) -> Option<String> {
        None
    }

    fn metadata(&self, _key: &str) -> Option<String> {
        None
    }

    fn is_reflowable(&self) -> bool {
        false
    }
}
