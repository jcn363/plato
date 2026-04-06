//! Reader Annotations Module
//!
//! Handles annotation management, notes, highlighting, and bookmarks.
//!
//! ## Methods to Move Here
//! - `toggle_edit_note()` - Edit annotation note (~45 lines)
//! - `toggle_annotation_menu()` - Annotation context menu (~70 lines)
//! - `find_annotation_ref()` - Lookup annotation
//! - `find_annotation_mut()` - Lookup and mutate annotation
//! - `toggle_bookmark()` - Bookmark management (~15 lines) ✓ MOVED
//! - `update_annotations()` - Update annotation UI
//! - `go_to_annotation()` - Navigate to annotation
//!
//! ## Size
//! Moderate size (~600 lines total), relatively well-contained.
//!
//! ## Dependencies
//! Depends on Reader's info and selection state.

use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::geom::{Point, Rectangle};
use crate::unit::{mm_to_px, scale_by_dpi};
use crate::view::{Id, RenderData, RenderQueue};

/// Toggle bookmark at current page
///
/// This is extracted from `Reader::toggle_bookmark()` and can be called
/// to manage bookmarks independently of the full Reader state.
pub(crate) fn toggle_bookmark_at_page(
    current_page: usize,
    rect_min_x: i32,
    rect_min_y: i32,
    rect_max_x: i32,
    rect_max_y: i32,
    reader_id: Id,
    bookmarks: &mut std::collections::BTreeSet<usize>,
    rq: &mut RenderQueue,
) {
    // Toggle bookmark state
    if !bookmarks.insert(current_page) {
        bookmarks.remove(&current_page);
    }

    // Invalidate bookmark indicator region
    let dpi = CURRENT_DEVICE.dpi;
    let thickness = scale_by_dpi(3.0, dpi) as u16;
    let radius = mm_to_px(0.4, dpi) as i32 + thickness as i32;
    let center = Point {
        x: rect_max_x - 5 * radius,
        y: rect_min_y + 5 * radius,
    };
    let rect = Rectangle::from_disk(center, radius);
    rq.add(RenderData::new(reader_id, rect, UpdateMode::Gui));
}

pub(crate) use super::reader_core::Selection;
