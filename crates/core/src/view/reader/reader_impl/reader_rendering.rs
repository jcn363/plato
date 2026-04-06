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

/// Extract text from a range of points
///
/// Retrieves the text content between two points in the document,
/// handling proper spacing and joining for different languages.
///
/// # Arguments
/// - `text`: Map of text locations to bounded text items
/// - `sel`: Array of [start_point, end_point] for the selection
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

/// Get the text from a selected region
///
/// Extracts text using the given selection, handling language-specific
/// spacing and word joining.
#[allow(dead_code)]
pub(crate) fn selected_text(
    text: &FxHashMap<usize, Vec<BoundedText>>,
    start: Point,
    end: Point,
    language: &str,
) -> Option<String> {
    text_excerpt(text, [start, end], language)
}

/// Calculate the bounding rectangle for text in a selection
///
/// Finds the overall rectangle that encompasses all text between two points
/// across multiple chunks, accounting for chunk scaling and positioning.
///
/// # Arguments
/// - `text`: Map of text locations to bounded text items
/// - `chunks`: Rendered page chunks with positioning info
/// - `sel`: Array of [start_point, end_point] for the selection
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
/// - `text_data`: FxHashMap containing BoundedText for all page chunks
/// - `chunks`: Rendered page chunks with positioning info
///
/// # Returns
/// Rectangular region covering the selection, or None if no selection
#[allow(dead_code)]
pub(crate) fn selection_rect(
    selection: Option<&super::reader_core::Selection>,
    text_data: &FxHashMap<usize, Vec<BoundedText>>,
    chunks: &[super::reader_core::RenderChunk],
) -> Option<Rectangle> {
    selection.and_then(|sel| text_rect(text_data, chunks, [sel.start, sel.end]))
}
