use crate::document::mupdf::annotation::Annotation;
use crate::document::mupdf_sys::*;
use std::ffi::CString;

/// Safe wrapper around an MuPDF pixmap with RAII cleanup.
pub struct Pixmap {
    pub(crate) ctx: *mut FzContext,
    pub(crate) pixmap: *mut FzPixmap,
}

impl Pixmap {
    /// Get the raw FFI pixmap pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut FzPixmap {
        self.pixmap
    }

    /// Get the pixmap width.
    #[inline]
    pub fn width(&self) -> i32 {
        unsafe { (*self.pixmap).w }
    }

    /// Get the pixmap height.
    #[inline]
    pub fn height(&self) -> i32 {
        unsafe { (*self.pixmap).h }
    }

    /// Get the number of components (samples per pixel).
    #[inline]
    pub fn components(&self) -> i32 {
        unsafe { (*self.pixmap).n }
    }

    /// Check if the pixmap has an alpha channel.
    #[inline]
    pub fn has_alpha(&self) -> bool {
        unsafe { (*self.pixmap).alpha != 0 }
    }

    /// Get the x offset of the pixmap.
    #[inline]
    pub fn x(&self) -> i32 {
        unsafe { (*self.pixmap).x }
    }

    /// Get the y offset of the pixmap.
    #[inline]
    pub fn y(&self) -> i32 {
        unsafe { (*self.pixmap).y }
    }

    /// Get the stride (bytes per row).
    #[inline]
    pub fn stride(&self) -> isize {
        unsafe { (*self.pixmap).stride }
    }

    /// Get a pointer to the sample data.
    #[inline]
    pub fn samples(&self) -> *mut libc::c_uchar {
        unsafe { (*self.pixmap).samples }
    }

    /// Clear the pixmap to transparent black.
    pub fn clear(&self) {
        unsafe { fz_clear_pixmap(self.ctx, self.pixmap) }
    }

    /// Fill the pixmap with a color.
    pub fn fill(&self, color: *mut FzColorspace, color_vals: *const libc::c_float, alpha: f32) {
        unsafe { fz_fill_pixmap(self.ctx, self.pixmap, color, color_vals, alpha) }
    }
}

impl Drop for Pixmap {
    fn drop(&mut self) {
        if !self.pixmap.is_null() {
            unsafe { fz_drop_pixmap(self.ctx, self.pixmap) }
        }
    }
}

/// Helper to create a new bounding box device.
pub fn new_bbox_device(ctx: *mut FzContext, rect: FzRect) -> Option<*mut FzDevice> {
    unsafe {
        let dev = fz_new_bbox_device(ctx, rect);
        if dev.is_null() {
            None
        } else {
            Some(dev)
        }
    }
}

/// Helper to close a device.
pub fn close_device(ctx: *mut FzContext, device: *mut FzDevice) {
    unsafe { fz_close_device(ctx, device) }
}

/// Helper to drop a device.
pub fn drop_device(ctx: *mut FzContext, device: *mut FzDevice) {
    if !device.is_null() {
        unsafe { fz_drop_device(ctx, device) }
    }
}

/// Helper to get the gray colorspace.
pub fn device_gray(ctx: *mut FzContext) -> *mut FzColorspace {
    unsafe { fz_device_gray(ctx) }
}

/// Helper to get the RGB colorspace.
pub fn device_rgb(ctx: *mut FzContext) -> *mut FzColorspace {
    unsafe { fz_device_rgb(ctx) }
}

/// Helper to create a new pixmap.
pub fn new_pixmap(
    ctx: *mut FzContext,
    colorspace: *mut FzColorspace,
    w: i32,
    h: i32,
    alpha: bool,
) -> Option<Pixmap> {
    unsafe {
        let pixmap = fz_new_pixmap(ctx, colorspace, w, h, alpha as libc::c_int);
        if pixmap.is_null() {
            None
        } else {
            Some(Pixmap { ctx, pixmap })
        }
    }
}

/// Helper to convert a quad to a rect.
pub fn rect_from_quad(ctx: *mut FzContext, quad: FzQuad) -> FzRect {
    unsafe { fz_rect_from_quad(ctx, quad) }
}

/// Helper to union two rects.
pub fn union_rect(ctx: *mut FzContext, a: FzRect, b: FzRect) -> FzRect {
    unsafe { fz_union_rect(ctx, a, b) }
}

/// Helper to create a new draw device for a pixmap.
pub fn new_draw_device(
    ctx: *mut FzContext,
    matrix: FzMatrix,
    pixmap: *mut FzPixmap,
) -> Option<*mut FzDevice> {
    unsafe {
        let dev = fz_new_draw_device(ctx, matrix, pixmap);
        if dev.is_null() {
            None
        } else {
            Some(dev)
        }
    }
}

/// Create an annotation on a page.
pub fn create_annot(ctx: *mut FzContext, page: *mut FzPage, type_: &str) -> Option<Annotation> {
    unsafe {
        if let Ok(c_type) = CString::new(type_) {
            let annot = fz_create_annot(ctx, page, c_type.as_ptr());
            if annot.is_null() {
                None
            } else {
                Some(Annotation { ctx, annot })
            }
        } else {
            None
        }
    }
}
