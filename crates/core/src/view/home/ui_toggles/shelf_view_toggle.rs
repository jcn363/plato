//! Shelf View Toggle Module
//!
//! This module handles shelf view visibility and interaction for the Home view.

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::settings::{FirstColumn, SecondColumn};
use crate::view::home::Shelf;
use crate::view::{Event, Hub, RenderData, RenderQueue, View, ViewId};

use super::super::Home;

/// Shelf view toggle configuration
#[derive(Debug, Clone)]
pub struct ShelfViewToggleConfig {
    pub auto_refresh: bool,
    pub show_metadata: bool,
    pub grid_columns: u8,
}

impl Default for ShelfViewToggleConfig {
    fn default() -> Self {
        Self {
            auto_refresh: true,
            show_metadata: true,
            grid_columns: 3,
        }
    }
}

/// Shelf view toggle state
#[derive(Debug, Clone)]
pub struct ShelfViewToggleState {
    pub _visible: bool,
    pub _active: bool,
    pub config: ShelfViewToggleConfig,
}

impl Default for ShelfViewToggleState {
    fn default() -> Self {
        Self {
            _visible: false,
            _active: false,
            config: ShelfViewToggleConfig::default(),
        }
    }
}

impl Home {
    /// Toggle shelf view visibility
    pub fn toggle_shelf_view(
        &mut self,
        enable: Option<bool>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(!self.shelf.is_some());

        if should_enable {
            self.show_shelf_view(rq, context);
        } else {
            self.hide_shelf_view(rq, context);
        }
    }

    /// Show shelf view
    fn show_shelf_view(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.shelf.is_some() {
            return;
        }

        let rect = self.calculate_shelf_view_rect(context);
        let shelf = Shelf::new(
            rect,
            FirstColumn::TitleAndAuthor,
            SecondColumn::Progress,
            false,
        );

        self.shelf = Some(Box::new(shelf) as Box<dyn View>);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide shelf view
    fn hide_shelf_view(&mut self, rq: &mut RenderQueue, _context: &mut Context) {
        if self.shelf.is_none() {
            return;
        }

        self.shelf = None;

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Calculate shelf view rectangle
    fn calculate_shelf_view_rect(&self, _context: &Context) -> Rectangle {
        let top_offset = self.calculate_top_offset();
        let bottom_offset = self.calculate_bottom_offset();

        rect![
            0,
            top_offset,
            self.rect.width() as i32,
            self.rect.height() as i32 - top_offset - bottom_offset
        ]
    }

    /// Get shelf view state
    fn get_shelf_view_state(&self) -> ShelfViewToggleState {
        ShelfViewToggleState {
            _visible: self.shelf.is_some(),
            _active: false, // Shelf view is never active
            config: ShelfViewToggleConfig::default(),
        }
    }

    /// Update shelf view configuration
    pub fn update_shelf_view_config(
        &mut self,
        config: ShelfViewToggleConfig,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let was_visible = self.shelf.is_some();
        let old_config = self.get_shelf_view_state().config;

        // If display settings changed and shelf is visible, recreate it
        if (config.show_metadata != old_config.show_metadata
            || config.grid_columns != old_config.grid_columns)
            && was_visible
        {
            // Recreate shelf with new configuration
            self.hide_shelf_view(rq, context);
            self.show_shelf_view(rq, context);
        }

        // Trigger refresh to reflect new settings
        if was_visible {
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    /// Handle shelf view events
    pub fn handle_shelf_view_event(
        &mut self,
        event: &Event,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Close(ViewId::Shelf(_)) => {
                self.hide_shelf_view(rq, context);
                true
            }
            _ => false,
        }
    }

    /// Check if shelf view should auto-refresh
    pub fn should_auto_refresh_shelf_view(&self) -> bool {
        self.get_shelf_view_state().config.auto_refresh
    }

    /// Get shelf view grid columns
    pub fn get_shelf_view_grid_columns(&self) -> u8 {
        self.get_shelf_view_state().config.grid_columns
    }

    /// Update shelf view content
    pub fn update_shelf_view_content(&mut self, rq: &mut RenderQueue) {
        if self.shelf.is_some() {
            // Refresh shelf content based on current directory
            // This would scan the current directory and update book display
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }
}

/// Utility functions for shelf view toggles
pub mod utils {
    use super::*;

    /// Create default shelf view toggle config
    pub fn create_default_shelf_view_config() -> ShelfViewToggleConfig {
        ShelfViewToggleConfig::default()
    }

    /// Calculate optimal grid columns for screen size
    pub fn calculate_optimal_grid_columns(screen_width: i32, item_width: i32) -> u8 {
        let max_columns = (screen_width / item_width) as u8;
        max_columns.min(5).max(2)
    }

    /// Calculate shelf item size based on grid columns
    pub fn calculate_shelf_item_size(grid_columns: u8, available_width: i32) -> (i32, i32) {
        let padding = 20; // Padding between items
        let total_padding = padding * (grid_columns as i32 - 1);
        let item_width = (available_width - total_padding) / grid_columns as i32;
        let item_height = (item_width as f32 * 1.4) as i32; // Book aspect ratio

        (item_width, item_height)
    }

    /// Shelf layout options
    #[derive(Debug, Clone)]
    pub enum ShelfLayoutOption {
        Grid { columns: u8 },
        List,
    }
}
