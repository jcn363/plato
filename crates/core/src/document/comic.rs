//! Comic Book Archive Support (CBZ/CBR)
//!
//! Provides support for comic book archive formats:
//! - CBZ: ZIP archive containing images
//! - CBR: RAR archive containing images
//!
//! Images are sorted alphabetically and displayed as pages.

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{format_err, Context, Error};
use rar::Archive;

use crate::framebuffer::Pixmap;
use crate::geom::{Boundary, CycleDir};
use crate::metadata::TextAlign;

use super::{BoundedText, Document, Location, TocEntry};

pub struct ComicDocument {
    path: std::path::PathBuf,
    pages: Vec<Vec<u8>>,
    page_count: usize,
    current_page: usize,
    dims: Vec<(f32, f32)>,
}

impl ComicDocument {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "cbz" | "zip" => Self::open_cbz(path),
            "cbr" | "rar" => Self::open_cbr(path),
            _ => Err(format_err!("Unsupported comic archive format: {}", ext)),
        }
    }

    fn open_cbz(path: &Path) -> Result<Self, Error> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("can't open CBZ file {}", path.display()))?;
        let mut archive = zip::ZipArchive::new(file)
            .with_context(|| format!("can't read CBZ archive {}", path.display()))?;

        let mut image_entries: Vec<(String, Vec<u8>)> = Vec::new();

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

        image_entries.sort_by(|a, b| a.0.cmp(&b.0));

        let page_count = image_entries.len();
        if page_count == 0 {
            return Err(format_err!("No images found in CBZ archive"));
        }

        let mut dims = Vec::with_capacity(page_count);
        let mut pages = Vec::with_capacity(page_count);

        for (_, data) in image_entries {
            if let Ok(img) = image::load_from_memory(&data) {
                dims.push((img.width() as f32, img.height() as f32));
            } else {
                dims.push((800.0, 1200.0));
            }
            pages.push(data);
        }

        Ok(ComicDocument {
            path: path.to_path_buf(),
            pages,
            page_count,
            current_page: 0,
            dims,
        })
    }

    fn open_cbr(path: &Path) -> Result<Self, Error> {
        let temp_dir = tempfile::tempdir()
            .with_context(|| format!("can't create temp dir for CBR extraction"))?;
        let temp_path = temp_dir.path();

        // Extract the CBR archive to temp directory
        let _archive = Archive::extract_all(
            path.to_str().ok_or_else(|| format_err!("Invalid path"))?,
            temp_path
                .to_str()
                .ok_or_else(|| format_err!("Invalid temp path"))?,
            "",
        )
        .with_context(|| format!("can't extract CBR archive {}", path.display()))?;

        let mut image_entries: Vec<(String, Vec<u8>)> = Vec::new();

        for entry in std::fs::read_dir(temp_path)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if Self::is_image_file(&name) {
                let data = std::fs::read(entry.path())
                    .with_context(|| format!("can't read extracted file {:?}", entry.path()))?;
                image_entries.push((entry.file_name().to_string_lossy().to_string(), data));
            }
        }

        // Temp directory is automatically cleaned up when temp_dir goes out of scope
        drop(temp_dir);

        image_entries.sort_by(|a, b| a.0.cmp(&b.0));

        let page_count = image_entries.len();
        if page_count == 0 {
            return Err(format_err!("No images found in CBR archive"));
        }

        let mut dims = Vec::with_capacity(page_count);
        let mut pages = Vec::with_capacity(page_count);

        for (_, data) in image_entries {
            if let Ok(img) = image::load_from_memory(&data) {
                dims.push((img.width() as f32, img.height() as f32));
            } else {
                dims.push((800.0, 1200.0));
            }
            pages.push(data);
        }

        Ok(ComicDocument {
            path: path.to_path_buf(),
            pages,
            page_count,
            current_page: 0,
            dims,
        })
    }

    fn is_image_file(name: &str) -> bool {
        name.ends_with(".jpg")
            || name.ends_with(".jpeg")
            || name.ends_with(".png")
            || name.ends_with(".gif")
            || name.ends_with(".bmp")
            || name.ends_with(".webp")
    }

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
        None
    }
    fn chapter<'a>(&mut self, _: usize, _: &'a [TocEntry]) -> Option<(&'a TocEntry, f32)> {
        None
    }
    fn chapter_relative<'a>(
        &mut self,
        _: usize,
        _: CycleDir,
        _: &'a [TocEntry],
    ) -> Option<&'a TocEntry> {
        None
    }
    fn words(&mut self, _: Location) -> Option<(Vec<BoundedText>, usize)> {
        None
    }
    fn lines(&mut self, _: Location) -> Option<(Vec<BoundedText>, usize)> {
        None
    }
    fn links(&mut self, _: Location) -> Option<(Vec<BoundedText>, usize)> {
        None
    }
    fn images(&mut self, _: Location) -> Option<(Vec<Boundary>, usize)> {
        None
    }

    fn pixmap(&mut self, loc: Location, _: f32, _: usize) -> Option<(Pixmap, usize)> {
        let page = match loc {
            Location::Exact(p) => p,
            Location::Previous(p) => p.saturating_sub(1),
            Location::Next(p) => (p + 1).min(self.page_count.saturating_sub(1)),
            _ => self.current_page,
        };

        if page >= self.page_count {
            return None;
        }

        let data = self.get_page_data(page)?;
        let img = image::load_from_memory(data).ok()?;
        let pixmap = Pixmap::from_dynamic_image(&img).ok()?;

        self.current_page = page;
        Some((pixmap, page))
    }

    fn layout(&mut self, _: u32, _: u32, _: f32, _: u16) {}
    fn set_font_family(&mut self, _: &str, _: &str) {}
    fn set_margin_width(&mut self, _: i32) {}
    fn set_text_align(&mut self, _: TextAlign) {}
    fn set_line_height(&mut self, _: f32) {}
    fn set_hyphen_penalty(&mut self, _: i32) {}
    fn set_stretch_tolerance(&mut self, _: f32) {}
    fn set_ignore_document_css(&mut self, _: bool) {}

    fn title(&self) -> Option<String> {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }

    fn author(&self) -> Option<String> {
        None
    }
    fn metadata(&self, _: &str) -> Option<String> {
        None
    }
    fn is_reflowable(&self) -> bool {
        false
    }
}
