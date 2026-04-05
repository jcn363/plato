use crate::document::mupdf_sys::*;
use std::ffi::CStr;

/// Safe wrapper around an MuPDF link. Owned, not a reference.
pub struct Link {
    pub(crate) ctx: *mut FzContext,
    pub(crate) link: *mut FzLink,
}

impl Link {
    /// Get the raw FFI link pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut FzLink {
        self.link
    }

    /// Get the link's rectangle.
    #[inline]
    pub fn rect(&self) -> FzRect {
        unsafe { (*self.link).rect }
    }

    /// Get the link's URI as a string.
    pub fn uri(&self) -> String {
        unsafe {
            if (*self.link).uri.is_null() {
                String::new()
            } else {
                CStr::from_ptr((*self.link).uri)
                    .to_string_lossy()
                    .into_owned()
            }
        }
    }

    /// Get the next link in the chain, consuming this link's next pointer.
    /// Returns None if there is no next link.
    pub fn next(&self) -> Option<Link> {
        unsafe {
            let next_ptr = (*self.link).next;
            if next_ptr.is_null() {
                None
            } else {
                Some(Link {
                    ctx: self.ctx,
                    link: next_ptr,
                })
            }
        }
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        if !self.link.is_null() {
            unsafe { fz_drop_link(self.ctx, self.link) }
        }
    }
}
