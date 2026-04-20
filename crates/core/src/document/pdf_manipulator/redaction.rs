//! PDF Redaction Module
//!
//! Provides functionality for redacting sensitive content from PDF documents.
//! Supports defining redaction regions and applying them to permanently remove
//! content from PDF files.

use super::mupdf;
use crate::consts::pdf::{MAX_FILE_SIZE_MB, MAX_PAGES_HARD_LIMIT, WARNING_FILE_SIZE_MB};
use crate::log_warn;
use anyhow::{format_err, Error};
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
    doc: mupdf::Document,
    file_path: PathBuf,
    regions: Vec<RedactionRegion>,
    current_page: usize,
    total_pages: usize,
    modified: bool,
}

impl RedactionEditor {
    /// Create a new redaction editor for a PDF file
    pub fn new(path: &Path) -> Result<RedactionEditor, Error> {
        let ctx = mupdf::MuPdfContext::new()?;

        let doc = ctx
            .open_document(path)
            .ok_or_else(|| format_err!("Failed to open PDF: {}", path.display()))?;

        let total_pages = doc.page_count() as usize;

        Ok(RedactionEditor {
            doc,
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
        if self.regions.is_empty() {
            return Err(format_err!("No redaction regions defined"));
        }

        self.check_memory_for_redaction(&self.file_path)?;

        let page = self
            .doc
            .load_page(self.current_page as i32)
            .map_err(|_| format_err!("Failed to load page for redaction"))?;

        page.apply_redactions(0);

        let opts = mupdf::FzWriteOptions::default();
        self.doc.save(output_path, &opts, "pdf");

        self.modified = false;
        self.regions.clear();
        Ok(output_path.to_path_buf())
    }

    /// Remove all redactions from the PDF
    pub fn remove_redactions(&mut self) -> Result<(), Error> {
        let page = self
            .doc
            .load_page(self.current_page as i32)
            .map_err(|_| format_err!("Failed to load page"))?;

        page.remove_redactions();
        self.regions.clear();
        self.modified = false;
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

    fn check_memory_for_redaction(&self, file_path: &Path) -> Result<(), Error> {
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
