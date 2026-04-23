//! Helper Functions for Reader Settings
//!
//! Helper functions for updating contrast, scroll mode, and zoom mode settings.

use crate::geom::Point;
use crate::metadata::{Info, ScrollMode, ZoomMode};
use crate::view::reader::reader_impl::reader_core::Contrast;

pub(crate) fn update_contrast_exponent(info: &mut Info, contrast: &mut Contrast, exponent: f32) {
    if let Some(ref mut r) = info.reader {
        r.contrast_exponent = Some(exponent);
    }
    contrast.exponent = exponent;
}

pub(crate) fn update_contrast_gray(info: &mut Info, contrast: &mut Contrast, gray: f32) {
    if let Some(ref mut r) = info.reader {
        r.contrast_gray = Some(gray);
    }
    contrast.gray = gray;
}

pub(crate) fn update_scroll_mode(
    scroll_mode_ref: &mut ScrollMode,
    page_offset_ref: &mut Point,
    scroll_mode: ScrollMode,
) {
    *scroll_mode_ref = scroll_mode;
    *page_offset_ref = Point { x: 0, y: 0 };
}

pub(crate) fn update_zoom_mode(
    zoom_mode_ref: &mut ZoomMode,
    page_offset_ref: &mut Point,
    zoom_mode: ZoomMode,
    reset_page_offset: bool,
) {
    *zoom_mode_ref = zoom_mode;
    if reset_page_offset {
        *page_offset_ref = Point { x: 0, y: 0 };
    }
}
