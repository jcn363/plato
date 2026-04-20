//! Directory View Toggle Module
//!
//! This module handles directory view visibility and interaction for the Home view.

use crate::context::Context;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;
use crate::impl_view_boilerplate;
use crate::view::{Bus, Event, Hub, Id, RenderData, RenderQueue, View, ViewId, ID_FEEDER};

use super::super::Home;

/// Directory view display component
///
/// Shows directory contents and navigation in a dedicated view panel.
pub struct DirectoryView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    current_path: Option<std::path::PathBuf>,
    _parent_id: Id,
    _show_hidden: bool,
}

impl DirectoryView {
    /// Create a new directory view
    pub fn new(rect: Rectangle, parent_id: Id, _context: &mut Context) -> Self {
        Self {
            id: ID_FEEDER.next(),
            rect,
            children: Vec::new(),
            current_path: None,
            _parent_id: parent_id,
            _show_hidden: false,
        }
    }
}

impl View for DirectoryView {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        _bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match evt {
            Event::Close(ViewId::DirectoryView) => {
                hub.send(Event::Close(ViewId::DirectoryView)).ok();
                true
            }
            Event::Gesture(crate::gesture::GestureEvent::Tap(center))
                if !self.rect.includes(*center) =>
            {
                // Close when tapping outside
                hub.send(Event::Close(ViewId::DirectoryView)).ok();
                true
            }
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, _rect: Rectangle, fonts: &mut crate::font::Fonts) {
        use crate::color::{background, text_normal};
        use crate::geom::{BorderSpec, CornerSpec};
        use crate::unit::scale_by_dpi;
        use crate::view::rendering::THICKNESS_MEDIUM;

        let dpi = crate::unit::get_device_dpi();
        let dark = crate::theme::is_dark_mode();
        let bg_color = background(dark);
        let fg_color = text_normal(dark);

        // Draw background with border
        let border_thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as u16;
        fb.draw_rounded_rectangle_with_border(
            &self.rect,
            &CornerSpec::Uniform(0),
            &BorderSpec {
                thickness: border_thickness,
                color: fg_color[0],
            },
            &bg_color,
        );

        // Draw placeholder text if no directory is set
        if self.current_path.is_none() {
            let text = "No directory selected";
            let font = crate::font::font_from_style(fonts, &crate::font::NORMAL_STYLE, dpi);
            let plan = font.plan(text, None, None);
            let x = self.rect.min.x + (self.rect.width() as i32 - plan.width) / 2;
            let y = self.rect.min.y + (self.rect.height() as i32) / 2;
            font.render(fb, fg_color[1], &plan, crate::geom::Point::new(x, y));
        }
    }

    impl_view_boilerplate!();

    fn view_id(&self) -> Option<ViewId> {
        Some(ViewId::DirectoryView)
    }
}

/// Directory view toggle configuration
#[derive(Debug, Clone)]
pub struct DirectoryViewToggleConfig {
    pub show_hidden: bool,
    pub sort_by_name: bool,
    pub show_details: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for DirectoryViewToggleConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            sort_by_name: true,
            show_details: false,
        }
    }
}

/// Directory view toggle state
#[derive(Debug, Clone)]
pub struct DirectoryViewToggleState {
    pub _visible: bool,
    pub _active: bool,
    pub config: DirectoryViewToggleConfig,
}
#[allow(clippy::derivable_impls)]
impl Default for DirectoryViewToggleState {
    fn default() -> Self {
        Self {
            _visible: false,
            _active: false,
            config: DirectoryViewToggleConfig::default(),
        }
    }
}

impl Home {
    /// Toggle directory view visibility
    pub fn toggle_directory_view(
        &mut self,
        enable: Option<bool>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(!self.directory_view.is_some());

        if should_enable {
            self.show_directory_view(rq, context);
        } else {
            self.hide_directory_view(rq, context);
        }
    }

    /// Show directory view
    fn show_directory_view(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.directory_view.is_some() {
            return;
        }

        let rect = self.calculate_directory_view_rect(context);
        let directory_view = DirectoryView::new(rect, self.id, context);

        self.directory_view = Some(Box::new(directory_view) as Box<dyn View>);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide directory view
    fn hide_directory_view(&mut self, rq: &mut RenderQueue, _context: &mut Context) {
        if self.directory_view.is_none() {
            return;
        }

        self.directory_view = None;

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Calculate directory view rectangle
    fn calculate_directory_view_rect(&self, _context: &Context) -> Rectangle {
        let top_offset = self.calculate_top_offset();
        let bottom_offset = self.calculate_bottom_offset();

        rect![
            0,
            top_offset,
            self.rect.width() as i32,
            self.rect.height() as i32 - top_offset - bottom_offset
        ]
    }

    /// Get directory view state
    fn get_directory_view_state(&self) -> DirectoryViewToggleState {
        DirectoryViewToggleState {
            _visible: self.directory_view.is_some(),
            _active: false, // Directory view is never active
            config: DirectoryViewToggleConfig::default(),
        }
    }

    /// Update directory view configuration
    pub fn update_directory_view_config(
        &mut self,
        config: DirectoryViewToggleConfig,
        rq: &mut RenderQueue,
    ) {
        let was_visible = self.directory_view.is_some();

        // Check if settings changed before any mutable borrows
        let should_refresh = if was_visible {
            let old_config = &self.get_directory_view_state().config;
            config.show_hidden != old_config.show_hidden
                || config.sort_by_name != old_config.sort_by_name
                || config.show_details != old_config.show_details
        } else {
            false
        };

        // If any display settings changed and view is open, refresh it
        if should_refresh {
            // Refresh the directory view with new settings
            self.update_directory_view_content(rq);
        }

        // Trigger refresh to reflect new settings
        if was_visible {
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    /// Handle directory view events
    pub fn handle_directory_view_event(
        &mut self,
        event: &Event,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Close(ViewId::DirectoryView) => {
                self.hide_directory_view(rq, context);
                true
            }
            _ => false,
        }
    }

    /// Check if directory view should show hidden files
    pub fn should_show_hidden_files(&self) -> bool {
        self.get_directory_view_state().config.show_hidden
    }

    /// Check if directory view should sort by name
    pub fn should_sort_by_name(&self) -> bool {
        self.get_directory_view_state().config.sort_by_name
    }

    /// Check if directory view should show details
    pub fn should_show_details(&self) -> bool {
        self.get_directory_view_state().config.show_details
    }

    /// Update directory view content
    pub fn update_directory_view_content(&mut self, rq: &mut RenderQueue) {
        if self.directory_view.is_some() {
            // Refresh the directory listing with current settings
            // This would scan the current directory and update the view
            // based on show_hidden, sort_by_name, and show_details settings
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }
}
