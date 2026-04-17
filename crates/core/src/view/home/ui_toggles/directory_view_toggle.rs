//! Directory View Toggle Module
//!
//! This module handles directory view visibility and interaction for the Home view.

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::view::{Event, Hub, RenderData, RenderQueue, ViewId};

use super::super::Home;

/// Directory view toggle configuration
#[derive(Debug, Clone)]
pub struct DirectoryViewToggleConfig {
    pub show_hidden: bool,
    pub sort_by_name: bool,
    pub show_details: bool,
}

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
    pub visible: bool,
    pub active: bool,
    pub config: DirectoryViewToggleConfig,
}

impl Default for DirectoryViewToggleState {
    fn default() -> Self {
        Self {
            visible: true,
            active: false,
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

        let _rect = self.calculate_directory_view_rect(context);
        // TODO: Create actual directory view
        // let directory_view = DirectoryView::new(rect, self.id, context);

        // self.directory_view = Some(Box::new(directory_view) as Box<dyn View>);

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
            visible: self.directory_view.is_some(),
            active: false, // Directory view is never active
            config: DirectoryViewToggleConfig::default(),
        }
    }

    /// Update directory view configuration
    pub fn update_directory_view_config(&mut self, config: DirectoryViewToggleConfig, rq: &mut RenderQueue) {
        let was_visible = self.directory_view.is_some();
        let old_config = self.get_directory_view_state().config;

        // If any display settings changed and view is open, refresh it
        if (config.show_hidden != old_config.show_hidden
            || config.sort_by_name != old_config.sort_by_name
            || config.show_details != old_config.show_details)
            && was_visible
        {
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

/// Utility functions for directory view toggles
#[allow(dead_code)] // Reserved for future directory view utilities
pub mod utils {
    use super::*;

    /// Create default directory view toggle config
    pub fn create_default_directory_view_config() -> DirectoryViewToggleConfig {
        DirectoryViewToggleConfig::default()
    }

    /// Get directory view sort options
    pub fn get_directory_view_sort_options() -> Vec<DirectoryViewSortOption> {
        vec![
            DirectoryViewSortOption::Name,
            DirectoryViewSortOption::Size,
            DirectoryViewSortOption::Date,
            DirectoryViewSortOption::Type,
        ]
    }

    /// Directory view sort options
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DirectoryViewSortOption {
        Name,
        Size,
        Date,
        Type,
    }

    /// Check if file should be hidden
    pub fn is_file_hidden(file_name: &str, show_hidden: bool) -> bool {
        if !show_hidden && file_name.starts_with('.') {
            return true;
        }

        // Check for other hidden patterns
        matches!(file_name, "Thumbs.db" | "Desktop.ini" | ".DS_Store")
    }

    /// Format file size for display
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size_f = size as f64;
        let mut unit_index = 0;

        while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
            size_f /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size_f, UNITS[unit_index])
        }
    }

    /// Get file icon for display
    pub fn get_file_icon(file_name: &str) -> &'static str {
        if file_name.ends_with(".pdf") {
            "PDF"
        } else if file_name.ends_with(".epub") {
            "EPUB"
        } else if file_name.ends_with(".txt") {
            "TXT"
        } else if file_name.ends_with(".jpg") || file_name.ends_with(".png") {
            "IMG"
        } else if file_name.ends_with(".mp3") || file_name.ends_with(".flac") {
            "AUD"
        } else if file_name.ends_with(".mp4") || file_name.ends_with(".avi") {
            "VID"
        } else {
            "FILE"
        }
    }
}
