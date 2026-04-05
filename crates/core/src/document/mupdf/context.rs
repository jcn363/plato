use crate::document::mupdf::document::Document;
use crate::document::mupdf_sys::*;
use anyhow::{format_err, Error};
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use std::rc::Rc;

pub use crate::document::mupdf_sys::{fz_identity, FzMatrix};

/// Identity matrix constant for transformations.
pub const IDENTITY: FzMatrix = fz_identity;

/// Create a scale transformation matrix.
#[inline]
pub fn scale(sx: f32, sy: f32) -> FzMatrix {
    unsafe { fz_scale(sx as libc::c_float, sy as libc::c_float) }
}

/// Concatenate two transformation matrices.
#[inline]
pub fn concat(a: FzMatrix, b: FzMatrix) -> FzMatrix {
    unsafe { fz_concat(ptr::null_mut(), a, b) }
}

/// Invert a transformation matrix.
#[inline]
pub fn invert_matrix(m: FzMatrix) -> FzMatrix {
    unsafe { fz_invert_matrix(ptr::null_mut(), m) }
}

/// Safe wrapper around an MuPDF context with RAII cleanup.
/// Uses Rc for shared ownership across documents.
pub struct MuPdfContext {
    inner: Rc<ContextInner>,
}

struct ContextInner {
    ctx: *mut FzContext,
}

impl MuPdfContext {
    /// Create a new MuPDF context with document handlers registered.
    pub fn new() -> Result<Self, Error> {
        new_mupdf_context()
            .map(|ctx| MuPdfContext {
                inner: Rc::new(ContextInner { ctx }),
            })
            .ok_or_else(|| format_err!("Failed to create MuPDF context"))
    }

    /// Get the raw FFI context pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut FzContext {
        self.inner.ctx
    }

    /// Open a document from a file path.
    pub fn open_document<P: AsRef<Path>>(&self, path: P) -> Option<Document> {
        unsafe {
            let c_path = CString::new(path.as_ref().as_os_str().as_bytes()).ok()?;
            let doc = mp_open_document(self.inner.ctx, c_path.as_ptr());
            if doc.is_null() {
                None
            } else {
                Some(Document {
                    ctx: self.inner.ctx,
                    doc,
                })
            }
        }
    }

    /// Open a document from a memory buffer with a magic string hint.
    pub fn open_document_memory(&self, magic: &str, buf: &[u8]) -> Option<Document> {
        unsafe {
            let stream = fz_open_memory(
                self.inner.ctx,
                buf.as_ptr() as *const libc::c_uchar,
                buf.len() as libc::size_t,
            );
            if stream.is_null() {
                return None;
            }
            let c_magic = CString::new(magic).ok()?;
            let doc = mp_open_document_with_stream(self.inner.ctx, c_magic.as_ptr(), stream);
            fz_drop_stream(self.inner.ctx, stream);
            if doc.is_null() {
                None
            } else {
                Some(Document {
                    ctx: self.inner.ctx,
                    doc,
                })
            }
        }
    }

    /// Set a global user stylesheet for HTML rendering.
    pub fn set_user_css(&self, css: &str) {
        if let Ok(c_css) = CString::new(css) {
            unsafe { fz_set_user_css(self.inner.ctx, c_css.as_ptr()) }
        }
    }

    /// Enable or disable the use of document-embedded CSS.
    pub fn set_use_document_css(&self, use_doc_css: bool) {
        unsafe { fz_set_use_document_css(self.inner.ctx, use_doc_css as libc::c_int) }
    }

    /// Create a new empty PDF document.
    pub fn new_pdf_document(&self) -> Option<Document> {
        unsafe {
            let doc = fz_new_pdf_document(self.inner.ctx);
            if doc.is_null() {
                None
            } else {
                Some(Document {
                    ctx: self.inner.ctx,
                    doc,
                })
            }
        }
    }

    /// Get the gray colorspace.
    #[inline]
    pub fn device_gray(&self) -> *mut FzColorspace {
        unsafe { fz_device_gray(self.inner.ctx) }
    }

    /// Get the RGB colorspace.
    #[inline]
    pub fn device_rgb(&self) -> *mut FzColorspace {
        unsafe { fz_device_rgb(self.inner.ctx) }
    }
}

impl Clone for MuPdfContext {
    fn clone(&self) -> Self {
        MuPdfContext {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl Drop for ContextInner {
    fn drop(&mut self) {
        if !self.ctx.is_null() {
            unsafe { fz_drop_context(self.ctx) }
        }
    }
}
