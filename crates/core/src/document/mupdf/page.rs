#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::document::mupdf::annotation::Annotation;
use crate::document::mupdf::image::Image;
use crate::document::mupdf::link::Link;
use crate::document::mupdf::pixmap::Pixmap;
use crate::document::mupdf::text::TextPage;
use crate::document::mupdf_sys::*;
use anyhow::{format_err, Error};
use std::ffi::CString;
use std::ptr;

/// Safe wrapper around an MuPDF page with RAII cleanup.
pub struct Page {
    pub(crate) ctx: *mut FzContext,
    pub(crate) page: *mut FzPage,
    pub(crate) index: usize,
}

impl Page {
    /// Get the raw FFI page pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut FzPage {
        self.page
    }

    /// Get the raw FFI context pointer.
    #[inline]
    pub fn ctx(&self) -> *mut FzContext {
        self.ctx
    }

    /// Get the page index (0-based).
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get the bounding box of the page.
    #[inline]
    pub fn bound(&self) -> FzRect {
        unsafe { fz_bound_page(self.ctx, self.page) }
    }

    /// Get the page dimensions (width, height).
    pub fn dims(&self) -> (f32, f32) {
        let bounds = self.bound();
        (
            (bounds.x1 - bounds.x0) as f32,
            (bounds.y1 - bounds.y0) as f32,
        )
    }

    /// Get the page width.
    #[inline]
    pub fn width(&self) -> f32 {
        self.dims().0
    }

    /// Get the page height.
    #[inline]
    pub fn height(&self) -> f32 {
        self.dims().1
    }

    /// Create a pixmap from this page at the given scale and colorspace.
    pub fn to_pixmap(
        &self,
        matrix: FzMatrix,
        colorspace: *mut FzColorspace,
        alpha: bool,
    ) -> Option<Pixmap> {
        unsafe {
            let pixmap = mp_new_pixmap_from_page(
                self.ctx,
                self.page,
                matrix,
                colorspace,
                alpha as libc::c_int,
            );
            if pixmap.is_null() {
                None
            } else {
                Some(Pixmap {
                    ctx: self.ctx,
                    pixmap,
                })
            }
        }
    }

    /// Render a pixmap from this page, returning raw pixel data.
    pub fn render_pixmap(
        &self,
        matrix: FzMatrix,
        colorspace: *mut FzColorspace,
        alpha: i32,
    ) -> Result<crate::framebuffer::Pixmap, Error> {
        unsafe {
            let pixmap = mp_new_pixmap_from_page(self.ctx, self.page, matrix, colorspace, alpha);
            if pixmap.is_null() {
                return Err(format_err!("Failed to render pixmap"));
            }

            let width = (*pixmap).w as u32;
            let height = (*pixmap).h as u32;
            let samples = (*pixmap).n as usize;
            let len = samples * (width * height) as usize;

            let mut data = Vec::with_capacity(len);
            data.extend_from_slice(std::slice::from_raw_parts((*pixmap).samples, len));
            fz_drop_pixmap(self.ctx, pixmap);

            Ok(crate::framebuffer::Pixmap {
                width,
                height,
                samples,
                data,
                update_flag: false,
            })
        }
    }

    /// Create a text page from this page with optional options.
    pub fn to_text_page(&self, options: Option<&FzTextOptions>) -> Option<TextPage> {
        unsafe {
            let opts_ptr = options
                .map(|o| o as *const FzTextOptions)
                .unwrap_or(ptr::null());
            let tp = mp_new_stext_page_from_page(self.ctx, self.page, opts_ptr);
            if tp.is_null() {
                None
            } else {
                Some(TextPage {
                    ctx: self.ctx,
                    text_page: tp,
                })
            }
        }
    }

    /// Load links from this page.
    pub fn load_links(&self) -> Option<Link> {
        unsafe {
            let link = mp_load_links(self.ctx, self.page);
            if link.is_null() {
                None
            } else {
                Some(Link {
                    ctx: self.ctx,
                    link,
                })
            }
        }
    }

    /// Search for text on the page. Returns the number of hits found.
    pub fn search(&self, text: &str, hits: &mut [FzRect]) -> i32 {
        unsafe {
            if let Ok(c_text) = CString::new(text) {
                fz_search_page(
                    self.ctx,
                    self.page,
                    c_text.as_ptr(),
                    hits.as_mut_ptr(),
                    hits.len() as libc::c_int,
                )
            } else {
                -1
            }
        }
    }

    /// Count images on this page.
    #[inline]
    pub fn count_images(&self) -> usize {
        unsafe { fz_count_page_images(self.ctx, self.page) as usize }
    }

    /// Load an image by index from this page.
    pub fn load_image(&self, index: usize) -> Option<Image> {
        unsafe {
            let image = fz_load_page_image(self.ctx, self.page, index as libc::c_int);
            if image.is_null() {
                None
            } else {
                Some(Image {
                    ctx: self.ctx,
                    image,
                })
            }
        }
    }

    /// Count fonts on this page.
    #[inline]
    pub fn count_fonts(&self) -> usize {
        unsafe { fz_count_page_fonts(self.ctx, self.page) as usize }
    }

    /// Get the first annotation on this page.
    pub fn first_annot(&self) -> Option<Annotation> {
        unsafe {
            let annot = fz_first_annot(self.ctx, self.page);
            if annot.is_null() {
                None
            } else {
                Some(Annotation {
                    ctx: self.ctx,
                    annot,
                })
            }
        }
    }

    /// Run the page through a device with the given matrix.
    pub fn run(&self, device: *mut FzDevice, matrix: FzMatrix) {
        unsafe { fz_run_page(self.ctx, self.page, device, matrix, ptr::null_mut()) }
    }

    /// Apply redactions to this page.
    pub fn apply_redactions(&self, flags: i32) {
        unsafe { fz_apply_redactions(self.ctx, self.page, flags) }
    }

    /// Remove redactions from this page.
    pub fn remove_redactions(&self) {
        unsafe { fz_remove_redactions(self.ctx, self.page) }
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        if !self.page.is_null() {
            unsafe { fz_drop_page(self.ctx, self.page) }
        }
    }
}
