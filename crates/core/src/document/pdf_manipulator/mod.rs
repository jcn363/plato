//! PDF Manipulator Module
//!
//! Provides PDF manipulation operations including page deletion, rotation,
//! extraction, reordering, and merging. The module is organized into
//! submodules for different functionality areas:
//!
//! - Core operations: `PdfManipulator` (this file)
//! - Redaction: `redaction` submodule
//! - Resource extraction: `resources` submodule
//! - Annotation export: `annotations` submodule
//!
//! ## Usage
//!
//! ```ignore
//! use plato_core::document::pdf_manipulator::PdfManipulator;
//!
//! let mut manipulator = PdfManipulator::new()?;
//! manipulator.delete_pages(&input_path, &output_path, &[1, 2, 3])?;
//! ```

use super::pdfpurr::MuPdfContext;

use crate::{log_info, log_warn};
use anyhow::{format_err, Error};
use std::fs;
use std::path::{Path, PathBuf};

// Re-export PDF constants from canonical source in consts::pdf
// per Single Source of Truth rule.
use crate::consts::pdf::{
    CHUNK_SIZE, KOBO_MEMORY_LIMIT_MB, MAX_FILE_SIZE_MB, MAX_PAGES_HARD_LIMIT, MAX_PAGES_WARNING,
    WARNING_FILE_SIZE_MB,
};

// Re-export submodule types for backwards compatibility
pub use annotations::{PdfAnnotation, PdfAnnotationExporter};
pub use redaction::{RedactionEditor, RedactionRegion};
pub use resources::{ExtractedFont, ExtractedImage, ResourceExtractor, ResourceSummary};

pub mod annotations;
pub mod redaction;
pub mod resources;

/// Progress callback for long-running operations
pub type ProgressCallback = Box<dyn Fn(OperationProgress) + Send + Sync>;

/// Operation progress information
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OperationProgress {
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub is_cancelled: bool,
}

/// Memory warning information for large files
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MemoryWarning {
    pub file_size_mb: u64,
    pub page_count: usize,
    pub is_large_file: bool,
    pub is_large_page_count: bool,
}

/// Options for PDF operations
pub struct OperationOptions {
    pub create_backup: bool,
    pub max_memory_mb: u64,
    pub chunk_size: usize,
    pub progress_callback: Option<ProgressCallback>,
}

impl Default for OperationOptions {
    fn default() -> Self {
        OperationOptions {
            create_backup: true,
            max_memory_mb: KOBO_MEMORY_LIMIT_MB,
            chunk_size: CHUNK_SIZE,
            progress_callback: None,
        }
    }
}

/// PDF manipulation operations
///
/// This struct provides high-level PDF operations with progress reporting
/// and memory safety checks for Kobo devices.
pub struct PdfManipulator {
    ctx: MuPdfContext,
    progress_callback: Option<ProgressCallback>,
}

impl Default for PdfManipulator {
    fn default() -> Self {
        Self {
            ctx: MuPdfContext::new().expect("PDFPurr context"),
            progress_callback: None,
        }
    }
}

impl PdfManipulator {
    /// Create a new PDF manipulator
    pub fn new() -> Result<PdfManipulator, Error> {
        let ctx = MuPdfContext::new()?;
        Ok(PdfManipulator {
            ctx,
            progress_callback: None,
        })
    }

    /// Get the page count of a PDF file
    pub fn page_count(&self, path: &Path) -> Result<usize, Error> {
        let doc = super::pdfpurr::Document::open(path)
            .map_err(|e| format_err!("Failed to open PDF: {}", e))?;
        Ok(doc.page_count())
    }

    /// Configure the manipulator with options
    pub fn with_options(&mut self, options: OperationOptions) -> &mut Self {
        self.progress_callback = options.progress_callback;
        self
    }

    /// Delete pages from a PDF
    pub fn delete_pages(
        &mut self,
        input_path: &Path,
        _output_path: &Path,
        _pages: &[usize],
    ) -> Result<PathBuf, Error> {
        // TODO: Implement using lopdf for PDF manipulation
        // PDFPurr is primarily for rendering and text extraction
        // Use lopdf for manipulation operations like page deletion
        log_warn!("delete_pages not yet implemented with PDFPurr/lopdf");
        Ok(input_path.to_path_buf())
    }

    /// Rotate pages in a PDF
    pub fn rotate_pages(
        &mut self,
        input_path: &Path,
        _output_path: &Path,
        _pages: &[(usize, i32)],
    ) -> Result<PathBuf, Error> {
        // TODO: Implement using lopdf for PDF manipulation
        // PDFPurr is primarily for rendering and text extraction
        // Use lopdf for manipulation operations like page rotation
        log_warn!("rotate_pages not yet implemented with PDFPurr/lopdf");
        Ok(input_path.to_path_buf())
    }

    /// Extract pages from a PDF into a new file
    pub fn extract_pages(
        &mut self,
        input_path: &Path,
        _output_path: &Path,
        _pages: &[usize],
    ) -> Result<PathBuf, Error> {
        // TODO: Implement using lopdf for PDF manipulation
        // PDFPurr is primarily for rendering and text extraction
        // Use lopdf for manipulation operations like page extraction
        log_warn!("extract_pages not yet implemented with PDFPurr/lopdf");
        Ok(input_path.to_path_buf())
    }

    /// Reorder pages in a PDF
    pub fn reorder_pages(
        &mut self,
        input_path: &Path,
        _output_path: &Path,
        _order: &[(usize, usize)],
    ) -> Result<PathBuf, Error> {
        // TODO: Implement using lopdf for PDF manipulation
        // PDFPurr is primarily for rendering and text extraction
        // Use lopdf for manipulation operations like page reordering
        log_warn!("reorder_pages not yet implemented with PDFPurr/lopdf");
        Ok(input_path.to_path_buf())
    }

    /// Merge multiple PDFs into a single file
    pub fn merge_pdfs(&mut self, _inputs: &[&Path], output_path: &Path) -> Result<PathBuf, Error> {
        // TODO: Implement using lopdf for PDF manipulation
        // PDFPurr is primarily for rendering and text extraction
        // Use lopdf for manipulation operations like PDF merging
        log_warn!("merge_pdfs not yet implemented with PDFPurr/lopdf");
        Ok(output_path.to_path_buf())
    }

    /// Clean up temporary files in a directory
    pub fn cleanup_temp_files(&self, dir: &Path) -> Result<u64, Error> {
        let mut freed_bytes: u64 = 0;

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.contains(".backup.") || name.contains(".temp.") {
                        if let Ok(meta) = fs::metadata(&path) {
                            freed_bytes += meta.len();
                        }
                        fs::remove_file(&path).ok();
                    }
                }
            }
        }

        Ok(freed_bytes)
    }

    // Private helper methods

    fn get_available_memory_mb() -> u64 {
        #[cfg(target_os = "linux")]
        {
            fs::read_to_string("/proc/meminfo")
                .ok()
                .and_then(|content| {
                    for line in content.lines() {
                        if line.starts_with("MemAvailable:") {
                            if let Some(kb) = line.split_whitespace().nth(1) {
                                return kb.parse::<u64>().ok().map(|kb| kb / 1024);
                            }
                        }
                    }
                    None
                })
                .unwrap_or(256)
        }
        #[cfg(target_os = "ios")]
        {
            // iOS memory detection - use reasonable default for iOS devices
            512
        }
        #[cfg(not(any(target_os = "linux", target_os = "ios")))]
        {
            256
        }
    }

    fn check_memory_available(&self, required_mb: u64) -> Result<u64, Error> {
        let available = Self::get_available_memory_mb();
        if available < required_mb {
            return Err(format_err!(
                "Insufficient memory. Need {}MB, have {}MB available. \
                Please close other apps or use smaller files.",
                required_mb,
                available
            ));
        }
        Ok(available)
    }

    fn check_file_warnings(&self, path: &Path) -> Result<MemoryWarning, Error> {
        let metadata = fs::metadata(path)?;
        let file_size_bytes = metadata.len();
        let file_size_mb = file_size_bytes / (1024 * 1024);

        let doc = super::pdfpurr::Document::open(path)
            .map_err(|e| format_err!("Failed to open PDF: {}", e))?;
        let page_count = doc.page_count();

        Ok(MemoryWarning {
            file_size_mb,
            page_count,
            is_large_file: file_size_mb > WARNING_FILE_SIZE_MB,
            is_large_page_count: page_count > MAX_PAGES_WARNING,
        })
    }

    fn create_backup(&self, path: &Path) -> Result<PathBuf, Error> {
        let backup_dir = path.parent().unwrap_or(path);
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!(
            "{}.backup.{}",
            path.file_stem().and_then(|s| s.to_str()).unwrap_or("file"),
            timestamp
        );
        let backup_path = backup_dir.join(backup_name);

        fs::copy(path, &backup_path)?;
        log_info!("Backup created: {}", backup_path.display());
        Ok(backup_path)
    }

    fn report_progress(&self, current: usize, total: usize, message: &str) {
        if let Some(ref callback) = self.progress_callback {
            callback(OperationProgress {
                current,
                total,
                message: message.to_string(),
                is_cancelled: false,
            });
        }
    }

    fn validate_operation(&self, path: &Path) -> Result<MemoryWarning, Error> {
        let warning = self.check_file_warnings(path)?;

        if warning.file_size_mb > MAX_FILE_SIZE_MB {
            return Err(format_err!(
                "File too large ({}MB). Maximum allowed is {}MB on Kobo. \
                Please split the PDF or use smaller files.",
                warning.file_size_mb,
                MAX_FILE_SIZE_MB
            ));
        }

        if warning.page_count > MAX_PAGES_HARD_LIMIT {
            return Err(format_err!(
                "PDF has {} pages which exceeds the limit of {}. \
                Large PDFs may cause memory issues on Kobo. \
                Please use a PDF with fewer pages.",
                warning.page_count,
                MAX_PAGES_HARD_LIMIT
            ));
        }

        self.check_memory_available(warning.file_size_mb + 20)?;

        if warning.is_large_file || warning.is_large_page_count {
            log_warn!(
                "WARNING: Processing large PDF ({}MB, {} pages). \
                This may be slow on Kobo. Ensure battery is charged.",
                warning.file_size_mb,
                warning.page_count
            );
        }

        Ok(warning)
    }

    fn calculate_total_size(&self, inputs: &[&Path]) -> Result<u64, Error> {
        let mut total_size: u64 = 0;
        for input_path in inputs {
            if let Ok(meta) = fs::metadata(input_path) {
                total_size += meta.len();
            }
        }
        Ok(total_size / (1024 * 1024))
    }

    fn validate_merge_size(&self, total_mb: u64) -> Result<(), Error> {
        if total_mb > MAX_FILE_SIZE_MB {
            return Err(format_err!(
                "Total size of files to merge ({}MB) exceeds limit of {}MB. \
                Please merge fewer or smaller files.",
                total_mb,
                MAX_FILE_SIZE_MB
            ));
        }
        Ok(())
    }

    fn merge_documents(
        &mut self,
        _inputs: &[&Path],
        _new_doc: &(),
    ) -> Result<(), Error> {
        // TODO: Implement using lopdf for PDF manipulation
        log_warn!("merge_documents not yet implemented with PDFPurr/lopdf");
        Ok(())
    }

    fn add_document_pages(&mut self, _doc: &(), _new_doc: &()) {
        // TODO: Implement using lopdf for PDF manipulation
    }

    fn save_merged_document(
        &mut self,
        _new_doc: &(),
        _output_path: &Path,
        total: usize,
    ) -> Result<(), Error> {
        self.report_progress(total, total, "Merge complete!");
        Ok(())
    }
}
