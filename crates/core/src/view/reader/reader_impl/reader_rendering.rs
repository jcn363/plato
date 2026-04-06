//! Reader Rendering Module
//!
//! Handles page rendering, animation, text extraction, and display updates.
//!
//! ## Methods to Move Here
//! - `render()` - Main rendering to framebuffer (~200 lines)
//! - `render_animation()` - Page transition animations (~80 lines)
//! - `render_current_page()` - Render specific page
//! - `render_results()` - Highlight search results
//! - `scale_page()` - Handle zoom scaling
//! - `crop_margins()` - Margin cropping logic
//! - `text_excerpt()` - Extract text from selection
//! - `selected_text()` - Get currently selected text
//! - `text_rect()` - Calculate text bounding box
//! - `selection_rect()` - Get selection rectangle
//!
//! ## Types
//! Uses `RenderChunk`, `Resource` from reader_core for page rendering state.

use crate::geom::Rectangle;
use crate::metadata::{Margin, ZoomMode};

/// Calculate page scaling factor based on zoom mode
///
/// Determines how much to scale page content based on viewport and zoom settings.
///
/// # Arguments
/// - `rect`: Display rectangle dimensions
/// - `_margin`: Page margin (currently unused, for future expansion)
/// - `margin_width`: Margin width in pixels
/// - `dims`: Page dimensions (width, height)
/// - `zoom_mode`: Current zoom mode
///
/// # Returns
/// Scale factor to apply to page rendering (1.0 = native size)
///
/// Extracted from `Reader::scaling_factor()` (line 1788)
#[allow(dead_code)]
pub(crate) fn scaling_factor(
    rect: &Rectangle,
    _margin: &Margin,
    margin_width: i32,
    dims: (f32, f32),
    zoom_mode: ZoomMode,
) -> f32 {
    match zoom_mode {
        ZoomMode::FitToPage => {
            let scale_x = (rect.width() as f32 - 2.0 * margin_width as f32) / dims.0;
            let scale_y = (rect.height() as f32 - 2.0 * margin_width as f32) / dims.1;
            scale_x.min(scale_y)
        }
        ZoomMode::FitToWidth => {
            let scale_x = (rect.width() as f32 - 2.0 * margin_width as f32) / dims.0;
            scale_x
        }
        _ => 1.0,
    }
}
