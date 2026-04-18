//! Home UI Toggle Utilities
//!
//! Common utility functions used across UI toggle modules.

use std::path::PathBuf;

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::halves;
use crate::geom::Rectangle;
use crate::unit::scale_by_dpi;
use crate::view::{
    EntryId, Event, Hub, RenderData, RenderQueue, ViewId, SMALL_BAR_HEIGHT, THICKNESS_MEDIUM,
};

use super::super::Home;

impl Home {
    /// Calculate top offset for view positioning based on visible UI elements
    pub fn calculate_top_offset(&self) -> i32 {
        let dpi = crate::unit::get_device_dpi();
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, _) = halves(thickness);

        let mut offset = small_height + small_thickness;

        // Account for address bar if visible
        if self.address_bar.is_some() {
            offset += small_height - thickness;
            offset += thickness; // separator
        }

        // Account for navigation bar if visible
        if self.navigation_bar.is_some() {
            offset += small_height - thickness;
            offset += thickness; // separator
        }

        offset
    }

    /// Calculate bottom offset for view positioning based on visible UI elements  
    pub fn calculate_bottom_offset(&self) -> i32 {
        let dpi = crate::unit::get_device_dpi();
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, _) = halves(thickness);

        // Bottom bar is always present
        small_height - small_thickness
    }

    /// Toggle rename document dialog
    pub fn toggle_rename_document(
        &mut self,
        enable: Option<bool>,
        hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        // Check if rename dialog should be shown
        let should_enable = enable.unwrap_or(false);

        if should_enable {
            // Send event to show rename dialog
            hub.send(Event::Show(ViewId::RenameDocument)).ok();
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        } else {
            // Close rename dialog
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    /// Toggle select directory dialog
    pub fn toggle_select_directory(
        &mut self,
        path: &PathBuf,
        hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        // Send event to toggle select directory
        hub.send(Event::Select(EntryId::ToggleSelectDirectory(path.clone())))
            .ok();
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }
}

/// Check if keyboard is currently visible
pub fn is_keyboard_visible(home: &Home) -> bool {
    home.keyboard.is_some()
}

/// Check if search bar is currently visible
pub fn is_search_bar_visible(home: &Home) -> bool {
    home.search_bar.is_some()
}

/// Get current focus view ID if any
pub fn current_focus(home: &Home) -> Option<crate::view::ViewId> {
    home.focus
}

/// Update focus to a specific view
pub fn set_focus(home: &mut Home, view_id: Option<crate::view::ViewId>) {
    home.focus = view_id;
}

/// Clear all menus and popups
pub fn clear_menus(home: &mut Home) {
    home.sort_menu = None;
    home.book_menu = None;
    home.library_menu = None;
    home.settings_menu = None;
}

/// Check if any menu is currently open
pub fn is_any_menu_open(home: &Home) -> bool {
    home.sort_menu.is_some()
        || home.book_menu.is_some()
        || home.library_menu.is_some()
        || home.settings_menu.is_some()
}

/// Get available screen height for content area
pub fn get_content_height(rect: &Rectangle, top_offset: i32, bottom_offset: i32) -> i32 {
    rect.height() as i32 - top_offset - bottom_offset
}

/// Get available screen width for content area
pub fn get_content_width(rect: &Rectangle) -> i32 {
    rect.width() as i32
}
