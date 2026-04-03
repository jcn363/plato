use crate::document::mupdf_sys::*;
use crate::{log_info, log_warn};
use anyhow::{format_err, Error};
use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};
use std::ptr;

const MAX_FILE_SIZE_MB: u64 = 50;
const WARNING_FILE_SIZE_MB: u64 = 30;
const MAX_PAGES_WARNING: usize = 300;
const MAX_PAGES_HARD_LIMIT: usize = 500;
const CHUNK_SIZE: usize = 10;
const KOBO_MEMORY_LIMIT_MB: u64 = 256;

pub struct PdfManipulator {
    ctx: *mut FzContext,
    progress_callback: Option<ProgressCallback>,
}

impl Default for PdfManipulator {
    fn default() -> Self {
        Self {
            ctx: ptr::null_mut(),
            progress_callback: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryWarning {
    pub file_size_mb: u64,
    pub page_count: usize,
    pub is_large_file: bool,
    pub is_large_page_count: bool,
}

#[derive(Debug, Clone)]
pub struct OperationProgress {
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub is_cancelled: bool,
}

pub type ProgressCallback = Box<dyn Fn(OperationProgress) + Send + Sync>;

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

impl PdfManipulator {
    pub fn new() -> Result<PdfManipulator, Error> {
        let ctx_ptr =
            new_mupdf_context().ok_or_else(|| format_err!("Failed to create MuPDF context"))?;
        Ok(PdfManipulator {
            ctx: ctx_ptr,
            progress_callback: None,
        })
    }

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
        #[cfg(not(target_os = "linux"))]
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

        let path_cstr = CString::new(path.to_string_lossy().as_bytes())?;
        // SAFETY: FFI call to MuPDF. Context pointer is valid and CString is null-terminated.
        let doc = unsafe { mp_open_document(self.ctx, path_cstr.as_ptr()) };

        let page_count = if !doc.is_null() {
            // SAFETY: FFI call to MuPDF. Context and document pointers are valid and initialized.
            let count = unsafe { fz_pdf_count_pages(self.ctx, doc) };
            // SAFETY: Cleaning up MuPDF document resource. Context and document pointers are valid.
            unsafe { fz_drop_document(self.ctx, doc) };
            count as usize
        } else {
            0
        };

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

    pub fn with_options(&mut self, options: OperationOptions) -> &mut Self {
        self.progress_callback = options.progress_callback;
        self
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

    pub fn delete_pages(
        &mut self,
        input_path: &Path,
        output_path: &Path,
        pages: &[usize],
    ) -> Result<PathBuf, Error> {
        if input_path.exists() {
            self.create_backup(input_path)?;
        }

        self.validate_operation(input_path)?;

        // SAFETY: FFI calls to MuPDF for document manipulation. Context pointer is valid.
        // CString values are properly null-terminated. Document is opened, manipulated, and dropped within this scope.
        unsafe {
            let path_cstr = CString::new(input_path.to_string_lossy().as_bytes())?;
            let doc = mp_open_document(self.ctx, path_cstr.as_ptr());

            if doc.is_null() {
                return Err(format_err!("Failed to open PDF"));
            }

            let _total_pages = fz_pdf_count_pages(self.ctx, doc) as usize;
            let total = pages.len();

            for (i, &page_num) in pages.iter().rev().enumerate() {
                self.report_progress(i + 1, total, "Deleting pages...");
                if (page_num as libc::c_int) < fz_pdf_count_pages(self.ctx, doc) {
                    fz_pdf_delete_page(self.ctx, doc, page_num as libc::c_int);
                }
            }

            let out_cstr = CString::new(output_path.to_string_lossy().as_bytes())?;
            let opts = FzWriteOptions::default();
            let format = CString::new("pdf")?;

            self.report_progress(total, total, "Saving PDF...");
            fz_save_document(self.ctx, doc, out_cstr.as_ptr(), &opts, format.as_ptr());
            fz_drop_document(self.ctx, doc);

            self.report_progress(total, total, "Operation complete!");
            Ok(output_path.to_path_buf())
        }
    }

    pub fn rotate_pages(
        &mut self,
        input_path: &Path,
        output_path: &Path,
        pages: &[(usize, i32)],
    ) -> Result<PathBuf, Error> {
        if input_path.exists() {
            self.create_backup(input_path)?;
        }

        self.validate_operation(input_path)?;

        // SAFETY: FFI calls to MuPDF for page rotation. Context pointer is valid.
        // CString values are properly null-terminated. Document is opened and dropped within this scope.
        unsafe {
            let path_cstr = CString::new(input_path.to_string_lossy().as_bytes())?;
            let doc = mp_open_document(self.ctx, path_cstr.as_ptr());

            if doc.is_null() {
                return Err(format_err!("Failed to open PDF"));
            }

            let total = pages.len();

            for (i, &(page_num, degrees)) in pages.iter().enumerate() {
                self.report_progress(i + 1, total, "Rotating pages...");
                fz_pdf_rotate_page(self.ctx, doc, page_num as libc::c_int, degrees);
            }

            let out_cstr = CString::new(output_path.to_string_lossy().as_bytes())?;
            let opts = FzWriteOptions::default();
            let format = CString::new("pdf")?;

            self.report_progress(total, total, "Saving PDF...");
            fz_save_document(self.ctx, doc, out_cstr.as_ptr(), &opts, format.as_ptr());
            fz_drop_document(self.ctx, doc);

            self.report_progress(total, total, "Operation complete!");
            Ok(output_path.to_path_buf())
        }
    }

    pub fn extract_pages(
        &mut self,
        input_path: &Path,
        output_path: &Path,
        pages: &[usize],
    ) -> Result<PathBuf, Error> {
        self.validate_operation(input_path)?;

        let estimated_size = {
            let meta = fs::metadata(input_path)?;
            (meta.len() / (pages.len() as u64 + 1)) / (1024 * 1024)
        };

        self.check_memory_available(estimated_size + 10)?;

        // SAFETY: FFI calls to MuPDF for page extraction. Context pointer is valid.
        // CString values are null-terminated. Documents are properly created and dropped within this scope.
        unsafe {
            let path_cstr = CString::new(input_path.to_string_lossy().as_bytes())?;
            let doc = mp_open_document(self.ctx, path_cstr.as_ptr());

            if doc.is_null() {
                return Err(format_err!("Failed to open PDF"));
            }

            let new_doc = fz_new_pdf_document(self.ctx);
            if new_doc.is_null() {
                fz_drop_document(self.ctx, doc);
                return Err(format_err!("Failed to create new PDF"));
            }

            let total_pages = fz_pdf_count_pages(self.ctx, doc);
            let total = pages.len();

            for (i, &page_num) in pages.iter().enumerate() {
                self.report_progress(i + 1, total, "Extracting pages...");

                if page_num < total_pages as usize {
                    let page = fz_load_page(self.ctx, doc, page_num as libc::c_int);
                    if !page.is_null() {
                        fz_pdf_insert_page(self.ctx, new_doc, page, -1);
                        fz_drop_page(self.ctx, page);
                    }
                }
            }

            let out_cstr = CString::new(output_path.to_string_lossy().as_bytes())?;
            let opts = FzWriteOptions::default();
            let format = CString::new("pdf")?;

            self.report_progress(total, total, "Saving extracted pages...");
            fz_save_document(self.ctx, new_doc, out_cstr.as_ptr(), &opts, format.as_ptr());
            fz_drop_document(self.ctx, new_doc);
            fz_drop_document(self.ctx, doc);

            self.report_progress(total, total, "Operation complete!");
            Ok(output_path.to_path_buf())
        }
    }

    pub fn reorder_pages(
        &mut self,
        input_path: &Path,
        output_path: &Path,
        order: &[(usize, usize)],
    ) -> Result<PathBuf, Error> {
        if input_path.exists() {
            self.create_backup(input_path)?;
        }

        self.validate_operation(input_path)?;

        // SAFETY: FFI calls to MuPDF for page reordering. Context pointer is valid.
        // CString values are null-terminated. Document is opened and dropped within this scope.
        unsafe {
            let path_cstr = CString::new(input_path.to_string_lossy().as_bytes())?;
            let doc = mp_open_document(self.ctx, path_cstr.as_ptr());

            if doc.is_null() {
                return Err(format_err!("Failed to open PDF"));
            }

            if fz_pdf_can_move_pages(self.ctx, doc) == 0 {
                fz_drop_document(self.ctx, doc);
                return Err(format_err!("This PDF doesn't support page moving"));
            }

            let total = order.len();

            for (i, &(src, dst)) in order.iter().enumerate() {
                self.report_progress(i + 1, total, "Reordering pages...");
                fz_pdf_move_page(self.ctx, doc, src as libc::c_int, dst as libc::c_int);
            }

            let out_cstr = CString::new(output_path.to_string_lossy().as_bytes())?;
            let opts = FzWriteOptions::default();
            let format = CString::new("pdf")?;

            self.report_progress(total, total, "Saving PDF...");
            fz_save_document(self.ctx, doc, out_cstr.as_ptr(), &opts, format.as_ptr());
            fz_drop_document(self.ctx, doc);

            self.report_progress(total, total, "Operation complete!");
            Ok(output_path.to_path_buf())
        }
    }

    pub fn merge_pdfs(&mut self, inputs: &[&Path], output_path: &Path) -> Result<PathBuf, Error> {
        let mut total_size: u64 = 0;
        for input_path in inputs {
            if let Ok(meta) = fs::metadata(input_path) {
                total_size += meta.len();
            }
        }
        let total_mb = total_size / (1024 * 1024);

        if total_mb > MAX_FILE_SIZE_MB {
            return Err(format_err!(
                "Total size of files to merge ({}MB) exceeds limit of {}MB. \
                Please merge fewer or smaller files.",
                total_mb,
                MAX_FILE_SIZE_MB
            ));
        }

        self.check_memory_available(total_mb + 30)?;

        if total_mb > WARNING_FILE_SIZE_MB {
            log_warn!(
                "WARNING: Merging {}MB of PDFs. This may be slow. \
                Ensure device is charged and not low on battery.",
                total_mb
            );
        }

        // SAFETY: FFI calls to MuPDF for merging PDFs. Context pointer is valid.
        // CString values are null-terminated. Documents are opened and dropped within this scope.
        unsafe {
            let new_doc = fz_new_pdf_document(self.ctx);
            if new_doc.is_null() {
                return Err(format_err!("Failed to create new PDF"));
            }

            let total_inputs = inputs.len();

            for (file_idx, input_path) in inputs.iter().enumerate() {
                self.report_progress(
                    file_idx + 1,
                    total_inputs,
                    &format!("Processing file {}/{}...", file_idx + 1, total_inputs),
                );

                let path_cstr = CString::new(input_path.to_string_lossy().as_bytes())?;
                let doc = mp_open_document(self.ctx, path_cstr.as_ptr());

                if doc.is_null() {
                    continue;
                }

                let file_pages = fz_pdf_count_pages(self.ctx, doc);

                for page_idx in 0..file_pages {
                    self.report_progress(
                        page_idx as usize + 1,
                        file_pages as usize,
                        "Adding pages...",
                    );

                    let page = fz_load_page(self.ctx, doc, page_idx);
                    if !page.is_null() {
                        fz_pdf_insert_page(self.ctx, new_doc, page, -1);
                        fz_drop_page(self.ctx, page);
                    }
                }

                fz_drop_document(self.ctx, doc);
            }

            let out_cstr = CString::new(output_path.to_string_lossy().as_bytes())?;
            let opts = FzWriteOptions::default();
            let format = CString::new("pdf")?;

            self.report_progress(total_inputs, total_inputs, "Saving merged PDF...");
            fz_save_document(self.ctx, new_doc, out_cstr.as_ptr(), &opts, format.as_ptr());
            fz_drop_document(self.ctx, new_doc);

            self.report_progress(total_inputs, total_inputs, "Merge complete!");
            Ok(output_path.to_path_buf())
        }
    }

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
}

#[derive(Debug, Clone)]
pub struct RedactionRegion {
    pub page: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct RedactionEditor {
    ctx: *mut FzContext,
    doc: *mut FzDocument,
    file_path: PathBuf,
    regions: Vec<RedactionRegion>,
    current_page: usize,
    total_pages: usize,
    modified: bool,
}

impl RedactionEditor {
    pub fn new(path: &Path) -> Result<RedactionEditor, Error> {
        let ctx =
            new_mupdf_context().ok_or_else(|| format_err!("Failed to create MuPDF context"))?;

        // SAFETY: FFI calls to MuPDF to open document and count pages. Context is newly created and valid.
        // CString is null-terminated. Document pointer is valid for the lifetime of this struct.
        unsafe {
            let path_cstr = CString::new(path.to_string_lossy().as_bytes())?;
            let doc = mp_open_document(ctx, path_cstr.as_ptr());

            if doc.is_null() {
                fz_drop_context(ctx);
                return Err(format_err!("Failed to open PDF: {}", path.display()));
            }

            let total_pages = mp_count_pages(ctx, doc) as usize;

            Ok(RedactionEditor {
                ctx,
                doc,
                file_path: path.to_path_buf(),
                regions: Vec::new(),
                current_page: 0,
                total_pages,
                modified: false,
            })
        }
    }

    pub fn page_count(&self) -> usize {
        self.total_pages
    }

    pub fn current_page(&self) -> usize {
        self.current_page
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn set_page(&mut self, page: usize) {
        if page < self.total_pages {
            self.current_page = page;
        }
    }

    pub fn add_redaction(&mut self, region: RedactionRegion) {
        self.regions.push(region);
        self.modified = true;
    }

    pub fn remove_redaction(&mut self, index: usize) {
        if index < self.regions.len() {
            self.regions.remove(index);
            self.modified = true;
        }
    }

    pub fn get_regions_for_page(&self, page: usize) -> Vec<&RedactionRegion> {
        self.regions.iter().filter(|r| r.page == page).collect()
    }

    pub fn apply_redactions(&mut self, output_path: &Path) -> Result<PathBuf, Error> {
        if self.regions.is_empty() {
            return Err(format_err!("No redaction regions defined"));
        }

        self.check_memory_for_redaction(&self.file_path)?;

        // SAFETY: FFI calls to MuPDF to apply redactions and save document. Context and document pointers are valid.
        // CString values are null-terminated. Document has been opened in RedactionEditor::new.
        unsafe {
            let page = fz_load_page(self.ctx, self.doc, self.current_page as libc::c_int);
            if page.is_null() {
                return Err(format_err!("Failed to load page for redaction"));
            }
            fz_apply_redactions(self.ctx, page, 0);
            fz_drop_page(self.ctx, page);

            let out_cstr = CString::new(output_path.to_string_lossy().as_bytes())?;
            let opts = FzWriteOptions::default();
            let format = CString::new("pdf")?;
            fz_save_document(
                self.ctx,
                self.doc,
                out_cstr.as_ptr(),
                &opts,
                format.as_ptr(),
            );

            self.modified = false;
            self.regions.clear();
            Ok(output_path.to_path_buf())
        }
    }

    pub fn remove_redactions(&mut self) -> Result<(), Error> {
        // SAFETY: FFI call to MuPDF to remove redactions. Context and document pointers are valid.
        unsafe {
            let page = fz_load_page(self.ctx, self.doc, self.current_page as libc::c_int);
            if page.is_null() {
                return Err(format_err!("Failed to load page"));
            }
            fz_remove_redactions(self.ctx, page);
            fz_drop_page(self.ctx, page);
            self.regions.clear();
            self.modified = false;
            Ok(())
        }
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

    pub fn create_backup(&self, path: &Path) -> Result<PathBuf, Error> {
        let backup_path = path.with_extension(format!(
            "backup.{}",
            path.extension().unwrap_or_default().to_string_lossy()
        ));
        fs::copy(path, &backup_path)?;
        Ok(backup_path)
    }
}

impl Drop for RedactionEditor {
    fn drop(&mut self) {
        // SAFETY: Cleaning up MuPDF resources. Pointers are checked for null before dropping.
        unsafe {
            if !self.doc.is_null() {
                fz_drop_document(self.ctx, self.doc);
            }
            if !self.ctx.is_null() {
                fz_drop_context(self.ctx);
            }
        }
    }
}

impl Drop for PdfManipulator {
    fn drop(&mut self) {
        // SAFETY: Cleaning up MuPDF context. Pointer is checked for null before dropping.
        unsafe {
            if !self.ctx.is_null() {
                fz_drop_context(self.ctx);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExtractedImage {
    pub page: usize,
    pub index: usize,
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ExtractedFont {
    pub name: String,
    pub data: Vec<u8>,
}

pub struct ResourceExtractor {
    ctx: *mut FzContext,
    doc: *mut FzDocument,
    file_path: PathBuf,
    total_pages: usize,
}

impl ResourceExtractor {
    pub fn new(path: &Path) -> Result<ResourceExtractor, Error> {
        let ctx =
            new_mupdf_context().ok_or_else(|| format_err!("Failed to create MuPDF context"))?;

        // SAFETY: FFI calls to MuPDF to open document and count pages. Context is newly created and valid.
        // CString is null-terminated. Document pointer is valid for the lifetime of this struct.
        unsafe {
            let path_cstr = CString::new(path.to_string_lossy().as_bytes())?;
            let doc = mp_open_document(ctx, path_cstr.as_ptr());

            if doc.is_null() {
                fz_drop_context(ctx);
                return Err(format_err!("Failed to open PDF: {}", path.display()));
            }

            let total_pages = mp_count_pages(ctx, doc) as usize;

            Ok(ResourceExtractor {
                ctx,
                doc,
                file_path: path.to_path_buf(),
                total_pages,
            })
        }
    }

    pub fn page_count(&self) -> usize {
        self.total_pages
    }

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

        let mut images = Vec::new();

        // SAFETY: FFI calls to MuPDF to load page and extract images. Context, document, and page pointers are valid.
        // Images and pages are properly dropped within this scope.
        unsafe {
            let page = fz_load_page(self.ctx, self.doc, page_num as i32);
            if page.is_null() {
                return Err(format_err!("Failed to load page {}", page_num + 1));
            }

            let image_count = fz_count_page_images(self.ctx, page);

            for i in 0..image_count {
                let image = fz_load_page_image(self.ctx, page, i);
                if !image.is_null() {
                    let width = fz_image_width(self.ctx, image);
                    let height = fz_image_height(self.ctx, image);

                    images.push(ExtractedImage {
                        page: page_num,
                        index: i as usize,
                        width,
                        height,
                        data: Vec::new(),
                    });

                    fz_drop_image(self.ctx, image);
                }
            }

            fz_drop_page(self.ctx, page);
        }

        Ok(images)
    }

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

    pub fn count_page_fonts(&self, page_num: usize) -> Result<usize, Error> {
        if page_num >= self.total_pages {
            return Err(format_err!("Page {} does not exist", page_num + 1));
        }

        // SAFETY: FFI calls to MuPDF to load page and count fonts. Context and document pointers are valid.
        // Page is properly dropped after counting.
        unsafe {
            let page = fz_load_page(self.ctx, self.doc, page_num as i32);
            if page.is_null() {
                return Err(format_err!("Failed to load page {}", page_num + 1));
            }

            let count = fz_count_page_fonts(self.ctx, page) as usize;
            fz_drop_page(self.ctx, page);
            Ok(count)
        }
    }

    pub fn extract_text_from_page(&self, page_num: usize) -> Result<String, Error> {
        if page_num >= self.total_pages {
            return Err(format_err!("Page {} does not exist", page_num + 1));
        }

        Ok(format!(
            "Text extraction for page {} - use Plato's built-in text selection",
            page_num + 1
        ))
    }

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

    pub fn is_pdf_a(&self) -> bool {
        !self.pdf_a_version().is_empty()
    }

    pub fn pdf_a_version(&self) -> String {
        // SAFETY: FFI call to MuPDF to get PDF/A output intent. Context and document pointers are valid.
        // Result pointer is checked for null before dereferencing.
        unsafe {
            let output_id = fz_pdf_output_intent(self.ctx, self.doc);
            if !output_id.is_null() {
                let cstr = std::ffi::CStr::from_ptr(output_id);
                let id = cstr.to_string_lossy().to_string();
                if id.contains("PDF/A") {
                    return id;
                }
            }
        }
        String::new()
    }

    pub fn read_annotations(&self) -> Result<Vec<PdfAnnotation>, Error> {
        let mut annotations = Vec::new();

        for page_num in 0..self.total_pages {
            // SAFETY: FFI calls to MuPDF to iterate annotations. Context, document, and page pointers are valid.
            // Annotation and page pointers are valid during iteration and properly cleaned up.
            unsafe {
                let page = fz_load_page(self.ctx, self.doc, page_num as i32);
                if page.is_null() {
                    continue;
                }

                let mut annot = fz_first_annot(self.ctx, page);
                while !annot.is_null() {
                    let contents_cstr = fz_annot_contents(self.ctx, annot);
                    let contents = if !contents_cstr.is_null() {
                        std::ffi::CStr::from_ptr(contents_cstr)
                            .to_string_lossy()
                            .to_string()
                    } else {
                        String::new()
                    };

                    let rect = fz_annot_rect(self.ctx, annot);

                    annotations.push(PdfAnnotation {
                        page: page_num,
                        annot_type: "Unknown".to_string(),
                        contents,
                        rect: Some((rect.x0, rect.y0, rect.x1, rect.y1)),
                        color: None,
                    });

                    annot = fz_next_annot(self.ctx, annot);
                }

                fz_drop_page(self.ctx, page);
            }
        }

        Ok(annotations)
    }
}

#[derive(Debug, Clone)]
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

impl Drop for ResourceExtractor {
    fn drop(&mut self) {
        // SAFETY: Cleaning up MuPDF resources. Pointers are checked for null before dropping.
        unsafe {
            if !self.doc.is_null() {
                fz_drop_document(self.ctx, self.doc);
            }
            if !self.ctx.is_null() {
                fz_drop_context(self.ctx);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PdfAnnotation {
    pub page: usize,
    pub annot_type: String,
    pub contents: String,
    pub rect: Option<(f32, f32, f32, f32)>,
    pub color: Option<(u8, u8, u8)>,
}

pub struct PdfAnnotationExporter {
    ctx: *mut FzContext,
    source_doc: *mut FzDocument,
    output_doc: *mut FzDocument,
    file_path: PathBuf,
    total_pages: usize,
}

impl PdfAnnotationExporter {
    pub fn new(source_path: &Path, output_path: &Path) -> Result<PdfAnnotationExporter, Error> {
        let ctx =
            new_mupdf_context().ok_or_else(|| format_err!("Failed to create MuPDF context"))?;

        // SAFETY: FFI calls to MuPDF to open source document and create output document.
        // Context is newly created and valid. CString values are null-terminated.
        unsafe {
            let source_cstr = CString::new(source_path.to_string_lossy().as_bytes())?;
            let source_doc = mp_open_document(ctx, source_cstr.as_ptr());

            if source_doc.is_null() {
                fz_drop_context(ctx);
                return Err(format_err!(
                    "Failed to open source PDF: {}",
                    source_path.display()
                ));
            }

            let total_pages = mp_count_pages(ctx, source_doc) as usize;

            let _output_cstr = CString::new(output_path.to_string_lossy().as_bytes())?;
            let output_doc = fz_new_pdf_document(ctx);
            if output_doc.is_null() {
                fz_drop_document(ctx, source_doc);
                fz_drop_context(ctx);
                return Err(format_err!("Failed to create output PDF"));
            }

            Ok(PdfAnnotationExporter {
                ctx,
                source_doc,
                output_doc,
                file_path: output_path.to_path_buf(),
                total_pages,
            })
        }
    }

    pub fn page_count(&self) -> usize {
        self.total_pages
    }

    pub fn add_annotation(&mut self, annot: PdfAnnotation) -> Result<(), Error> {
        if annot.page >= self.total_pages {
            return Err(format_err!("Page {} does not exist", annot.page + 1));
        }

        // SAFETY: FFI calls to MuPDF to create and configure annotations. Context, document, and page pointers are valid.
        // CString values are null-terminated. Annotation and page are properly dropped after use.
        unsafe {
            let page = fz_load_page(self.ctx, self.source_doc, annot.page as i32);
            if page.is_null() {
                return Err(format_err!("Failed to load page {}", annot.page + 1));
            }

            let annot_type_cstr = CString::new(annot.annot_type.as_str())?;
            let pdf_annot = fz_create_annot(self.ctx, page, annot_type_cstr.as_ptr());

            if !pdf_annot.is_null() {
                if !annot.contents.is_empty() {
                    let contents_cstr = CString::new(annot.contents.as_str())?;
                    fz_set_annot_contents(self.ctx, pdf_annot, contents_cstr.as_ptr());
                }

                if let Some((x0, y0, x1, y1)) = annot.rect {
                    let rect = FzRect { x0, y0, x1, y1 };
                    fz_set_annot_rect(self.ctx, pdf_annot, rect);
                }

                fz_drop_annot(self.ctx, pdf_annot);
            }

            fz_drop_page(self.ctx, page);
        }

        Ok(())
    }

    pub fn save(&self) -> Result<PathBuf, Error> {
        // SAFETY: FFI call to MuPDF to save document. Context and output document pointers are valid.
        // CString values are null-terminated.
        unsafe {
            let out_cstr = CString::new(self.file_path.to_string_lossy().as_bytes())?;
            let opts = FzWriteOptions::default();
            let format = CString::new("pdf")?;

            fz_save_document(
                self.ctx,
                self.output_doc,
                out_cstr.as_ptr(),
                &opts,
                format.as_ptr(),
            );

            Ok(self.file_path.clone())
        }
    }
}

impl Drop for PdfAnnotationExporter {
    fn drop(&mut self) {
        // SAFETY: Cleaning up MuPDF resources. Pointers are checked for null before dropping.
        unsafe {
            if !self.output_doc.is_null() {
                fz_drop_document(self.ctx, self.output_doc);
            }
            if !self.source_doc.is_null() {
                fz_drop_document(self.ctx, self.source_doc);
            }
            if !self.ctx.is_null() {
                fz_drop_context(self.ctx);
            }
        }
    }
}
