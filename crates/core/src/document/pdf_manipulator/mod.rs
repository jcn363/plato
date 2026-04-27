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
pub use annotations::{
    AnnotationQuery, AnnotationSubtype, PdfAnnotation, PdfAnnotationExporter, PdfAnnotationManager,
    XfdfHandler,
};
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
#[derive(Default)]
pub struct PdfManipulator {
    progress_callback: Option<ProgressCallback>,
}

impl PdfManipulator {
    /// Create a new PDF manipulator
    pub fn new() -> Result<PdfManipulator, Error> {
        Ok(PdfManipulator {
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
        output_path: &Path,
        pages: &[usize],
    ) -> Result<PathBuf, Error> {
        use lopdf::Document;
        use std::fs::File;
        use std::io::Cursor;
        use std::io::Write;

        log_info!("Deleting pages from PDF: {:?}", pages);

        // Load the PDF document using lopdf
        let mut doc = Document::load(input_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        // Get pages map
        let pages_map = doc.get_pages();
        let page_ids: Vec<_> = pages_map.values().collect();

        // Convert pages to 0-indexed and sort in descending order to avoid index shifting
        let mut pages_to_delete: Vec<usize> = pages.iter().map(|p| p - 1).collect();
        pages_to_delete.sort();
        pages_to_delete.reverse();

        // Delete pages by removing from pages dictionary
        for page_num in pages_to_delete {
            if page_num < page_ids.len() {
                doc.delete_pages(&[page_num as u32]);
            }
        }

        // Save the modified document to bytes
        let mut buffer = Cursor::new(Vec::new());
        doc.save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        let mut file = File::create(output_path)
            .map_err(|e| format_err!("Failed to create output file: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        log_info!("Successfully deleted pages and saved to: {:?}", output_path);
        Ok(output_path.to_path_buf())
    }

    /// Rotate pages in a PDF
    pub fn rotate_pages(
        &mut self,
        input_path: &Path,
        output_path: &Path,
        pages: &[usize],
        degrees: i32,
    ) -> Result<PathBuf, Error> {
        use lopdf::{Document, Object};
        use std::fs::File;
        use std::io::Cursor;
        use std::io::Write;

        log_info!("Rotating pages by {} degrees: {:?}", degrees, pages);

        // Normalize degrees to 0-360 range
        let rotation = degrees.rem_euclid(360);

        // Load the PDF document using lopdf
        let mut doc = Document::load(input_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        // Get pages map
        let pages_map = doc.get_pages();
        let page_ids: Vec<_> = pages_map.values().collect();

        // Rotate specified pages
        for page_num in pages {
            let page_index = page_num - 1; // Convert to 0-indexed
            if page_index < page_ids.len() {
                if let Some(page_id) = page_ids.get(page_index) {
                    let page_dict = doc
                        .get_object_mut(**page_id)
                        .map_err(|e| format_err!("Failed to get page object: {}", e))?
                        .as_dict_mut()
                        .map_err(|e| format_err!("Page object is not a dictionary: {}", e))?;

                    // Get current rotation or default to 0
                    let current_rotation = page_dict
                        .get(b"Rotate")
                        .and_then(|obj| obj.as_i64())
                        .unwrap_or(0) as i32;

                    // Set new rotation
                    let new_rotation = (current_rotation + rotation) % 360;
                    page_dict.set("Rotate", Object::Integer(new_rotation as i64));
                }
            }
        }

        // Save the modified document to bytes
        let mut buffer = Cursor::new(Vec::new());
        doc.save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        let mut file = File::create(output_path)
            .map_err(|e| format_err!("Failed to create output file: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        log_info!("Successfully rotated pages and saved to: {:?}", output_path);
        Ok(output_path.to_path_buf())
    }

    /// Extract pages from a PDF into a new file
    pub fn extract_pages(
        &mut self,
        input_path: &Path,
        output_path: &Path,
        pages: &[usize],
    ) -> Result<PathBuf, Error> {
        use lopdf::Document;
        use std::fs::File;
        use std::io::Cursor;
        use std::io::Write;

        log_info!("Extracting pages from PDF: {:?}", pages);

        // Load the PDF document using lopdf
        let doc = Document::load(input_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        // Get pages map
        let pages_map = doc.get_pages();
        let page_ids: Vec<_> = pages_map.values().collect();

        // Create a new document with only the specified pages
        let mut new_doc = Document::with_version("1.4");

        // Convert pages to 0-indexed
        let pages_to_extract: Vec<usize> = pages.iter().map(|p| p - 1).collect();

        // Copy pages to new document
        for page_num in pages_to_extract {
            if page_num < page_ids.len() {
                if let Some(page_id) = page_ids.get(page_num) {
                    let page_object = doc
                        .get_object(**page_id)
                        .map_err(|e| format_err!("Failed to get page object: {}", e))?;
                    new_doc.add_object(page_object.clone());
                }
            }
        }

        // Save the new document to bytes
        let mut buffer = Cursor::new(Vec::new());
        new_doc
            .save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        let mut file = File::create(output_path)
            .map_err(|e| format_err!("Failed to create output file: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        log_info!(
            "Successfully extracted pages and saved to: {:?}",
            output_path
        );
        Ok(output_path.to_path_buf())
    }

    /// Reorder pages in a PDF
    pub fn reorder_pages(
        &mut self,
        input_path: &Path,
        output_path: &Path,
        order: &[(usize, usize)],
    ) -> Result<PathBuf, Error> {
        use lopdf::Document;
        use std::fs::File;
        use std::io::Cursor;
        use std::io::Write;

        log_info!("Reordering pages in PDF: {:?}", order);

        // Load the PDF document using lopdf
        let doc = Document::load(input_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        // Get pages map
        let pages_map = doc.get_pages();
        let page_ids: Vec<_> = pages_map.values().collect();

        // Collect all pages in the desired order
        let mut pages_in_order = Vec::new();
        for (from, _to) in order {
            let from_index = from - 1; // Convert to 0-indexed
            if from_index < page_ids.len() {
                if let Some(page_id) = page_ids.get(from_index) {
                    let page_object = doc
                        .get_object(**page_id)
                        .map_err(|e| format_err!("Failed to get page object: {}", e))?;
                    pages_in_order.push(page_object.clone());
                }
            }
        }

        // Create a new document with reordered pages
        let mut new_doc = Document::with_version("1.4");
        for page_object in pages_in_order {
            new_doc.add_object(page_object);
        }

        // Save the new document to bytes
        let mut buffer = Cursor::new(Vec::new());
        new_doc
            .save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        let mut file = File::create(output_path)
            .map_err(|e| format_err!("Failed to create output file: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        log_info!(
            "Successfully reordered pages and saved to: {:?}",
            output_path
        );
        Ok(output_path.to_path_buf())
    }

    /// Reorder pages for booklet printing (2-up imposition)
    /// Pages are arranged so that when printed double-sided and folded, they form a booklet
    pub fn reorder_pages_for_booklet(
        &mut self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<PathBuf, Error> {
        use lopdf::Document;
        use std::fs::File;
        use std::io::Cursor;
        use std::io::Write;

        log_info!("Reordering pages for booklet printing");

        // Load the PDF document using lopdf
        let doc = Document::load(input_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        // Get pages map
        let pages_map = doc.get_pages();
        let page_ids: Vec<_> = pages_map.values().collect();
        let total_pages = page_ids.len();

        // Calculate booklet imposition order
        let booklet_order = calculate_booklet_order(total_pages);

        // Collect pages in booklet order
        let mut pages_in_order = Vec::new();
        for (from, _to) in &booklet_order {
            let from_index = from - 1; // Convert to 0-indexed
            if from_index < page_ids.len() {
                if let Some(page_id) = page_ids.get(from_index) {
                    let page_object = doc
                        .get_object(**page_id)
                        .map_err(|e| format_err!("Failed to get page object: {}", e))?;
                    pages_in_order.push(page_object.clone());
                }
            }
        }

        // Create a new document with reordered pages
        let mut new_doc = Document::with_version("1.4");
        for page_object in pages_in_order {
            new_doc.add_object(page_object);
        }

        // Save the new document to bytes
        let mut buffer = Cursor::new(Vec::new());
        new_doc
            .save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        let mut file = File::create(output_path)
            .map_err(|e| format_err!("Failed to create output file: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        log_info!(
            "Successfully reordered pages for booklet and saved to: {:?}",
            output_path
        );
        Ok(output_path.to_path_buf())
    }
}

/// Calculate the page order for booklet printing (2-up imposition)
/// Returns a vector of (from_page, to_page) tuples
fn calculate_booklet_order(total_pages: usize) -> Vec<(usize, usize)> {
    let mut order = Vec::new();

    // Pad to multiple of 4 (signature size)
    let padded_pages = ((total_pages + 3) / 4) * 4;

    // Calculate booklet imposition
    // For each signature (4 pages), the order is:
    // Front side: last page with first page
    // Back side: second page with second-to-last page
    for i in (0..padded_pages).step_by(4) {
        let page1 = i + 1; // First page of signature
        let page2 = i + 2; // Second page of signature
        let page3 = padded_pages - i; // Last page of signature
        let page4 = padded_pages - i - 1; // Second-to-last page of signature

        // Front of sheet: page3 with page1
        if page3 <= total_pages && page1 <= total_pages {
            order.push((page3, page1));
        } else if page1 <= total_pages {
            order.push((page1, page1)); // Blank page for page3
        }

        // Back of sheet: page2 with page4
        if page2 <= total_pages && page4 <= total_pages {
            order.push((page2, page4));
        } else if page2 <= total_pages {
            order.push((page2, page2)); // Blank page for page4
        }
    }

    order
}

impl PdfManipulator {
    /// Merge multiple PDFs into a single file
    pub fn merge_pdfs(&mut self, inputs: &[&Path], output_path: &Path) -> Result<PathBuf, Error> {
        use lopdf::Document;
        use std::fs::File;
        use std::io::Cursor;
        use std::io::Write;

        log_info!("Merging PDFs: {:?}", inputs);

        // Create a new document for the merged result
        let mut merged_doc = Document::with_version("1.4");

        // Load and merge each input PDF
        for input_path in inputs {
            let doc = Document::load(input_path).map_err(|e| {
                format_err!("Failed to load PDF {:?} with lopdf: {}", input_path, e)
            })?;

            // Get pages map
            let pages_map = doc.get_pages();
            let page_ids: Vec<_> = pages_map.values().collect();

            // Copy all pages from the input document to the merged document
            for page_id in page_ids {
                let page_object = doc
                    .get_object(*page_id)
                    .map_err(|e| format_err!("Failed to get page object: {}", e))?;
                merged_doc.add_object(page_object.clone());
            }
        }

        // Save the merged document to bytes
        let mut buffer = Cursor::new(Vec::new());
        merged_doc
            .save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save merged PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        let mut file = File::create(output_path)
            .map_err(|e| format_err!("Failed to create output file: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        log_info!("Successfully merged PDFs and saved to: {:?}", output_path);
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

    fn _get_available_memory_mb() -> u64 {
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

    fn _check_memory_available(&self, required_mb: u64) -> Result<u64, Error> {
        let available = Self::_get_available_memory_mb();
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

    fn _check_file_warnings(&self, path: &Path) -> Result<MemoryWarning, Error> {
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

    fn _create_backup(&self, path: &Path) -> Result<PathBuf, Error> {
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

    fn _report_progress(&self, current: usize, total: usize, message: &str) {
        if let Some(ref callback) = self.progress_callback {
            callback(OperationProgress {
                current,
                total,
                message: message.to_string(),
                is_cancelled: false,
            });
        }
    }

    fn _validate_operation(&self, path: &Path) -> Result<MemoryWarning, Error> {
        let warning = self._check_file_warnings(path)?;

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

        self._check_memory_available(warning.file_size_mb + 20)?;

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

    fn _calculate_total_size(&self, inputs: &[&Path]) -> Result<u64, Error> {
        let mut total_size: u64 = 0;
        for input_path in inputs {
            if let Ok(meta) = fs::metadata(input_path) {
                total_size += meta.len();
            }
        }
        Ok(total_size / (1024 * 1024))
    }

    fn _validate_merge_size(&self, total_mb: u64) -> Result<(), Error> {
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
}
