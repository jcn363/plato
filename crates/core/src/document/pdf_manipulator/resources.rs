//! PDF Resource Extraction Module
//!
//! Provides functionality for extracting resources from PDF documents
//! including images, fonts, and metadata.
//!
//! Implemented using lopdf for PDF manipulation

use crate::log_info;
use anyhow::{format_err, Error};
use lopdf::Document;
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
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct ResourceSummary {
    pub total_pages: usize,
    pub total_images: usize,
    pub total_fonts: usize,
    pub pages_with_images: Vec<usize>,
    pub is_pdf_a: bool,
    pub pdf_a_version: String,
}

/// PDF resource extractor
///
/// Extracts images, fonts, and other resources from PDF documents.
pub struct ResourceExtractor {
    _file_path: PathBuf,
    total_pages: usize,
}

impl ResourceExtractor {
    /// Create a new resource extractor
    pub fn new(path: &Path) -> Result<ResourceExtractor, Error> {
        let doc = super::super::pdfpurr::Document::open(path)
            .map_err(|e| format_err!("Failed to open PDF: {}", e))?;

        let total_pages = doc.page_count();

        Ok(ResourceExtractor {
            _file_path: path.to_path_buf(),
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

        log_info!("Extracting images from page {}", page_num + 1);

        // Load the PDF document using lopdf
        let doc = Document::load(&self._file_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        let pages_map = doc.get_pages();
        let page_ids: Vec<_> = pages_map.values().collect();
        let page_index = page_num;
        if page_index >= page_ids.len() {
            return Ok(Vec::new());
        }

        let page_id = page_ids
            .get(page_index)
            .ok_or_else(|| format_err!("page index out of bounds after check"))?;
        let page_dict = doc
            .get_object(**page_id)
            .map_err(|e| format_err!("page object not found: {}", e))?
            .as_dict()
            .map_err(|_| format_err!("page object is not a dictionary"))?;

        let mut images = Vec::new();

        // Get resources dictionary
        if let Ok(resources) = page_dict.get(b"Resources") {
            if let Ok(res_dict) = resources.as_dict() {
                // Get XObject dictionary for images
                if let Ok(xobject) = res_dict.get(b"XObject") {
                    if let Ok(xobj_dict) = xobject.as_dict() {
                        let mut image_index = 0;
                        for (_name, obj_ref) in xobj_dict.iter() {
                            if let Ok(dict) = obj_ref.as_dict() {
                                // Check if it's an image
                                if dict.get(b"Subtype").is_ok() {
                                    let subtype = dict
                                        .get(b"Subtype")
                                        .map_err(|_| format_err!("subtype missing after check"))?;
                                    if let Ok(name_bytes) = subtype.as_name() {
                                        if name_bytes == b"Image" {
                                            // Extract image dimensions if available
                                            let width = dict
                                                .get(b"Width")
                                                .and_then(|w| w.as_i64())
                                                .unwrap_or(0)
                                                as i32;
                                            let height = dict
                                                .get(b"Height")
                                                .and_then(|h| h.as_i64())
                                                .unwrap_or(0)
                                                as i32;

                                            images.push(ExtractedImage {
                                                page: page_num,
                                                index: image_index,
                                                width,
                                                height,
                                                data: Vec::new(), // Image data extraction requires more complex handling
                                            });
                                            image_index += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        log_info!("Found {} images on page {}", images.len(), page_num + 1);
        Ok(images)
    }

    /// Extract images from all pages (up to max_pages limit)
    pub fn extract_all_images(&self, max_pages: usize) -> Result<Vec<ExtractedImage>, Error> {
        log_info!("Extracting images from all pages (max {})", max_pages);

        let mut all_images = Vec::new();
        let pages_to_scan = self.total_pages.min(max_pages);

        for page_num in 0..pages_to_scan {
            match self.extract_images_from_page(page_num) {
                Ok(mut images) => all_images.append(&mut images),
                Err(e) => {
                    log_info!("Failed to extract images from page {}: {}", page_num + 1, e);
                }
            }
        }

        log_info!("Extracted {} total images", all_images.len());
        Ok(all_images)
    }

    /// Count fonts on a specific page
    pub fn count_page_fonts(&self, page_num: usize) -> Result<usize, Error> {
        if page_num >= self.total_pages {
            return Err(format_err!("Page {} does not exist", page_num + 1));
        }

        log_info!("Counting fonts on page {}", page_num + 1);

        // Load the PDF document using lopdf
        let doc = Document::load(&self._file_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        let pages_map = doc.get_pages();
        let page_ids: Vec<_> = pages_map.values().collect();
        let page_index = page_num;
        if page_index >= page_ids.len() {
            return Ok(0);
        }

        let page_id = page_ids
            .get(page_index)
            .ok_or_else(|| format_err!("page index out of bounds after check"))?;
        let page_dict = doc
            .get_object(**page_id)
            .map_err(|e| format_err!("page object not found: {}", e))?
            .as_dict()
            .map_err(|_| format_err!("page object is not a dictionary"))?;

        let mut font_count = 0;

        // Get resources dictionary
        if let Ok(resources) = page_dict.get(b"Resources") {
            if let Ok(res_dict) = resources.as_dict() {
                // Get Font dictionary
                if let Ok(font_dict) = res_dict.get(b"Font") {
                    if let Ok(fonts) = font_dict.as_dict() {
                        font_count = fonts.len();
                    }
                }
            }
        }

        log_info!("Found {} fonts on page {}", font_count, page_num + 1);
        Ok(font_count)
    }

    /// Extract text from a page using PDFPurr
    pub fn extract_text_from_page(&self, page_num: usize) -> Result<String, Error> {
        if page_num >= self.total_pages {
            return Err(format_err!("Page {} does not exist", page_num + 1));
        }

        log_info!("Extracting text from page {}", page_num + 1);

        let doc = super::super::pdfpurr::Document::open(&self._file_path)
            .map_err(|e| format_err!("Failed to open PDF: {}", e))?;

        let page = doc
            .load_page(page_num as i32)
            .map_err(|e| format_err!("Failed to get page {}: {}", page_num + 1, e))?;

        let text_page = page
            .to_text_page(None)
            .ok_or_else(|| format_err!("Failed to extract text from page {}", page_num + 1))?;

        let text: String = text_page
            .blocks()
            .iter()
            .flat_map(|block| block.lines())
            .flat_map(|line| line.chars())
            .map(|c| char::from_u32(c.char_code as u32).unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();

        log_info!(
            "Extracted {} characters from page {}",
            text.len(),
            page_num + 1
        );
        Ok(text)
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
        // Load the PDF document using lopdf
        if let Ok(doc) = Document::load(&self._file_path) {
            // Check for PDF/A metadata in the document catalog
            if let Ok(catalog) = doc.catalog() {
                if let Ok(metadata) = catalog.get(b"Metadata") {
                    // Check for PDF/A identifier in metadata
                    if let Ok(metadata_ref) = metadata.as_reference() {
                        if let Ok(metadata_obj) = doc.get_object(metadata_ref) {
                            if let Ok(metadata_stream) = metadata_obj.as_stream() {
                                if let Ok(content) = metadata_stream.get_plain_content() {
                                    let content_str = String::from_utf8_lossy(&content);
                                    if content_str.contains("pdfaid")
                                        || content_str.contains("PDF/A")
                                    {
                                        if content_str.contains("1") {
                                            return "PDF/A-1".to_string();
                                        } else if content_str.contains("2") {
                                            return "PDF/A-2".to_string();
                                        } else if content_str.contains("3") {
                                            return "PDF/A-3".to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        String::new()
    }

    /// Read all annotations from the PDF
    pub fn read_annotations(&self) -> Result<Vec<super::PdfAnnotation>, Error> {
        log_info!("Reading annotations from PDF");

        // Load the PDF document using lopdf
        let doc = Document::load(&self._file_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        let mut annotations = Vec::new();

        // Get pages map
        let pages_map = doc.get_pages();

        // Iterate through all pages
        for (page_num, page_id) in pages_map {
            let page_dict = doc
                .get_object(page_id)
                .map_err(|e| format_err!("page object not found: {}", e))?
                .as_dict()
                .map_err(|_| format_err!("page object is not a dictionary"))?;

            // Get annotations array
            if let Ok(annot_ref) = page_dict.get(b"Annots") {
                if let Ok(annot_array) = annot_ref.as_array() {
                    for annot_obj_ref in annot_array {
                        if let Ok(annot_ref) = annot_obj_ref.as_reference() {
                            if let Ok(annot_obj) = doc.get_object(annot_ref) {
                                if let Ok(annot_dict) = annot_obj.as_dict() {
                                    // Extract annotation type
                                    let subtype_str = annot_dict
                                        .get(b"Subtype")
                                        .and_then(|s| s.as_name())
                                        .map(|n| String::from_utf8_lossy(n).to_string())
                                        .unwrap_or_else(|_| "Text".to_string());
                                    let subtype = super::AnnotationSubtype::from_str(&subtype_str)
                                        .unwrap_or(super::AnnotationSubtype::Text);

                                    // Extract contents
                                    let contents = match annot_dict.get(b"Contents") {
                                        Ok(obj) => obj
                                            .as_str()
                                            .ok()
                                            .and_then(|s| std::str::from_utf8(s).ok())
                                            .unwrap_or("")
                                            .to_string(),
                                        Err(_) => String::new(),
                                    };

                                    // Extract rectangle
                                    let _rect = annot_dict
                                        .get(b"Rect")
                                        .and_then(|r| r.as_array())
                                        .ok()
                                        .and_then(|arr| {
                                            if arr.len() >= 4 {
                                                Some((
                                                    arr[0]
                                                        .as_i64()
                                                        .ok()
                                                        .map(|f| f as f32)
                                                        .unwrap_or(0.0),
                                                    arr[1]
                                                        .as_i64()
                                                        .ok()
                                                        .map(|f| f as f32)
                                                        .unwrap_or(0.0),
                                                    arr[2]
                                                        .as_i64()
                                                        .ok()
                                                        .map(|f| f as f32)
                                                        .unwrap_or(0.0),
                                                    arr[3]
                                                        .as_i64()
                                                        .ok()
                                                        .map(|f| f as f32)
                                                        .unwrap_or(0.0),
                                                ))
                                            } else {
                                                None
                                            }
                                        });

                                    annotations.push(super::PdfAnnotation::new(
                                        page_num as usize,
                                        subtype,
                                        contents,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        log_info!("Found {} annotations", annotations.len());
        Ok(annotations)
    }
}
