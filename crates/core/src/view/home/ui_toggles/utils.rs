//! Home UI Toggle Utilities
//!
//! Common utility functions used across UI toggle modules.

use std::path::PathBuf;

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::halves;
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
