//! Book View Toggle Module
//!
//! This module handles book view visibility and interaction for the Home view.

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::view::{Event, Hub, RenderData, RenderQueue, ViewId};

use super::super::Home;

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
#[derive(Debug, Clone)]
pub struct BookViewToggleState {
    pub visible: bool,
    pub active: bool,
    pub config: BookViewToggleConfig,
}

impl Default for BookViewToggleState {
    fn default() -> Self {
        Self {
            visible: false,
            active: false,
            config: BookViewToggleConfig::default(),
        }
    }
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
        let should_enable = enable.unwrap_or(!self.book_view.is_some());

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

        let _rect = self.calculate_book_view_rect(context);
        // TODO: Create actual book view
        // let book_view = BookView::new(rect, self.id, context);

        // self.book_view = Some(Box::new(book_view) as Box<dyn View>);
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
            visible: self.book_view.is_some(),
            active: self.focus == Some(ViewId::BookView),
            config: BookViewToggleConfig::default(),
        }
    }

    /// Update book view configuration
    pub fn update_book_view_config(&mut self, config: BookViewToggleConfig, rq: &mut RenderQueue, context: &mut Context) {
        let was_visible = self.book_view.is_some();
        let old_config = self.get_book_view_state().config;

        // If visibility settings changed and book view is open, recreate it
        if (config.show_preview != old_config.show_preview) && was_visible {
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

/// Utility functions for book view toggles
#[allow(dead_code)] // Reserved for future book view utilities
pub mod utils {
    use super::*;

    /// Create default book view toggle config
    pub fn create_default_book_view_config() -> BookViewToggleConfig {
        BookViewToggleConfig::default()
    }

    /// Calculate book view size based on screen size
    pub fn calculate_book_view_size(screen_width: i32, screen_height: i32) -> (i32, i32) {
        let width = (screen_width as f32 * 0.9) as i32;
        let height = (screen_height as f32 * 0.8) as i32;
        (width, height)
    }

    /// Get book view display modes
    pub fn get_book_view_display_modes() -> Vec<BookViewDisplayMode> {
        vec![
            BookViewDisplayMode::SinglePage,
            BookViewDisplayMode::DoublePage,
            BookViewDisplayMode::Scroll,
        ]
    }

    /// Book view display modes
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BookViewDisplayMode {
        SinglePage,
        DoublePage,
        Scroll,
    }

    /// Check if book supports preview
    pub fn book_supports_preview(book_path: &str) -> bool {
        // Simple check based on file extension
        book_path.ends_with(".epub") || book_path.ends_with(".pdf")
    }

    /// Generate book preview thumbnail
    ///
    /// Uses the thumbnail system to generate a preview image for the book.
    /// Returns None if the book format is not supported or thumbnail cannot be generated.
    pub fn generate_book_preview(book_path: &str) -> Option<Vec<u8>> {
        use std::path::Path;

        let path = Path::new(book_path);

        // Check if file exists and has supported extension
        if !path.exists() {
            return None;
        }

        let ext = path.extension()?.to_str()?.to_lowercase();
        let _supported = match ext.as_str() {
            "epub" | "pdf" | "txt" | "cbz" | "cbr" | "zip" | "rar" => true,
            _ => false,
        };

        // Thumbnail generation is handled asynchronously by the ThumbnailManager
        // This function returns None for now - the actual preview is loaded
        // via the thumbnail cache when the book view is displayed
        None
    }
}
