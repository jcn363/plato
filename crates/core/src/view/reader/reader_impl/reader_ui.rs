//! Reader UI Module
//!
//! UI update helpers and basic view operations for the Reader.

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::view::notification::Notification;
use crate::view::{Event, Hub, RenderData, RenderQueue, View};

use super::reader::Reader;

impl Reader {
    /// Update the view
    pub fn update(
        &mut self,
        _update: Option<UpdateMode>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.queue_partial_update(rq);
    }

    /// Update the toolbar
    pub fn update_tool_bar(&mut self, rq: &mut RenderQueue, _context: &Context) {
        self.queue_partial_update(rq);
    }

    /// Update the bottom bar
    pub fn update_bottom_bar(&mut self, rq: &mut RenderQueue) {
        self.queue_partial_update(rq);
    }

    /// Update non-inverted regions
    pub fn update_noninverted_regions(&mut self, rq: &mut RenderQueue) {
        self.queue_partial_update(rq);
    }

    /// Save document reading state
    pub fn handle_save(&mut self, hub: &Hub, rq: &mut RenderQueue, _context: &mut Context) {
        // Update metadata with current reading position
        if let Some(ref mut r) = self.info.reader {
            r.current_page = self.current_page;
            r.pages_count = self.pages_count;
            r.finished = self.finished;

            // Save to library if available
            if !self.ephemeral {
                hub.send(Event::Save).ok();
            }
        }

        // Trigger UI update to reflect saved state
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    /// Handle focus change
    pub fn handle_focus(
        &mut self,
        _v: bool,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.queue_partial_update(rq);
    }

    /// Toggle top toolbar visibility
    pub fn toggle_bars(
        &mut self,
        show: Option<bool>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.bars_visible = show.unwrap_or(!self.bars_visible);
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Toggle keyboard
    pub fn toggle_keyboard(
        &mut self,
        enable: bool,
        _update: Option<UpdateMode>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        if enable {
            self.focus = Some(crate::view::ViewId::Keyboard);
        } else {
            self.focus = None;
        }
        self.queue_partial_update(rq);
    }

    /// Toggle search bar
    pub fn toggle_search_bar(
        &mut self,
        enable: bool,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        if enable {
            self.focus = Some(crate::view::ViewId::SearchBar);
        } else {
            self.focus = None;
        }
        self.queue_partial_update(rq);
    }

    /// Toggle margin cropper
    pub fn toggle_margin_cropper(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.margin_cropper_visible = !self.margin_cropper_visible;
        self.queue_partial_update(rq);
    }

    /// Handle shown
    pub fn handle_shown(&mut self, _hub: &Hub, rq: &mut RenderQueue, _context: &mut Context) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Full));
    }

    /// Handle open
    pub fn handle_open(
        &mut self,
        _file: &Box<crate::metadata::Info>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Full));
    }

    /// Show notification message
    pub fn show_notification(
        &mut self,
        message: String,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let notif = Notification::new(message, hub, rq, context);
        self.children.push(Box::new(notif) as Box<dyn View>);
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }
}
