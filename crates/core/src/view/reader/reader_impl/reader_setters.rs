//! Reader Settings Setters
//!
//! This module provides setter methods for the Reader's display and rendering settings.
//! These methods handle font configuration, text alignment, zoom modes, scroll modes,
//! contrast adjustments, and margin settings.
//!
//! All setter methods follow a consistent pattern:
//! 1. Validate that the document can be modified
//! 2. Update the info.reader configuration
//! 3. Reapply the document with new settings
//! 4. Refresh the UI components (tool bar, bottom bar)
//! 5. Queue appropriate render updates
use crate::context::Context;
use crate::geom::Rectangle;
use crate::metadata::Margin;
use crate::view::{Hub, RenderQueue, ViewId, View};
use crate::view::common::locate_by_id;
use crate::view::menu::MenuEntry;
use crate::document::{Document, Location};
use crate::metadata::{TextAlign, ZoomMode, ScrollMode, CroppingMargins};
use crate::framebuffer::UpdateMode;
use crate::device::CURRENT_DEVICE;
use crate::settings::DEFAULT_FONT_FAMILY;
use std::sync::{Arc, MutexGuard};

use super::reader::Reader;
use super::reader_core::Resource;
use super::{reader_rendering, reader_settings};

impl Reader {
    /// Helper: Refresh UI after document settings change
    /// Clears caches, updates view, and refreshes toolbars
    #[inline]
    fn refresh_after_change(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        self.cache.clear();
        self.text.clear();
        self.update(None, hub, rq, context);
        self.update_tool_bar(rq, context);
        self.update_bottom_bar(rq);
    }

    pub fn set_font_size(&mut self, font_size: f32, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        if Arc::strong_count(&self._doc) > 1 { return; }
        if let Some(ref mut r) = self.info.reader { r.font_size = Some(font_size); }
        let (width, height) = context.display.dims;
        {
            let mut doc = self._doc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            doc.layout(width, height, font_size, CURRENT_DEVICE.dpi);
            if self._synthetic {
                let current_page = self.current_page.min(doc.pages_count() - 1);
                if let Some(location) = doc.resolve_location(Location::Exact(current_page)) {
                    self.current_page = location;
                }
            } else {
                let ratio = doc.pages_count() / self.pages_count;
                self.pages_count = doc.pages_count();
                self.current_page = (ratio * self.current_page).min(self.pages_count - 1);
            }
        }
        self.refresh_after_change(hub, rq, context);
    }

    pub fn set_text_align(&mut self, text_align: TextAlign, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        if Arc::strong_count(&self._doc) > 1 { return; }
        if let Some(ref mut r) = self.info.reader { r.text_align = Some(text_align); }
        {
            let mut doc = self._doc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            doc.set_text_align(text_align);
            if self._synthetic {
                let current_page = self.current_page.min(doc.pages_count() - 1);
                if let Some(location) = doc.resolve_location(Location::Exact(current_page)) {
                    self.current_page = location;
                }
            } else {
                self.pages_count = doc.pages_count();
                self.current_page = self.current_page.min(self.pages_count - 1);
            }
        }
        self.refresh_after_change(hub, rq, context);
    }

    pub fn set_font_family(&mut self, font_family: &str, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        if Arc::strong_count(&self._doc) > 1 { return; }
        if let Some(ref mut r) = self.info.reader { r.font_family = Some(font_family.to_string()); }
        {
            let mut doc = self._doc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let font_path = if font_family == DEFAULT_FONT_FAMILY { "fonts" } else { &context.settings.reader.font_path };
            doc.set_font_family(font_family, font_path);
            if self._synthetic {
                let current_page = self.current_page.min(doc.pages_count() - 1);
                if let Some(location) = doc.resolve_location(Location::Exact(current_page)) {
                    self.current_page = location;
                }
            } else {
                self.pages_count = doc.pages_count();
                self.current_page = self.current_page.min(self.pages_count - 1);
            }
        }
        self.refresh_after_change(hub, rq, context);
    }

    pub fn set_line_height(&mut self, line_height: f32, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        if Arc::strong_count(&self._doc) > 1 { return; }
        if let Some(ref mut r) = self.info.reader { r.line_height = Some(line_height); }
        {
            let mut doc = self._doc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            doc.set_line_height(line_height);
            if self._synthetic {
                let current_page = self.current_page.min(doc.pages_count() - 1);
                if let Some(location) = doc.resolve_location(Location::Exact(current_page)) {
                    self.current_page = location;
                }
            } else {
                self.pages_count = doc.pages_count();
                self.current_page = self.current_page.min(self.pages_count - 1);
            }
        }
        self.cache.clear();
        self.text.clear();
        self.update(None, hub, rq, context);
        self.update_tool_bar(rq, context);
        self.update_bottom_bar(rq);
    }

    pub fn set_margin_width(&mut self, width: i32, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        if Arc::strong_count(&self._doc) > 1 { return; }
        if let Some(ref mut r) = self.info.reader {
            if self.reflowable {
                r.margin_width = Some(width);
            } else {
                if width == 0 { r.screen_margin_width = None; } else { r.screen_margin_width = Some(width); }
            }
        }
        self.view_port.margin_width = width;
        {
            let mut doc = self._doc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            doc.set_margin_width(width);
            if self._synthetic {
                let current_page = self.current_page.min(doc.pages_count() - 1);
                if let Some(location) = doc.resolve_location(Location::Exact(current_page)) {
                    self.current_page = location;
                }
            } else {
                self.pages_count = doc.pages_count();
                self.current_page = self.current_page.min(self.pages_count - 1);
            }
        }
        self.cache.clear();
        self.text.clear();
        self.update(None, hub, rq, context);
        self.update_tool_bar(rq, context);
        self.update_bottom_bar(rq);
    }

    pub fn set_contrast_exponent(&mut self, exponent: f32, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        reader_settings::update_contrast_exponent(&mut self.info, &mut self.contrast, exponent);
        self.update(None, hub, rq, context);
        self.update_tool_bar(rq, context);
    }

    pub fn set_contrast_gray(&mut self, gray: f32, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        reader_settings::update_contrast_gray(&mut self.info, &mut self.contrast, gray);
        self.update(None, hub, rq, context);
        self.update_tool_bar(rq, context);
    }

    pub fn set_zoom_mode(&mut self, zoom_mode: ZoomMode, reset_page_offset: bool, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        if self.view_port.zoom_mode == zoom_mode { return; }
        if let Some(index) = locate_by_id(self, ViewId::TitleMenu) {
            self.child_mut(index).child_mut(1).downcast_mut::<MenuEntry>().map(|entry| entry.set_disabled(zoom_mode != ZoomMode::FitToWidth, rq));
        }
        reader_settings::update_zoom_mode(&mut self.view_port.zoom_mode, &mut self.view_port.page_offset, zoom_mode, reset_page_offset);
        self.cache.clear();
        self.update(None, hub, rq, context);
    }

    pub fn set_scroll_mode(&mut self, scroll_mode: ScrollMode, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        if self.view_port.scroll_mode == scroll_mode || self.view_port.zoom_mode != ZoomMode::FitToWidth { return; }
        reader_settings::update_scroll_mode(&mut self.view_port.scroll_mode, &mut self.view_port.page_offset, scroll_mode);
        self.update(None, hub, rq, context);
    }

    pub fn toggle_bookmark(&mut self, rq: &mut RenderQueue) {
        super::reader_annotations::toggle_bookmark(self.current_page, &mut self.info);
        self.update_tool_bar(rq, &crate::context::Context::default());
    }

    pub fn scaling_factor(rect: &Rectangle, _margin: &Margin, margin_width: i32, dims: (f32, f32), zoom_mode: ZoomMode) -> f32 {
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

    pub fn crop_margins(&mut self, index: usize, margin: &Margin, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        if self.view_port.zoom_mode != ZoomMode::FitToPage {
            let Some(Resource { pixmap, frame, .. }) = self.cache.get(&index) else { return; };
            let offset = frame.min + self.view_port.page_offset;
            let dims = {
                let doc = self._doc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                doc.dims(index).unwrap_or((0.0, 0.0))
            };
            let scale = reader_rendering::scaling_factor(&self.rect, margin, self.view_port.margin_width, dims, self.view_port.zoom_mode);
            if let Some(new_offset) = reader_rendering::calculate_margin_offset(offset, pixmap.width, pixmap.height, margin.left, margin.right, margin.top, margin.bottom, scale, dims) {
                self.view_port.page_offset = new_offset;
            }
        }
        if let Some(r) = self.info.reader.as_mut() {
            if r.cropping_margins.is_none() { r.cropping_margins = Some(CroppingMargins::Any(Margin::default())); }
            for c in r.cropping_margins.iter_mut() { *c.margin_mut(index) = margin.clone(); }
        }
        self.cache.clear();
        self.update(None, hub, rq, context);
    }
}
