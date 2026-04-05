use crate::document::mupdf_sys::*;

/// Safe wrapper around an MuPDF image with RAII cleanup.
pub struct Image {
    pub(crate) ctx: *mut FzContext,
    pub(crate) image: *mut FzImage,
}

impl Image {
    /// Get the raw FFI image pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut FzImage {
        self.image
    }

    /// Get the image width.
    #[inline]
    pub fn width(&self) -> usize {
        unsafe { fz_image_width(self.ctx, self.image) as usize }
    }

    /// Get the image height.
    #[inline]
    pub fn height(&self) -> usize {
        unsafe { fz_image_height(self.ctx, self.image) as usize }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        if !self.image.is_null() {
            unsafe { fz_drop_image(self.ctx, self.image) }
        }
    }
}
