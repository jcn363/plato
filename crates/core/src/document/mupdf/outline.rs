use crate::document::mupdf_sys::*;
use std::ffi::CStr;

/// Safe wrapper around an MuPDF outline entry. Owned, not a reference.
pub struct Outline {
    pub(crate) ctx: *mut FzContext,
    pub(crate) outline: *mut FzOutline,
}

impl Outline {
    /// Get the raw FFI outline pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut FzOutline {
        self.outline
    }

    /// Get the outline entry's title.
    pub fn title(&self) -> String {
        unsafe {
            if (*self.outline).title.is_null() {
                "Untitled".to_string()
            } else {
                CStr::from_ptr((*self.outline).title)
                    .to_string_lossy()
                    .into_owned()
            }
        }
    }

    /// Get the outline entry's URI.
    pub fn uri(&self) -> Option<String> {
        unsafe {
            if (*self.outline).uri.is_null() {
                None
            } else {
                Some(
                    CStr::from_ptr((*self.outline).uri)
                        .to_string_lossy()
                        .into_owned(),
                )
            }
        }
    }

    /// Get the outline entry's page location.
    #[inline]
    pub fn page(&self) -> FzLocation {
        unsafe { (*self.outline).page }
    }

    /// Get the next sibling outline entry.
    pub fn next(&self) -> Option<Outline> {
        unsafe {
            let next_ptr = (*self.outline).next;
            if next_ptr.is_null() {
                None
            } else {
                Some(Outline {
                    ctx: self.ctx,
                    outline: next_ptr,
                })
            }
        }
    }

    /// Clone this outline entry (for iteration purposes).
    pub fn clone_outline(&self) -> Outline {
        Outline {
            ctx: self.ctx,
            outline: self.outline,
        }
    }

    /// Get the first child outline entry (down the tree).
    pub fn down(&self) -> Option<Outline> {
        unsafe {
            let down_ptr = (*self.outline).down;
            if down_ptr.is_null() {
                None
            } else {
                Some(Outline {
                    ctx: self.ctx,
                    outline: down_ptr,
                })
            }
        }
    }

    /// Check if this outline entry is open.
    #[inline]
    pub fn is_open(&self) -> bool {
        unsafe { (*self.outline).is_open != 0 }
    }
}

impl Drop for Outline {
    fn drop(&mut self) {
        if !self.outline.is_null() {
            unsafe { fz_drop_outline(self.ctx, self.outline) }
        }
    }
}
