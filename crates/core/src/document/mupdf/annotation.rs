use crate::document::mupdf_sys::*;
use std::ffi::{CStr, CString};

/// Safe wrapper around an MuPDF annotation with RAII cleanup.
pub struct Annotation {
    pub(crate) ctx: *mut FzContext,
    pub(crate) annot: *mut FzAnnot,
}

impl Annotation {
    /// Get the raw FFI annotation pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut FzAnnot {
        self.annot
    }

    /// Get the annotation's contents as a string.
    pub fn contents(&self) -> String {
        unsafe {
            let contents_cstr = fz_annot_contents(self.ctx, self.annot);
            if contents_cstr.is_null() {
                String::new()
            } else {
                let s = CStr::from_ptr(contents_cstr).to_string_lossy().into_owned();
                libc::free(contents_cstr as *mut libc::c_void);
                s
            }
        }
    }

    /// Set the annotation's contents.
    pub fn set_contents(&self, contents: &str) {
        if let Ok(c_contents) = CString::new(contents) {
            unsafe { fz_set_annot_contents(self.ctx, self.annot, c_contents.as_ptr()) }
        }
    }

    /// Get the annotation's rectangle.
    #[inline]
    pub fn rect(&self) -> FzRect {
        unsafe { fz_annot_rect(self.ctx, self.annot) }
    }

    /// Set the annotation's rectangle.
    pub fn set_rect(&self, rect: FzRect) {
        unsafe { fz_set_annot_rect(self.ctx, self.annot, rect) }
    }

    /// Get the next annotation on the page.
    pub fn next(&self) -> Option<Annotation> {
        unsafe {
            let next_annot = fz_next_annot(self.ctx, self.annot);
            if next_annot.is_null() {
                None
            } else {
                Some(Annotation {
                    ctx: self.ctx,
                    annot: next_annot,
                })
            }
        }
    }
}

impl Drop for Annotation {
    fn drop(&mut self) {
        if !self.annot.is_null() {
            unsafe { fz_drop_annot(self.ctx, self.annot) }
        }
    }
}
