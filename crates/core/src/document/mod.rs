//! Document Handling Module
//!
//! This module provides document loading, rendering, and manipulation for multiple
//! document formats including PDF, EPUB, HTML, and various image formats.
//!
//! ## Architecture
//!
//! The document module is organized by format and function:
//!
//! ### Format-Specific Modules
//! - **pdf/**: PDF document support via MuPDF
//!   - `PdfOpener`, `PdfDocument`, `PdfPage` for PDF handling
//!   - Table of contents, links, annotations
//! - **epub/**: EPUB e-book support
//!   - `EpubDocument` for EPUB rendering
//!   - NCX/Navigation parsing
//! - **html/**: HTML document support
//!   - Custom HTML/CSS rendering engine optimized for e-ink
//!   - DOM, layout, text shaping, line breaking
//! - **mupdf/**: MuPDF integration layer
//!   - Safe wrappers for MuPDF FFI
//!   - Context management, document abstraction
//!
//! ### Support Modules
//! - **pdf_manipulator/**: PDF manipulation tools (merge, split, redact)
//! - **progressive_loader.rs**: Progressive document loading with caching
//! - **mupdf_sys.rs**: Low-level MuPDF FFI bindings
//! - **sysinfo.rs**: System information HTML generator
//!
//! ## Module Hierarchy
//!
//! ```text
//! document/
//! ├── mod.rs              (core Document trait and shared types)
//! ├── pdf/                (PDF format support)
//! │   ├── mod.rs          (PdfOpener, PdfDocument, PdfPage)
//! │   └── text.rs         (PDF text extraction)
//! ├── epub/               (EPUB format support)
//! │   ├── mod.rs          (EpubDocument)
//! │   └── opener.rs       (EPUB opening utilities)
//! ├── html/               (HTML format support)
//! │   ├── mod.rs          (HtmlDocument, Engine)
//! │   ├── css.rs          (CSS parsing)
//! │   ├── dom.rs          (DOM structure)
//! │   ├── engine.rs       (Layout engine)
//! │   ├── engine_*.rs     (Engine components)
//! │   ├── layout.rs       (Text layout)
//! │   ├── parse.rs        (HTML parsing)
//! │   └── engine_text/    (Text processing)
//! ├── mupdf/              (MuPDF integration)
//! │   ├── mod.rs          (Safe wrappers)
//! │   ├── context.rs      (Context management)
//! │   ├── document.rs     (Document abstraction)
//! │   ├── page.rs         (Page abstraction)
//! │   ├── pixmap.rs       (Pixmap handling)
//! │   └── text.rs         (Text extraction)
//! ├── mupdf_sys.rs        (FFI bindings)
//! ├── pdf_manipulator/    (PDF tools)
//! │   ├── mod.rs          (PdfManipulator core)
//! │   ├── redaction.rs    (Redaction editor)
//! │   ├── resources.rs    (Resource extraction)
//! │   └── annotations.rs  (Annotation export)
//! ├── progressive_loader.rs (Progressive loading)
//! └── sysinfo.rs          (System info HTML generator)
//! ```
//!
//! ## Core Trait
//!
//! The `Document` trait defines the interface all document types implement:
//! - Page access and rendering
//! - Table of contents/navigation
//! - Text extraction and search
//! - Annotation support
//!
//! ## Dependencies
//!
//! - `mupdf` - PDF rendering via MuPDF
//! - `html` - HTML/CSS engine
//! - `framebuffer` - Output rendering
//! - `metadata` - Document metadata

pub mod epub;
pub mod html;
pub mod mupdf;
pub mod pdf;
pub mod pdf_manipulator;
pub mod progressive_loader;
pub mod sysinfo;

mod mupdf_sys;

#[cfg(test)]
mod document_tests;

use self::epub::EpubDocument;
use self::html::HtmlDocument;
use self::pdf::PdfOpener;
use crate::framebuffer::Pixmap;
use crate::geom::{Boundary, CycleDir, Point};
use crate::log_error;
use crate::metadata::{Annotation, TextAlign};
use crate::validation::validate_path;
use anyhow::{format_err, Context, Error};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::Path;
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

pub const BYTES_PER_PAGE: f64 = 2048.0;

#[derive(Debug, Clone)]
pub enum Location {
    Exact(usize),
    Previous(usize),
    Next(usize),
    LocalUri(usize, String),
    Uri(String),
}

impl Location {
    pub fn as_page(&self) -> Option<usize> {
        match self {
            Location::Exact(page) => Some(*page),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoundedText {
    pub text: String,
    pub rect: Boundary,
    pub location: Point,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextLocation {
    Static(usize, usize),
    Dynamic(usize),
}

impl std::fmt::Display for TextLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TextLocation::Static(page, pos) => write!(f, "{},{}", page, pos),
            TextLocation::Dynamic(offset) => write!(f, "{}", offset),
        }
    }
}

impl TextLocation {
    pub fn location(self) -> usize {
        match self {
            TextLocation::Static(page, _) => page,
            TextLocation::Dynamic(offset) => offset,
        }
    }

    #[inline]
    pub fn min_max(self, other: Self) -> (Self, Self) {
        if self < other {
            (self, other)
        } else {
            (other, self)
        }
    }
}

#[derive(Debug, Clone)]
pub struct TocEntry {
    pub title: String,
    pub location: Location,
    pub index: usize,
    pub children: Vec<TocEntry>,
    #[doc(hidden)]
    pub page: Option<usize>,
    #[doc(hidden)]
    pub level: usize,
}

impl TocEntry {
    pub fn new(title: String, location: Location, index: usize, children: Vec<TocEntry>) -> Self {
        let page = location.as_page();
        let level = 0;
        Self {
            title,
            location,
            index,
            children,
            page,
            level,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Neighbors {
    pub previous_page: Option<usize>,
    pub next_page: Option<usize>,
}

pub trait Document: Send + Sync {
    fn dims(&self, index: usize) -> Option<(f32, f32)>;
    fn pages_count(&self) -> usize;

    fn toc(&mut self) -> Option<Vec<TocEntry>>;
    fn chapter<'a>(&mut self, offset: usize, toc: &'a [TocEntry]) -> Option<(&'a TocEntry, f32)>;
    fn chapter_relative<'a>(
        &mut self,
        offset: usize,
        dir: CycleDir,
        toc: &'a [TocEntry],
    ) -> Option<&'a TocEntry>;
    fn words(&mut self, loc: Location) -> Option<(Vec<BoundedText>, usize)>;
    fn lines(&mut self, loc: Location) -> Option<(Vec<BoundedText>, usize)>;
    fn links(&mut self, loc: Location) -> Option<(Vec<BoundedText>, usize)>;
    fn images(&mut self, loc: Location) -> Option<(Vec<Boundary>, usize)>;

    fn pixmap(&mut self, loc: Location, scale: f32, samples: usize) -> Option<(Pixmap, usize)>;
    fn layout(&mut self, width: u32, height: u32, font_size: f32, dpi: u16);

    /// Sets the font family for text rendering.
    fn set_font_family(&mut self, family_name: &str, search_path: &str);

    /// Sets the page margin width.
    fn set_margin_width(&mut self, width: i32);

    /// Sets the text alignment.
    fn set_text_align(&mut self, text_align: TextAlign);

    /// Sets the line height multiplier.
    fn set_line_height(&mut self, line_height: f32);

    /// Sets the hyphen penalty for text reflow.
    fn set_hyphen_penalty(&mut self, hyphen_penalty: i32);

    /// Sets the stretch tolerance for text reflow.
    fn set_stretch_tolerance(&mut self, stretch_tolerance: f32);

    fn set_ignore_document_css(&mut self, ignore: bool);

    fn title(&self) -> Option<String>;
    fn author(&self) -> Option<String>;
    fn metadata(&self, key: &str) -> Option<String>;

    fn is_reflowable(&self) -> bool;

    fn is_encrypted(&self) -> bool {
        false
    }

    fn unlock(&mut self, _password: &str) -> Result<bool, Error> {
        Err(format_err!("this document type doesn't support encryption"))
    }

    fn auto_crop_margins(
        &mut self,
        _color_samples: usize,
        _threshold: u8,
    ) -> Option<(f32, f32, f32, f32)> {
        None
    }

    fn has_synthetic_page_numbers(&self) -> bool {
        false
    }

    fn save(&self, _path: &str) -> Result<(), Error> {
        Err(format_err!("this document can't be saved"))
    }

    fn search_page(
        &mut self,
        _page_index: usize,
        _text: &str,
        _max_results: usize,
    ) -> Option<Vec<TextLocation>> {
        None
    }

    fn preview_pixmap(&mut self, width: f32, height: f32, samples: usize) -> Option<Pixmap> {
        self.dims(0)
            .and_then(|dims| {
                let scale = (width / dims.0).min(height / dims.1);
                self.pixmap(Location::Exact(0), scale, samples)
            })
            .map(|(pixmap, _)| pixmap)
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
            _ => None,
        }
    }
}

pub fn file_kind<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_lowercase)
        .or_else(|| guess_kind(path.as_ref()).ok().map(String::from))
}

pub fn guess_kind<P: AsRef<Path>>(path: P) -> Result<&'static str, Error> {
    // Validate path before attempting to open
    validate_path(&path, "document path")?;
    let file = File::open(path.as_ref())
        .with_context(|| format!("can't open file {}", path.as_ref().display()))?;
    let mut magic = [0; 4];
    file.read_exact_at(&mut magic, 0)?;

    if &magic == b"PK\x03\x04" {
        let mut mime_type = [0; 28];
        file.read_exact_at(&mut mime_type, 30)?;
        if &mime_type == b"mimetypeapplication/epub+zip" {
            return Ok("epub");
        }
    } else if &magic == b"%PDF" {
        return Ok("pdf");
    }

    Err(format_err!("Unknown file type"))
}

pub trait HumanSize {
    fn human_size(&self) -> String;
}

const SIZE_BASE: f32 = 1024.0;

impl HumanSize for u64 {
    fn human_size(&self) -> String {
        let value = *self as f32;
        let level = (value.max(1.0).log(SIZE_BASE).floor() as usize).min(3);
        let factor = value / (SIZE_BASE).powi(level as i32);
        let precision = level.saturating_sub(1 + factor.log(10.0).floor() as usize);
        format!(
            "{0:.1$} {2}",
            factor,
            precision,
            ['B', 'K', 'M', 'G'][level]
        )
    }
}

impl HumanSize for u32 {
    fn human_size(&self) -> String {
        (*self as u64).human_size()
    }
}

pub fn asciify(name: &str) -> String {
    name.nfkd()
        .filter(|&c| !is_combining_mark(c))
        .collect::<String>()
        .replace('œ', "oe")
        .replace('Œ', "Oe")
        .replace('æ', "ae")
        .replace('Æ', "Ae")
        .replace(['—', '–'], "-")
        .replace('’', "'")
}

pub fn open<P: AsRef<Path>>(path: P) -> Option<Box<dyn Document>> {
    // Validate path before attempting to open
    if let Err(e) = validate_path(&path, "document path") {
        log_error!("Failed to open document {}: {}", path.as_ref().display(), e);
        return None;
    }
    file_kind(path.as_ref()).and_then(|k| match k.as_ref() {
        "epub" => EpubDocument::new(&path)
            .map_err(|e| {
                log_error!(
                    "Failed to open EPUB {}: {}. Please check the file is not corrupted.",
                    path.as_ref().display(),
                    e
                )
            })
            .map(|d| Box::new(d) as Box<dyn Document>)
            .ok(),
        "html" | "htm" => HtmlDocument::new(&path)
            .map_err(|e| {
                log_error!(
                    "Failed to open HTML {}: {}. Please check the file is valid.",
                    path.as_ref().display(),
                    e
                )
            })
            .map(|d| Box::new(d) as Box<dyn Document>)
            .ok(),
        _ => PdfOpener::new().and_then(|mut o| {
            if matches!(k.as_ref(), "mobi" | "fb2" | "fbz" | "xps" | "txt") {
                o.load_user_stylesheet();
            }
            o.open(path).map(|d| Box::new(d) as Box<dyn Document>)
        }),
    })
}

pub fn open_html(html: &str) -> Result<Box<dyn Document>, Error> {
    // Input validation: ensure HTML is not empty
    if html.trim().is_empty() {
        return Err(format_err!("HTML content cannot be empty"));
    }
    let doc = HtmlDocument::new_from_memory(html);
    Ok(Box::new(doc) as Box<dyn Document>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SimpleTocEntry {
    Leaf(String, TocLocation),
    Container(String, TocLocation, Vec<SimpleTocEntry>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TocLocation {
    Exact(usize),
    Uri(String),
}

impl From<TocLocation> for Location {
    fn from(loc: TocLocation) -> Location {
        match loc {
            TocLocation::Exact(n) => Location::Exact(n),
            TocLocation::Uri(uri) => Location::Uri(uri),
        }
    }
}

pub fn toc_as_html(toc: &[TocEntry], chap_index: usize) -> String {
    let mut buf = "<html>\n\t<head>\n\t\t<title>Table of Contents</title>\n\t\t\
                   <link rel=\"stylesheet\" type=\"text/css\" href=\"css/toc.css\"/>\n\t\
                   </head>\n\t<body>\n"
        .to_string();
    toc_as_html_aux(toc, chap_index, 0, &mut buf);
    buf.push_str("\t</body>\n</html>");
    buf
}

pub fn toc_as_html_aux(toc: &[TocEntry], chap_index: usize, depth: usize, buf: &mut String) {
    buf.push_str(&"\t".repeat(depth + 2));
    buf.push_str("<ul>\n");
    for entry in toc {
        buf.push_str(&"\t".repeat(depth + 3));
        match entry.location {
            Location::Exact(n) => buf.push_str(&format!("<li><a href=\"@{}\">", n)),
            Location::Uri(ref uri) => buf.push_str(&format!("<li><a href=\"@{}\">", uri)),
            _ => buf.push_str("<li><a href=\"#\">"),
        }
        let title = entry.title.replace('<', "&lt;").replace('>', "&gt;");
        if entry.index == chap_index {
            buf.push_str(&format!("<strong>{}</strong>", title));
        } else {
            buf.push_str(&title);
        }
        buf.push_str("</a></li>\n");
        if !entry.children.is_empty() {
            toc_as_html_aux(&entry.children, chap_index, depth + 1, buf);
        }
    }
    buf.push_str(&"\t".repeat(depth + 2));
    buf.push_str("</ul>\n");
}

pub fn annotations_as_html(
    annotations: &[Annotation],
    active_range: Option<(TextLocation, TextLocation)>,
) -> String {
    let mut buf = "<html>\n\t<head>\n\t\t<title>Annotations</title>\n\t\t\
                   <link rel=\"stylesheet\" type=\"text/css\" href=\"css/annotations.css\"/>\n\t\
                   </head>\n\t<body>\n"
        .to_string();
    buf.push_str("\t\t<ul>\n");
    for annot in annotations {
        let mut note = annot.note.replace('<', "&lt;").replace('>', "&gt;");
        let mut text = annot.text.replace('<', "&lt;").replace('>', "&gt;");
        let start = annot.selection[0];
        if active_range
            .map(|(first, last)| start >= first && start <= last)
            .unwrap_or(false)
        {
            if !note.is_empty() {
                note = format!("<b>{}</b>", note);
            }
            text = format!("<b>{}</b>", text);
        }
        if note.is_empty() {
            buf.push_str(&format!(
                "\t\t<li><a href=\"@{}\">{}</a></li>\n",
                start.location(),
                text
            ));
        } else {
            buf.push_str(&format!(
                "\t\t<li><a href=\"@{}\"><i>{}</i> — {}</a></li>\n",
                start.location(),
                note,
                text
            ));
        }
    }
    buf.push_str("\t\t</ul>\n");
    buf.push_str("\t</body>\n</html>");
    buf
}

pub fn bookmarks_as_html(bookmarks: &BTreeSet<usize>, index: usize, synthetic: bool) -> String {
    let mut buf = "<html>\n\t<head>\n\t\t<title>Bookmarks</title>\n\t\t\
                   <link rel=\"stylesheet\" type=\"text/css\" href=\"css/bookmarks.css\"/>\n\t\
                   </head>\n\t<body>\n"
        .to_string();
    buf.push_str("\t\t<ul>\n");
    for bkm in bookmarks {
        let mut text = if synthetic {
            format!("{:.1}", *bkm as f64 / BYTES_PER_PAGE)
        } else {
            format!("{}", bkm + 1)
        };
        if *bkm == index {
            text = format!("<b>{}</b>", text);
        }
        buf.push_str(&format!("\t\t<li><a href=\"@{}\">{}</a></li>\n", bkm, text));
    }
    buf.push_str("\t\t</ul>\n");
    buf.push_str("\t</body>\n</html>");
    buf
}

#[inline]
fn chapter(index: usize, pages_count: usize, toc: &[TocEntry]) -> Option<(&TocEntry, f32)> {
    let mut chap = None;
    let mut chap_index = 0;
    let mut end_index = pages_count;
    chapter_aux(toc, index, &mut chap, &mut chap_index, &mut end_index);
    chap.zip(Some(
        (index - chap_index) as f32 / (end_index - chap_index) as f32,
    ))
}

fn chapter_aux<'a>(
    toc: &'a [TocEntry],
    index: usize,
    chap: &mut Option<&'a TocEntry>,
    chap_index: &mut usize,
    end_index: &mut usize,
) {
    for entry in toc {
        if let Location::Exact(entry_index) = entry.location {
            if entry_index <= index && (chap.is_none() || entry_index > *chap_index) {
                *chap = Some(entry);
                *chap_index = entry_index;
            }
            if entry_index > index && entry_index < *end_index {
                *end_index = entry_index;
            }
        }
        chapter_aux(&entry.children, index, chap, chap_index, end_index);
    }
}

#[inline]
fn chapter_relative(index: usize, dir: CycleDir, toc: &[TocEntry]) -> Option<&TocEntry> {
    let chap = chapter(index, usize::MAX, toc).map(|(c, _)| c);

    match dir {
        CycleDir::Previous => previous_chapter(chap, index, toc),
        CycleDir::Next => next_chapter(chap, index, toc),
    }
}

fn previous_chapter<'a>(
    chap: Option<&TocEntry>,
    index: usize,
    toc: &'a [TocEntry],
) -> Option<&'a TocEntry> {
    for entry in toc.iter().rev() {
        let result = previous_chapter(chap, index, &entry.children);
        if result.is_some() {
            return result;
        }

        if let Some(chap) = chap {
            if entry.index < chap.index {
                if let Location::Exact(entry_index) = entry.location {
                    if entry_index != index {
                        return Some(entry);
                    }
                }
            }
        } else {
            if let Location::Exact(entry_index) = entry.location {
                if entry_index < index {
                    return Some(entry);
                }
            }
        }
    }
    None
}

fn next_chapter<'a>(
    chap: Option<&TocEntry>,
    index: usize,
    toc: &'a [TocEntry],
) -> Option<&'a TocEntry> {
    for entry in toc {
        if let Some(chap) = chap {
            if entry.index > chap.index {
                if let Location::Exact(entry_index) = entry.location {
                    if entry_index != index {
                        return Some(entry);
                    }
                }
            }
        } else {
            if let Location::Exact(entry_index) = entry.location {
                if entry_index > index {
                    return Some(entry);
                }
            }
        }

        let result = next_chapter(chap, index, &entry.children);
        if result.is_some() {
            return result;
        }
    }
    None
}

pub fn chapter_from_uri<'a>(target_uri: &str, toc: &'a [TocEntry]) -> Option<&'a TocEntry> {
    for entry in toc {
        if let Location::Uri(ref uri) = entry.location {
            if uri.starts_with(target_uri) {
                return Some(entry);
            }
        }
        let result = chapter_from_uri(target_uri, &entry.children);
        if result.is_some() {
            return result;
        }
    }
    None
}

// Re-export sys_info_as_html from sysinfo module for backwards compatibility
pub use sysinfo::sys_info_as_html;
