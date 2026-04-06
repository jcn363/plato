//! Reader Annotations Module
//!
//! Handles annotation management, notes, highlighting, and bookmarks.
//!
//! ## Methods to Move Here
//! - `toggle_edit_note()` - Edit annotation note (~45 lines)
//! - `toggle_annotation_menu()` - Annotation context menu (~70 lines)
//! - `find_annotation_ref()` - Lookup annotation ✓ MOVED
//! - `find_annotation_mut()` - Lookup and mutate annotation ✓ MOVED
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
use crate::document::TextLocation;
use crate::framebuffer::UpdateMode;
use crate::geom::{Point, Rectangle};
use crate::metadata::{Annotation, Info};
use crate::unit::{mm_to_px, scale_by_dpi};
use crate::view::{Id, RenderData, RenderQueue};

/// Find annotation by text selection (immutable reference)
///
/// Searches through annotations in the given Info struct to find one matching
/// the specified text selection range.
///
/// Returns an immutable reference to the annotation if found, or None.
#[allow(dead_code)]
pub(crate) fn find_annotation_ref(info: &Info, sel: [TextLocation; 2]) -> Option<&Annotation> {
    info.reader.as_ref().and_then(|r| {
        r.annotations
            .iter()
            .find(|a| a.selection[0] == sel[0] && a.selection[1] == sel[1])
    })
}

/// Find annotation by text selection (mutable reference)
///
/// Searches through annotations in the given Info struct to find one matching
/// the specified text selection range.
///
/// Returns a mutable reference to the annotation if found, or None.
#[allow(dead_code)]
pub(crate) fn find_annotation_mut(
    info: &mut Info,
    sel: [TextLocation; 2],
) -> Option<&mut Annotation> {
    info.reader.as_mut().and_then(|r| {
        r.annotations
            .iter_mut()
            .find(|a| a.selection[0] == sel[0] && a.selection[1] == sel[1])
    })
}

/// Toggle bookmark at current page
///
/// This is extracted from `Reader::toggle_bookmark()` and manages bookmark state
/// and UI invalidation in a single operation.
#[allow(dead_code)]
pub(crate) fn toggle_bookmark(
    current_page: usize,
    info: &mut Info,
    reader_id: Id,
    rect: Rectangle,
    rq: &mut RenderQueue,
) {
    // Toggle bookmark state
    if let Some(ref mut r) = info.reader {
        if !r.bookmarks.insert(current_page) {
            r.bookmarks.remove(&current_page);
        }
    }

    // Invalidate bookmark indicator region
    let dpi = CURRENT_DEVICE.dpi;
    let thickness = scale_by_dpi(3.0, dpi) as u16;
    let radius = mm_to_px(0.4, dpi) as i32 + thickness as i32;
    let center = Point {
        x: rect.max.x - 5 * radius,
        y: rect.min.y + 5 * radius,
    };
    let bookmark_rect = Rectangle::from_disk(center, radius);
    rq.add(RenderData::new(reader_id, bookmark_rect, UpdateMode::Gui));
}
