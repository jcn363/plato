use crate::document::mupdf::context::MuPdfContext;
use crate::document::mupdf::link::Link;
use crate::document::mupdf::outline::Outline;
use crate::document::mupdf::page::Page;
use crate::document::mupdf_sys::*;
use anyhow::{format_err, Error};
use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

/// Safe wrapper around an MuPDF document with RAII cleanup.
pub struct Document {
    pub(crate) ctx: *mut FzContext,
    pub(crate) doc: *mut FzDocument,
}

impl Document {
    /// Open a document from a file path using an existing context.
    pub fn open(ctx: &MuPdfContext, path: &Path) -> Result<Self, Error> {
        ctx.open_document(path)
            .ok_or_else(|| format_err!("Failed to open document: {}", path.display()))
    }

    /// Get the raw FFI document pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut FzDocument {
        self.doc
    }

    /// Get the raw FFI context pointer.
    #[inline]
    pub fn ctx(&self) -> *mut FzContext {
        self.ctx
    }

    /// Load a page by index (0-based).
    pub fn load_page(&self, index: i32) -> Result<Page, Error> {
        unsafe {
            let page = mp_load_page(self.ctx, self.doc, index);
            if page.is_null() {
                Err(format_err!("Failed to load page {}", index))
            } else {
                Ok(Page {
                    ctx: self.ctx,
                    page,
                    index: index as usize,
                })
            }
        }
    }

    /// Count the total number of pages in the document.
    #[inline]
    pub fn page_count(&self) -> i32 {
        unsafe { mp_count_pages(self.ctx, self.doc) as i32 }
    }

    /// Check if the document requires a password.
    #[inline]
    pub fn needs_password(&self) -> bool {
        unsafe { fz_needs_password(self.ctx, self.doc) == 1 }
    }

    /// Authenticate with a password. Returns true if successful.
    pub fn authenticate_password(&self, password: &str) -> bool {
        unsafe {
            if let Ok(c_pass) = CString::new(password) {
                fz_authenticate_password(self.ctx, self.doc, c_pass.as_ptr()) == 1
            } else {
                false
            }
        }
    }

    /// Look up metadata by key.
    pub fn lookup_metadata(&self, key: &str) -> Option<String> {
        unsafe {
            let c_key = CString::new(key).ok()?;
            let mut buf: [libc::c_char; 256] = [0; 256];
            let len = fz_lookup_metadata(
                self.ctx,
                self.doc,
                c_key.as_ptr(),
                buf.as_mut_ptr(),
                buf.len() as libc::c_int,
            );
            if len == -1 {
                None
            } else {
                Some(CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned())
            }
        }
    }

    /// Get the document title from metadata.
    pub fn title(&self) -> Option<String> {
        self.lookup_metadata(FZ_META_INFO_TITLE)
    }

    /// Get the document author from metadata.
    pub fn author(&self) -> Option<String> {
        self.lookup_metadata(FZ_META_INFO_AUTHOR)
    }

    /// Check if the document is reflowable (e.g., EPUB).
    #[inline]
    pub fn is_reflowable(&self) -> bool {
        unsafe { fz_is_document_reflowable(self.ctx, self.doc) == 1 }
    }

    /// Check if the document is linearized (fast web view).
    #[inline]
    pub fn is_linearized(&self) -> bool {
        unsafe { fz_is_document_linearized(self.ctx, self.doc) == 1 }
    }

    /// Layout the document with the given dimensions.
    pub fn layout(&self, width: f32, height: f32) {
        unsafe { fz_layout_document(self.ctx, self.doc, width, height) }
    }

    /// Load the document outline (table of contents).
    pub fn load_outline(&self) -> Option<Outline> {
        unsafe {
            let outline = mp_load_outline(self.ctx, self.doc);
            if outline.is_null() {
                None
            } else {
                Some(Outline {
                    ctx: self.ctx,
                    outline,
                })
            }
        }
    }

    /// Resolve a link destination to a bounding rectangle.
    pub fn resolve_link_dest(&self, link: &Link) -> FzRect {
        unsafe { fz_resolve_link_dest(self.ctx, self.doc, link.as_ptr(), ptr::null_mut()) }
    }

    /// Get the page number from a location.
    pub fn page_number_from_location(&self, loc: FzLocation) -> i32 {
        unsafe { mp_page_number_from_location(self.ctx, self.doc, loc) }
    }

    /// Save the document to a file path.
    pub fn save<P: AsRef<Path>>(&self, path: P, opts: &FzWriteOptions, fmt: &str) {
        unsafe {
            if let (Ok(c_path), Ok(c_fmt)) = (
                CString::new(path.as_ref().as_os_str().as_bytes()),
                CString::new(fmt),
            ) {
                fz_save_document(self.ctx, self.doc, c_path.as_ptr(), opts, c_fmt.as_ptr());
            }
        }
    }

    /// Get the PDF output intent (for PDF/A detection).
    pub fn pdf_output_intent(&self) -> Option<String> {
        unsafe {
            let output_id = fz_pdf_output_intent(self.ctx, self.doc);
            if output_id.is_null() {
                None
            } else {
                let s = CStr::from_ptr(output_id).to_string_lossy().into_owned();
                libc::free(output_id as *mut libc::c_void);
                Some(s)
            }
        }
    }

    /// Count pages in a PDF document.
    #[inline]
    pub fn pdf_page_count(&self) -> usize {
        unsafe { fz_pdf_count_pages(self.ctx, self.doc) as usize }
    }

    /// Check if pages can be moved in this PDF.
    #[inline]
    pub fn pdf_can_move_pages(&self) -> bool {
        unsafe { fz_pdf_can_move_pages(self.ctx, self.doc) != 0 }
    }

    /// Move a page from src to dst index.
    pub fn pdf_move_page(&self, src: usize, dst: usize) {
        unsafe { fz_pdf_move_page(self.ctx, self.doc, src as libc::c_int, dst as libc::c_int) }
    }

    /// Delete a page by index.
    pub fn pdf_delete_page(&self, index: usize) {
        unsafe { fz_pdf_delete_page(self.ctx, self.doc, index as libc::c_int) }
    }

    /// Rotate a page by the given degrees.
    pub fn pdf_rotate_page(&self, index: usize, degrees: i32) {
        unsafe { fz_pdf_rotate_page(self.ctx, self.doc, index as libc::c_int, degrees) }
    }

    /// Insert a page into this PDF document after the given position.
    pub fn pdf_insert_page(&self, page: &Page, after: i32) -> i32 {
        unsafe { fz_pdf_insert_page(self.ctx, self.doc, page.as_ptr(), after) }
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        if !self.doc.is_null() {
            unsafe { fz_drop_document(self.ctx, self.doc) }
        }
    }
}
