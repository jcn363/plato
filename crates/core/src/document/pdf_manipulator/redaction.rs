//! PDF Redaction Module
//!
//! Provides functionality for redacting sensitive content from PDF documents.
//! Supports defining redaction regions and applying them to permanently remove
//! content from PDF files.
//!
//! Implemented using lopdf for PDF manipulation

use crate::consts::pdf::{MAX_FILE_SIZE_MB, MAX_PAGES_HARD_LIMIT, WARNING_FILE_SIZE_MB};
use crate::{log_info, log_warn};
use anyhow::{format_err, Error};
use lopdf::{Dictionary, Document, Object};
use std::fs;
use std::path::{Path, PathBuf};

/// Redaction region definition
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RedactionRegion {
    pub page: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// PDF redaction editor
///
/// Manages redaction regions and applies them to PDF documents.
/// Provides preview, add/remove regions, and apply operations.
pub struct RedactionEditor {
    file_path: PathBuf,
    regions: Vec<RedactionRegion>,
    current_page: usize,
    total_pages: usize,
    modified: bool,
}

impl RedactionEditor {
    /// Create a new redaction editor for a PDF file
    pub fn new(path: &Path) -> Result<RedactionEditor, Error> {
        let doc = super::super::pdfpurr::Document::open(path)
            .map_err(|e| format_err!("Failed to open PDF: {}", e))?;

        let total_pages = doc.page_count();

        Ok(RedactionEditor {
            file_path: path.to_path_buf(),
            regions: Vec::new(),
            current_page: 0,
            total_pages,
            modified: false,
        })
    }

    /// Get the total page count
    pub fn page_count(&self) -> usize {
        self.total_pages
    }

    /// Get the current page number
    pub fn current_page(&self) -> usize {
        self.current_page
    }

    /// Get the file path
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Set the current page
    pub fn set_page(&mut self, page: usize) {
        if page < self.total_pages {
            self.current_page = page;
        }
    }

    /// Add a redaction region
    pub fn add_redaction(&mut self, region: RedactionRegion) {
        self.regions.push(region);
        self.modified = true;
    }

    /// Remove a redaction region by index
    pub fn remove_redaction(&mut self, index: usize) {
        if index < self.regions.len() {
            self.regions.remove(index);
            self.modified = true;
        }
    }

    /// Get redaction regions for a specific page
    pub fn get_regions_for_page(&self, page: usize) -> Vec<&RedactionRegion> {
        self.regions.iter().filter(|r| r.page == page).collect()
    }

    /// Apply all redactions to the PDF
    pub fn apply_redactions(&mut self, output_path: &Path) -> Result<PathBuf, Error> {
        log_info!("Applying {} redaction regions", self.regions.len());

        // Load the PDF document using lopdf
        let mut doc = Document::load(&self.file_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        // Get pages map
        let pages_map = doc.get_pages();
        let page_ids: Vec<_> = pages_map.values().collect();

        // Group redactions by page
        let mut redactions_by_page: std::collections::HashMap<usize, Vec<&RedactionRegion>> =
            std::collections::HashMap::new();
        for region in &self.regions {
            redactions_by_page
                .entry(region.page)
                .or_insert_with(Vec::new)
                .push(region);
        }

        // Apply redactions for each page
        for (page_num, regions) in redactions_by_page {
            let page_index = page_num;
            if page_index >= page_ids.len() {
                continue;
            }

            if let Some(page_id) = page_ids.get(page_index) {
                // Create redaction annotation for each region
                let mut annot_array = Vec::new();
                for region in regions {
                    let mut redact_dict = Dictionary::new();
                    redact_dict.set("Subtype", Object::Name(b"Redact".to_vec()));
                    redact_dict.set("Type", Object::Name(b"Annot".to_vec()));

                    // Set rectangle
                    let rect_array = vec![
                        Object::Real(region.x),
                        Object::Real(region.y),
                        Object::Real(region.x + region.width),
                        Object::Real(region.y + region.height),
                    ];
                    redact_dict.set("Rect", Object::Array(rect_array));

                    // Add annotation to document
                    let annot_id = doc.add_object(Object::Dictionary(redact_dict));
                    annot_array.push(Object::Reference(annot_id));
                }

                // Add annotations to page
                if !annot_array.is_empty() {
                    let page_dict = doc.get_object_mut(**page_id).unwrap().as_dict_mut().unwrap();
                    page_dict.set("Annots", Object::Array(annot_array));
                }
            }
        }

        // Save the redacted document
        let mut buffer = std::io::Cursor::new(Vec::new());
        doc.save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        fs::write(output_path, bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        log_info!("Successfully applied redactions and saved to: {:?}", output_path);
        Ok(output_path.to_path_buf())
    }

    /// Remove all redactions from the PDF
    pub fn remove_redactions(&mut self) -> Result<(), Error> {
        log_info!("Removing all redactions");

        // Load the PDF document using lopdf
        let mut doc = Document::load(&self.file_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        // Get pages map
        let pages_map = doc.get_pages();
        let page_ids: Vec<_> = pages_map.values().collect();

        // Remove redaction annotations from all pages
        for page_id in page_ids {
            let page_dict = doc.get_object_mut(*page_id).unwrap().as_dict_mut().unwrap();

            // Remove Annots key if it exists
            if page_dict.get(b"Annots").is_ok() {
                page_dict.remove(b"Annots");
            }
        }

        // Save the modified document
        let mut buffer = std::io::Cursor::new(Vec::new());
        doc.save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        fs::write(&self.file_path, bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        self.regions.clear();
        self.modified = false;

        log_info!("Successfully removed all redactions");
        Ok(())
    }

    /// Create a backup of the file
    pub fn create_backup(&self, path: &Path) -> Result<PathBuf, Error> {
        let backup_path = path.with_extension(format!(
            "backup.{}",
            path.extension().unwrap_or_default().to_string_lossy()
        ));
        fs::copy(path, &backup_path)?;
        Ok(backup_path)
    }

    fn _check_memory_for_redaction(&self, file_path: &Path) -> Result<(), Error> {
        let file_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0) / (1024 * 1024);

        if file_size > MAX_FILE_SIZE_MB {
            return Err(format_err!(
                "PDF file ({}MB) is too large for redaction. Maximum is {}MB.",
                file_size,
                MAX_FILE_SIZE_MB
            ));
        }

        if self.total_pages > MAX_PAGES_HARD_LIMIT {
            return Err(format_err!(
                "PDF has too many pages ({}). Maximum is {} for redaction.",
                self.total_pages,
                MAX_PAGES_HARD_LIMIT
            ));
        }

        if file_size > WARNING_FILE_SIZE_MB {
            log_warn!(
                "WARNING: Redacting large PDF ({}MB). This may be slow.",
                file_size
            );
        }

        Ok(())
    }
}
