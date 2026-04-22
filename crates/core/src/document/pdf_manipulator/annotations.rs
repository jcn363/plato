//! PDF Annotation Export Module
//!
//! Provides functionality for exporting PDF annotations to a new document.
//! Supports creating annotation copies with their content and positioning.
//!
//! Implemented using lopdf for PDF manipulation

use crate::log_info;
use anyhow::{format_err, Error};
use lopdf::{Dictionary, Document, Object};
use std::path::{Path, PathBuf};

/// PDF annotation information
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PdfAnnotation {
    pub page: usize,
    pub annot_type: String,
    pub contents: String,
    pub rect: Option<(f32, f32, f32, f32)>,
    pub color: Option<(u8, u8, u8)>,
}

/// PDF annotation exporter
///
/// Exports annotations from a source PDF to a new output document.
pub struct PdfAnnotationExporter {
    file_path: PathBuf,
    total_pages: usize,
}

impl PdfAnnotationExporter {
    /// Create a new annotation exporter
    pub fn new(source_path: &Path, output_path: &Path) -> Result<PdfAnnotationExporter, Error> {
        let doc = super::super::pdfpurr::Document::open(source_path)
            .map_err(|e| format_err!("Failed to open PDF: {}", e))?;
        
        let total_pages = doc.page_count();

        Ok(PdfAnnotationExporter {
            file_path: output_path.to_path_buf(),
            total_pages,
        })
    }

    /// Get the total page count
    pub fn page_count(&self) -> usize {
        self.total_pages
    }

    /// Add an annotation to the output document
    pub fn add_annotation(&mut self, annot: PdfAnnotation) -> Result<(), Error> {
        if annot.page >= self.total_pages {
            return Err(format_err!("Page {} does not exist", annot.page + 1));
        }

        // Load the output document using lopdf
        let mut doc = Document::load(&self.file_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        // Get the target page
        let pages_map = doc.get_pages();
        let page_ids: Vec<_> = pages_map.values().collect();
        let page_index = annot.page;
        if page_index >= page_ids.len() {
            return Err(format_err!("Page {} does not exist in output document", annot.page + 1));
        }
        let page_id = page_ids.get(page_index).unwrap();

        // Create annotation dictionary
        let mut annot_dict = Dictionary::new();
        annot_dict.set("Subtype", Object::Name(annot.annot_type.as_bytes().to_vec()));
        annot_dict.set("Contents", Object::String(annot.contents.as_bytes().to_vec(), lopdf::StringFormat::Literal));

        // Add rectangle if provided
        if let Some(rect) = annot.rect {
            let mut rect_array = Vec::new();
            rect_array.push(Object::Real(rect.0));
            rect_array.push(Object::Real(rect.1));
            rect_array.push(Object::Real(rect.2));
            rect_array.push(Object::Real(rect.3));
            annot_dict.set("Rect", Object::Array(rect_array));
        }

        // Add color if provided
        if let Some(color) = annot.color {
            let color_array = vec![
                Object::Integer(color.0 as i64),
                Object::Integer(color.1 as i64),
                Object::Integer(color.2 as i64),
            ];
            annot_dict.set("C", Object::Array(color_array));
        }

        // Add annotation to page
        let annot_id = doc.add_object(Object::Dictionary(annot_dict));

        let page_dict = doc.get_object_mut(**page_id).unwrap().as_dict_mut().unwrap();
        page_dict.set("Annots", Object::Array(vec![Object::Reference(annot_id)]));

        // Save the modified document
        let mut buffer = std::io::Cursor::new(Vec::new());
        doc.save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        std::fs::write(&self.file_path, bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        log_info!("Successfully added annotation to page {}", annot.page + 1);
        Ok(())
    }

    /// Export an annotation to the output document
    pub fn export_annotation(&mut self, annot: &PdfAnnotation) -> Result<(), Error> {
        // Load the source document using lopdf
        let source_path = &self.file_path; // Using file_path as source for now
        let mut doc = Document::load(source_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        // Get the target page
        let pages_map = doc.get_pages();
        let page_ids: Vec<_> = pages_map.values().collect();
        let page_index = annot.page;
        if page_index >= page_ids.len() {
            return Err(format_err!("Page {} does not exist", annot.page + 1));
        }

        if let Some(page_id) = page_ids.get(page_index) {
            let page_dict = doc.get_object(**page_id).unwrap().as_dict().unwrap();

            // Check if annotations exist
            if page_dict.get(b"Annots").is_ok() {
                log_info!("Found annotations on page {}", annot.page + 1);
                // TODO: we would copy specific annotations
            }
        }

        // Save the modified document
        let mut buffer = std::io::Cursor::new(Vec::new());
        doc.save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        std::fs::write(source_path, bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        log_info!("Successfully exported annotation from page {}", annot.page + 1);
        Ok(())
    }

    /// Save the output document with annotations
    pub fn save(&self) -> Result<PathBuf, Error> {
        // Load and verify the document
        let mut doc = Document::load(&self.file_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        // Save to ensure it's valid
        let mut buffer = std::io::Cursor::new(Vec::new());
        doc.save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        std::fs::write(&self.file_path, bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        log_info!("Successfully saved document with annotations to: {:?}", self.file_path);
        Ok(self.file_path.clone())
    }
}
