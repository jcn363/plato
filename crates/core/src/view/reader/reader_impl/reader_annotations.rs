//! Reader Annotations Module
//!
//! Handles annotation management, notes, highlighting, and bookmarks.

use crate::context::Context;
use crate::document::TextLocation;
use crate::framebuffer::UpdateMode;
use crate::geom::{Point, Rectangle};
use crate::metadata::{Annotation, Info};
use crate::unit::{mm_to_px, scale_by_dpi};
use crate::view::{Hub, Id, RenderData, RenderQueue, View};
use crate::view::notification::Notification;
use chrono::Local;

use super::reader::Reader;

/// Find annotation by text selection (mutable reference)
///
/// Searches through annotations in the given Info struct to find one matching
/// the specified text selection range.
///
/// Returns a mutable reference to the annotation if found, or None.
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
    let dpi = crate::unit::get_device_dpi();
    let thickness = scale_by_dpi(3.0, dpi) as u16;
    let radius = mm_to_px(0.4, dpi) as i32 + thickness as i32;
    let center = Point {
        x: rect.max.x - 5 * radius,
        y: rect.min.y + 5 * radius,
    };
    let bookmark_rect = Rectangle::from_disk(center, radius);
    rq.add(RenderData::new(reader_id, bookmark_rect, UpdateMode::Gui));
}

impl Reader {
    /// Get text excerpt from current selection
    pub fn get_selected_text_excerpt(&self) -> Option<String> {
        self.selection.as_ref().and_then(|sel| {
            let points = [sel.start, sel.end];
            self.text_excerpt(points)
        })
    }

    /// Handle adjust selection
    pub fn handle_adjust_selection(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &Context,
    ) {
        self.selection = None;
        self.queue_partial_update(rq);
    }

    /// Handle edit note submit - update annotation note text
    pub fn handle_edit_note_submit(
        &mut self,
        note: &str,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &Context,
    ) {
        if let Some(ref target) = self._target_annotation {
            if let Some(ann) = self.find_annotation_mut(*target) {
                ann.note = note.to_string();
                ann.modified = Local::now().naive_local();
            }
        }
        self.queue_partial_update(rq);
    }

    /// Handle close edit note
    pub fn handle_close_edit_note(&mut self, _hub: &Hub, rq: &mut RenderQueue, _context: &Context) {
        self._target_annotation = None;
        self.focus = None;
        self.queue_partial_update(rq);
    }

    /// Handle show annotations - display annotation sidebar
    pub fn handle_show_annotations(
        &mut self,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let annotation_count = self
            .info
            .reader
            .as_ref()
            .map(|r| r.annotations.len())
            .unwrap_or(0);

        if annotation_count > 0 {
            let msg = format!("{} annotations in document", annotation_count);
            let notif = Notification::new(msg, hub, rq, context);
            self.children.push(Box::new(notif) as Box<dyn View>);
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        } else {
            let msg = "No annotations in this document".to_string();
            let notif = Notification::new(msg, hub, rq, context);
            self.children.push(Box::new(notif) as Box<dyn View>);
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
        }
    }

    /// Handle show bookmarks - display bookmark list
    pub fn handle_show_bookmarks(
        &mut self,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let bookmark_count = self
            .info
            .reader
            .as_ref()
            .map(|r| r.bookmarks.len())
            .unwrap_or(0);

        if bookmark_count > 0 {
            let msg = format!("{} bookmarks in document", bookmark_count);
            let notif = Notification::new(msg, hub, rq, context);
            self.children.push(Box::new(notif) as Box<dyn View>);
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        } else {
            let msg = "No bookmarks in this document".to_string();
            let notif = Notification::new(msg, hub, rq, context);
            self.children.push(Box::new(notif) as Box<dyn View>);
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
        }
    }
}
