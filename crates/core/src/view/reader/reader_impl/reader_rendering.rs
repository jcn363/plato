//! Reader Rendering Module
//!
//! Handles page rendering, animation, text extraction, and display updates.
//!
//! ## Methods to Move Here
//! - `render()` - Main rendering to framebuffer (~200 lines)
//! - `render_animation()` - Page transition animations (~80 lines)
//! - `render_current_page()` - Render specific page
//! - `render_results()` - Highlight search results ✓
//! - `scale_page()` - Handle zoom scaling
//! - `crop_margins()` - Margin cropping logic
//! - `text_excerpt()` - Extract text from selection ✓
//! - `selected_text()` - Get currently selected text ✓
//! - `text_rect()` - Calculate text bounding box ✓
//! - `selection_rect()` - Get selection rectangle
//!
//! ## Types
//! Uses `RenderChunk`, `Resource` from reader_core for page rendering state.

use crate::document::BoundedText;
use crate::geom::{Point, Rectangle};
use crate::metadata::{Margin, ZoomMode};
use rustc_hash::FxHashMap;

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

/// Extract text content from a selection range
///
/// Extracts and concatenates text from bounded text objects within the specified
/// selection range, handling language-specific text direction and spacing.
///
/// # Arguments
/// - `text`: Hash map of page numbers to bounded text objects
/// - `sel`: Selection range as start and end points
/// - `language`: Language code for text direction handling
///
/// # Returns
/// The extracted text as a String, or None if no text found in range
pub(crate) fn text_excerpt(
    text: &FxHashMap<usize, Vec<BoundedText>>,
    sel: [Point; 2],
    language: &str,
) -> Option<String> {
    let [start, end] = sel;
    let parts = text
        .values()
        .flatten()
        .filter(|bnd| bnd.location >= start && bnd.location <= end)
        .map(|bnd| bnd.text.as_str())
        .collect::<Vec<&str>>();

    if parts.is_empty() {
        return None;
    }

    let ws = if language.starts_with("zh") || language.starts_with("ja") {
        ""
    } else {
        " "
    };
    let mut text_str = parts[0].to_string();

    for p in &parts[1..] {
        if text_str.ends_with('\u{00AD}') {
            text_str.pop();
        } else if !text_str.ends_with('-') {
            text_str.push_str(ws);
        }
        text_str += p;
    }

    Some(text_str)
}

/// Calculate bounding rectangle for selected text
///
/// Computes the minimal bounding rectangle that encompasses all bounded text objects
/// within the specified selection range.
///
/// # Arguments
/// - `text`: Hash map of page numbers to bounded text objects
/// - `chunks`: Render chunks for coordinate transformation
/// - `sel`: Selection range as start and end points
///
/// # Returns
/// The bounding rectangle for the selected text, or None if no text found
pub(crate) fn text_rect(
    text: &FxHashMap<usize, Vec<BoundedText>>,
    chunks: &[super::reader_core::RenderChunk],
    sel: [Point; 2],
) -> Option<Rectangle> {
    let [start, end] = sel;
    let mut result: Option<Rectangle> = None;

    for chunk in chunks {
        if let Some(words) = text.get(&chunk.location) {
            for word in words {
                if word.location >= start && word.location <= end {
                    let rect =
                        (word.rect * chunk.scale).to_rect() - chunk.frame.min + chunk.position;
                    if let Some(ref mut r) = result {
                        r.absorb(&rect);
                    } else {
                        result = Some(rect);
                    }
                }
            }
        }
    }

    result
}

/// Calculate bounding rectangle for current selection
///
/// Returns the rectangular region encompassing the user's current text selection,
/// or None if no selection is active.
///
/// Uses text_rect() to compute bounds for the selection start and end locations.
///
/// # Arguments
/// - `selection`: Optional current selection with start/end locations
/// - `text_data`: Hash map of page text data for bounds calculation
/// - `chunks`: Render chunks for coordinate transformation
///
/// # Returns
/// Rectangular region covering the selection, or None if no selection
pub(crate) fn selection_rect(
    selection: Option<&super::reader_core::Selection>,
    text_data: &FxHashMap<usize, Vec<BoundedText>>,
    chunks: &[super::reader_core::RenderChunk],
) -> Option<Rectangle> {
    selection.and_then(|sel| text_rect(text_data, chunks, [sel.start, sel.end]))
}

/// Calculate cropped margin offset within a page
///
/// Computes the adjusted page offset when margins are cropped to ensure the
/// content remains within valid page boundaries.
///
/// # Arguments
/// - `offset`: Original page offset
/// - `pixmap_width`: Width of the page pixmap
/// - `pixmap_height`: Height of the page pixmap
/// - `margin`: Current margin settings
///
/// # Returns
/// Adjusted page offset, or None if margins are out of bounds
pub(crate) fn calculate_margin_offset(
    offset: Point,
    pixmap_width: u32,
    pixmap_height: u32,
    margin_left: f32,
    margin_right: f32,
    margin_top: f32,
    margin_bottom: f32,
    scale: f32,
    dims: (f32, f32),
) -> Option<Point> {
    let x_ratio = offset.x as f32 / pixmap_width as f32;
    let y_ratio = offset.y as f32 / pixmap_height as f32;

    let x = if x_ratio >= margin_left && x_ratio <= (1.0 - margin_right) {
        (scale * (x_ratio - margin_left) * dims.0) as i32
    } else {
        0
    };

    let y = if y_ratio >= margin_top && y_ratio <= (1.0 - margin_bottom) {
        (scale * (y_ratio - margin_top) * dims.1) as i32
    } else {
        0
    };

    Some(Point { x, y })
}
