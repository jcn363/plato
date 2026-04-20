//! Book View Toggle Module
//!
//! This module handles book view visibility and interaction for the Home view.

use crate::context::Context;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;
use crate::impl_view_boilerplate;
use crate::view::{Bus, Event, Hub, Id, RenderData, RenderQueue, View, ViewId, ID_FEEDER};

use super::super::Home;

/// Book view display component
///
/// Shows book details and preview in a dedicated view panel.
pub struct BookView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    book_path: Option<std::path::PathBuf>,
}

impl BookView {
    /// Create a new book view
    pub fn new(rect: Rectangle, _parent_id: Id, _context: &mut Context) -> Self {
        Self {
            id: ID_FEEDER.next(),
            rect,
            children: Vec::new(),
            book_path: None,
        }
    }
}

impl View for BookView {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        _bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match evt {
            Event::Close(ViewId::BookView) => {
                hub.send(Event::Close(ViewId::BookView)).ok();
                true
            }
            Event::Gesture(crate::gesture::GestureEvent::Tap(center))
                if !self.rect.includes(*center) =>
            {
                // Close when tapping outside
                hub.send(Event::Close(ViewId::BookView)).ok();
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

        // Draw placeholder text if no book is set
        if self.book_path.is_none() {
            let text = "No book selected";
            let font = crate::font::font_from_style(fonts, &crate::font::NORMAL_STYLE, dpi);
            let plan = font.plan(text, None, None);
            let x = self.rect.min.x + (self.rect.width() as i32 - plan.width) / 2;
            let y = self.rect.min.y + (self.rect.height() as i32) / 2;
            font.render(fb, fg_color[1], &plan, crate::geom::Point::new(x, y));
        }
    }

    impl_view_boilerplate!();

    fn view_id(&self) -> Option<ViewId> {
        Some(ViewId::BookView)
    }
}

/// Book view toggle configuration
#[derive(Debug, Clone)]
pub struct BookViewToggleConfig {
    pub auto_open: bool,
    pub show_preview: bool,
    pub animation_duration: u32,
}

impl Default for BookViewToggleConfig {
    fn default() -> Self {
        Self {
            auto_open: true,
            show_preview: true,
            animation_duration: 300,
        }
    }
}

/// Book view toggle state
#[derive(Debug, Clone, Default)]
pub struct BookViewToggleState {
    pub _visible: bool,
    pub _active: bool,
    pub config: BookViewToggleConfig,
}

impl Home {
    /// Toggle book view visibility
    pub fn toggle_book_view(
        &mut self,
        enable: Option<bool>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(self.book_view.is_none());

        if should_enable {
            self.show_book_view(rq, context);
        } else {
            self.hide_book_view(rq, context);
        }
    }

    /// Show book view
    fn show_book_view(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.book_view.is_some() {
            return;
        }

        let rect = self.calculate_book_view_rect(context);
        let book_view = BookView::new(rect, self.id, context);

        self.book_view = Some(Box::new(book_view) as Box<dyn View>);
        self.focus = Some(ViewId::BookView);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide book view
    fn hide_book_view(&mut self, rq: &mut RenderQueue, _context: &mut Context) {
        if self.book_view.is_none() {
            return;
        }

        self.book_view = None;
        self.focus = None;

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Calculate book view rectangle
    fn calculate_book_view_rect(&self, _context: &Context) -> Rectangle {
        let top_offset = self.calculate_top_offset();
        let bottom_offset = self.calculate_bottom_offset();

        rect![
            0,
            top_offset,
            self.rect.width() as i32,
            self.rect.height() as i32 - top_offset - bottom_offset
        ]
    }

    /// Get book view state
    fn get_book_view_state(&self) -> BookViewToggleState {
        BookViewToggleState {
            _visible: self.book_view.is_some(),
            _active: self.focus == Some(ViewId::BookView),
            config: BookViewToggleConfig::default(),
        }
    }

    /// Update book view configuration
    pub fn update_book_view_config(
        &mut self,
        config: BookViewToggleConfig,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let was_visible = self.book_view.is_some();

        // Check if settings changed before any mutable borrows
        let should_recreate = if was_visible {
            let old_config = &self.get_book_view_state().config;
            config.show_preview != old_config.show_preview
        } else {
            false
        };

        // If visibility settings changed and book view is open, recreate it
        if should_recreate {
            self.hide_book_view(rq, context);
            // Book view will be recreated on next show with new config
        }

        // Trigger refresh to reflect new settings
        if was_visible {
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    /// Handle book view events
    pub fn handle_book_view_event(
        &mut self,
        event: &Event,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Close(ViewId::BookView) => {
                self.hide_book_view(rq, context);
                true
            }
            _ => false,
        }
    }

    /// Check if book view should auto-open
    pub fn should_auto_open_book_view(&self) -> bool {
        self.get_book_view_state().config.auto_open
    }

    /// Get book view animation duration
    pub fn get_book_view_animation_duration(&self) -> u32 {
        self.get_book_view_state().config.animation_duration
    }

    /// Open book in book view
    pub fn open_book_in_view(
        &mut self,
        book_path: &str,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        use std::path::Path;

        let path = Path::new(book_path);

        // Validate the book exists
        if !path.exists() {
            return;
        }

        // Check if we have this book in our library
        // Library uses fingerprint-based lookup: paths -> fingerprint -> info
        let book_in_library = context
            .library
            .paths
            .get(path)
            .and_then(|fp| context.library.db.get(fp));

        if book_in_library.is_some() {
            // Book found in library, show book view with metadata
            self.show_book_view(rq, context);

            // If auto-open is enabled and we have a valid book, trigger open event
            if self.should_auto_open_book_view() {
                // The actual opening is handled by the main event loop
                // We just prepare the book view here
            }
        } else {
            // Book not in library, just show empty book view
            self.show_book_view(rq, context);
        }
    }
}
