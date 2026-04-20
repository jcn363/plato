//! PDF Resource Extraction Module
//!
//! Provides functionality for extracting resources from PDF documents
//! including images, fonts, and metadata.

use super::mupdf;
use crate::consts::pdf::MAX_FILE_SIZE_MB;
use crate::log_warn;
use anyhow::{format_err, Error};
use std::fs;
use std::path::{Path, PathBuf};

/// Extracted image information
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExtractedImage {
    pub page: usize,
    pub index: usize,
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

/// Extracted font information
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExtractedFont {
    pub name: String,
    pub data: Vec<u8>,
}

/// Resource extraction summary
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ResourceSummary {
    pub total_pages: usize,
    pub total_images: usize,
    pub total_fonts: usize,
    pub pages_with_images: Vec<usize>,
    pub is_pdf_a: bool,
    pub pdf_a_version: String,
}

impl Default for ResourceSummary {
    fn default() -> Self {
        ResourceSummary {
            total_pages: 0,
            total_images: 0,
            total_fonts: 0,
            pages_with_images: Vec::new(),
            is_pdf_a: false,
            pdf_a_version: String::new(),
        }
    }
}

/// PDF resource extractor
///
/// Extracts images, fonts, and other resources from PDF documents.
pub struct ResourceExtractor {
    doc: mupdf::Document,
    file_path: PathBuf,
    total_pages: usize,
}

impl ResourceExtractor {
    /// Create a new resource extractor for a PDF file
    pub fn new(path: &Path) -> Result<ResourceExtractor, Error> {
        let ctx = mupdf::MuPdfContext::new()?;

        let doc = ctx
            .open_document(path)
            .ok_or_else(|| format_err!("Failed to open PDF: {}", path.display()))?;

        let total_pages = doc.page_count() as usize;

        Ok(ResourceExtractor {
            doc,
            file_path: path.to_path_buf(),
            total_pages,
        })
    }

    /// Get the total page count
    pub fn page_count(&self) -> usize {
        self.total_pages
    }

    /// Extract images from a specific page
    pub fn extract_images_from_page(&self, page_num: usize) -> Result<Vec<ExtractedImage>, Error> {
        if page_num >= self.total_pages {
            return Err(format_err!("Page {} does not exist", page_num + 1));
        }

        let file_size = fs::metadata(&self.file_path).map(|m| m.len()).unwrap_or(0) / (1024 * 1024);
        if file_size > MAX_FILE_SIZE_MB {
            return Err(format_err!(
                "PDF file ({}MB) is too large. Maximum is {}MB.",
                file_size,
                MAX_FILE_SIZE_MB
            ));
        }

        let page = self
            .doc
            .load_page(page_num as i32)
            .map_err(|_| format_err!("Failed to load page {}", page_num + 1))?;

        let image_count = page.count_images();
        let mut images = Vec::with_capacity(image_count);

        for i in 0..image_count {
            if let Some(image) = page.load_image(i) {
                let width = image.width() as i32;
                let height = image.height() as i32;

                images.push(ExtractedImage {
                    page: page_num,
                    index: i,
                    width,
                    height,
                    data: Vec::new(),
                });
            }
        }

        Ok(images)
    }

    /// Extract images from all pages (up to max_pages limit)
    pub fn extract_all_images(&self, max_pages: usize) -> Result<Vec<ExtractedImage>, Error> {
        let mut all_images = Vec::new();
        let pages_to_scan = self.total_pages.min(max_pages);

        for page_num in 0..pages_to_scan {
            match self.extract_images_from_page(page_num) {
                Ok(images) => all_images.extend(images),
                Err(e) => {
                    log_warn!(
                        "Warning: Failed to extract images from page {}: {}",
                        page_num + 1,
                        e
                    );
                }
            }
        }

        Ok(all_images)
    }

    /// Count fonts on a specific page
    pub fn count_page_fonts(&self, page_num: usize) -> Result<usize, Error> {
        if page_num >= self.total_pages {
            return Err(format_err!("Page {} does not exist", page_num + 1));
        }

        let page = self
            .doc
            .load_page(page_num as i32)
            .map_err(|_| format_err!("Failed to load page {}", page_num + 1))?;

        Ok(page.count_fonts())
    }

    /// Extract text from a page (placeholder - use Plato's built-in text selection)
    pub fn extract_text_from_page(&self, page_num: usize) -> Result<String, Error> {
        if page_num >= self.total_pages {
            return Err(format_err!("Page {} does not exist", page_num + 1));
        }

        Ok(format!(
            "Text extraction for page {} - use Plato's built-in text selection",
            page_num + 1
        ))
    }

    /// Get a summary of all resources in the PDF
    pub fn list_resources(&self) -> Result<ResourceSummary, Error> {
        let mut summary = ResourceSummary {
            total_pages: self.total_pages,
            total_images: 0,
            total_fonts: 0,
            pages_with_images: Vec::new(),
            is_pdf_a: false,
            pdf_a_version: String::new(),
        };

        for page_num in 0..self.total_pages.min(20) {
            let images = self.extract_images_from_page(page_num)?;
            if !images.is_empty() {
                summary.total_images += images.len();
                summary.pages_with_images.push(page_num);
            }

            if let Ok(font_count) = self.count_page_fonts(page_num) {
                summary.total_fonts += font_count;
            }
        }

        summary.is_pdf_a = self.is_pdf_a();
        summary.pdf_a_version = self.pdf_a_version();

        Ok(summary)
    }

    /// Check if the PDF is PDF/A compliant
    pub fn is_pdf_a(&self) -> bool {
        !self.pdf_a_version().is_empty()
    }

    /// Get the PDF/A version if applicable
    pub fn pdf_a_version(&self) -> String {
        self.doc.pdf_output_intent().unwrap_or_default()
    }

    /// Read all annotations from the PDF
    pub fn read_annotations(&self) -> Result<Vec<super::PdfAnnotation>, Error> {
        let mut annotations = Vec::new();

        for page_num in 0..self.total_pages {
            if let Ok(page) = self.doc.load_page(page_num as i32) {
                if let Some(mut annot) = page.first_annot() {
                    loop {
                        let contents = annot.contents();
                        let rect = annot.rect();

                        annotations.push(super::PdfAnnotation {
                            page: page_num,
                            annot_type: "Unknown".to_string(),
                            contents,
                            rect: Some((rect.x0, rect.y0, rect.x1, rect.y1)),
                            color: None,
                        });

                        match annot.next() {
                            Some(next) => annot = next,
                            None => break,
                        }
                    }
                }
            }
        }

        Ok(annotations)
    }
}
