//! Go To Page Toggle Module
//!
//! This module handles go-to-page dialog visibility and interaction for the Home view.

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::unit::scale_by_dpi;
use crate::view::named_input::NamedInput;
use crate::view::BIG_BAR_HEIGHT;
use crate::view::{Event, Hub, RenderData, RenderQueue, View, ViewId};

use super::super::Home;

/// Go to page toggle configuration
#[derive(Debug, Clone)]
pub struct GoToPageToggleConfig {
    pub auto_hide: bool,
    pub show_page_count: bool,
    pub animation_duration: u32,
}

impl Default for GoToPageToggleConfig {
    fn default() -> Self {
        Self {
            auto_hide: true,
            show_page_count: true,
            animation_duration: 200,
        }
    }
}

/// Go to page toggle state
#[derive(Debug, Clone)]
pub struct GoToPageToggleState {
    pub _visible: bool,
    pub _active: bool,
    pub config: GoToPageToggleConfig,
}

impl Default for GoToPageToggleState {
    fn default() -> Self {
        Self {
            _visible: false,
            _active: false,
            config: GoToPageToggleConfig::default(),
        }
    }
}

impl Home {
    /// Toggle go-to-page dialog visibility
    pub fn toggle_go_to_page(
        &mut self,
        enable: Option<bool>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(!self.go_to_page.is_some());

        if should_enable {
            self.show_go_to_page(rq, context);
        } else {
            self.hide_go_to_page(rq, context);
        }
    }

    /// Show go-to-page dialog
    fn show_go_to_page(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.go_to_page.is_some() {
            return;
        }

        let _rect = self.calculate_go_to_page_rect(context);
        let go_to_page = NamedInput::new(
            "Go to Page".to_string(),
            ViewId::GoToPageInput,
            ViewId::GoToPageInput,
            5,
            context,
        );

        self.go_to_page = Some(Box::new(go_to_page) as Box<dyn View>);
        self.focus = Some(ViewId::GoToPage);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide go-to-page dialog
    fn hide_go_to_page(&mut self, rq: &mut RenderQueue, _context: &mut Context) {
        if self.go_to_page.is_none() {
            return;
        }

        self.go_to_page = None;
        self.focus = None;

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Calculate go-to-page dialog rectangle
    fn calculate_go_to_page_rect(&self, context: &Context) -> Rectangle {
        let screen_width = context.display.dims.0 as i32;
        let screen_height = context.display.dims.1 as i32;

        let width = (screen_width as f32 * 0.6) as i32;
        let height = scale_by_dpi(BIG_BAR_HEIGHT, CURRENT_DEVICE.dpi) as i32;
        let x = (screen_width - width) / 2;
        let y = (screen_height - height) / 2;

        rect![x, y, width, height]
    }

    /// Get go-to-page state
    fn get_go_to_page_state(&self) -> GoToPageToggleState {
        GoToPageToggleState {
            _visible: self.go_to_page.is_some(),
            _active: self.focus == Some(ViewId::GoToPage),
            config: GoToPageToggleConfig::default(),
        }
    }

    /// Update go-to-page configuration
    pub fn update_go_to_page_config(&mut self, config: GoToPageToggleConfig, rq: &mut RenderQueue) {
        let was_visible = self.go_to_page.is_some();
        let old_config = self.get_go_to_page_state().config;

        // If page count display setting changed and dialog is visible, refresh it
        if config.show_page_count != old_config.show_page_count && was_visible {
            // Refresh go-to-page dialog to show/hide page count
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }

        // Trigger refresh to reflect new settings
        if was_visible {
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    /// Handle go-to-page events
    pub fn handle_go_to_page_event(
        &mut self,
        event: &Event,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Submit(ViewId::GoToPageInput, text) => {
                self.handle_go_to_page_submit(&text, hub, rq, context);
                true
            }
            Event::Close(ViewId::GoToPage) => {
                self.hide_go_to_page(rq, context);
                true
            }
            _ => false,
        }
    }

    /// Handle go-to-page submission
    fn handle_go_to_page_submit(
        &mut self,
        text: &str,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Ok(page_num) = text.parse::<usize>() {
            let page_index = page_num.saturating_sub(1); // Convert to 0-based index
            self.go_to_page(page_index, hub, rq, context);
        }

        self.hide_go_to_page(rq, context);
    }

    /// Check if go-to-page should auto-hide
    pub fn should_auto_hide_go_to_page(&self) -> bool {
        self.get_go_to_page_state().config.auto_hide
    }

    /// Get go-to-page animation duration
    pub fn get_go_to_page_animation_duration(&self) -> u32 {
        self.get_go_to_page_state().config.animation_duration
    }
}

/// Utility functions for go-to-page toggles
pub mod utils {
    use super::*;

    /// Create default go-to-page toggle config
    pub fn create_default_go_to_page_config() -> GoToPageToggleConfig {
        GoToPageToggleConfig::default()
    }

    /// Format page number for display
    pub fn format_page_number(page: usize, total_pages: usize) -> String {
        if total_pages == 0 {
            "Page 1 of 1".to_string()
        } else {
            format!("Page {} of {}", page + 1, total_pages)
        }
    }

    /// Get page number suggestions
    pub fn get_page_number_suggestions(current_page: usize, total_pages: usize) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Add current page
        suggestions.push((current_page + 1).to_string());

        // Add first and last pages
        if current_page != 0 {
            suggestions.push("1".to_string());
        }
        if current_page != total_pages - 1 {
            suggestions.push(total_pages.to_string());
        }

        // Add nearby pages
        for offset in [-5, -1, 1, 5].iter() {
            let suggested_page = current_page as isize + *offset;
            if suggested_page > 0 && suggested_page < total_pages as isize {
                suggestions.push(suggested_page.to_string());
            }
        }

        suggestions.sort();
        suggestions.dedup();
        suggestions
    }

    /// Calculate optimal page jump size
    pub fn calculate_page_jump_size(total_pages: usize) -> usize {
        match total_pages {
            0..=10 => 1,
            11..=50 => 5,
            51..=200 => 10,
            201..=1000 => 25,
            _ => 50,
        }
    }
}
