//! Search Bar Toggle Module
//!
//! This module handles search bar visibility and interaction for the Home view.

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::unit::scale_by_dpi;
use crate::view::search_bar::SearchBar;
use crate::view::BIG_BAR_HEIGHT;
use crate::view::{Event, Hub, RenderData, RenderQueue, View, ViewId};

use super::super::Home;

/// Search bar toggle configuration
#[derive(Debug, Clone)]
pub struct SearchBarToggleConfig {
    pub auto_hide: bool,
    pub show_history: bool,
    pub animation_duration: u32,
}

impl Default for SearchBarToggleConfig {
    fn default() -> Self {
        Self {
            auto_hide: true,
            show_history: true,
            animation_duration: 200,
        }
    }
}

/// Search bar toggle state
#[derive(Debug, Clone)]
pub struct SearchBarToggleState {
    pub visible: bool,
    pub active: bool,
    pub config: SearchBarToggleConfig,
}

impl Default for SearchBarToggleState {
    fn default() -> Self {
        Self {
            visible: false,
            active: false,
            config: SearchBarToggleConfig::default(),
        }
    }
}

impl Home {
    /// Toggle search bar visibility
    pub fn toggle_search_bar(
        &mut self,
        enable: Option<bool>,
        _update: bool,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(!self.search_bar.is_some());

        if should_enable {
            self.show_search_bar(rq, context);
        } else {
            self.hide_search_bar(rq, context);
        }
    }

    /// Show search bar
    fn show_search_bar(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.search_bar.is_some() {
            return;
        }

        let rect = self.calculate_search_bar_rect(context);
        let search_bar = SearchBar::new(rect, self.id, context);

        self.search_bar = Some(Box::new(search_bar) as Box<dyn View>);
        self.focus = Some(ViewId::SearchBar);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide search bar
    fn hide_search_bar(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.search_bar.is_none() {
            return;
        }

        self.search_bar = None;
        self.focus = None;

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Calculate search bar rectangle
    fn calculate_search_bar_rect(&self, context: &Context) -> Rectangle {
        let height = scale_by_dpi(BIG_BAR_HEIGHT, CURRENT_DEVICE.dpi) as i32;
        let width = self.rect.width() as i32;

        rect![0, 0, width, height]
    }

    /// Get search bar state
    fn get_search_bar_state(&self) -> SearchBarToggleState {
        SearchBarToggleState {
            visible: self.search_bar.is_some(),
            active: self.focus == Some(ViewId::SearchBar),
            config: SearchBarToggleConfig::default(),
        }
    }

    /// Update search bar configuration
    pub fn update_search_bar_config(&mut self, config: SearchBarToggleConfig) {
        // TODO: Implement search bar config update
        // This would require recreating the search bar if visible
    }

    /// Handle search bar events
    pub fn handle_search_bar_event(
        &mut self,
        event: &Event,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Submit(text) => {
                if self.focus == Some(ViewId::SearchBar) {
                    self.handle_search_bar_submit(text, hub, rq, context);
                    true
                } else {
                    false
                }
            }
            Event::Close(ViewId::SearchBar) => {
                self.hide_search_bar(rq, context);
                true
            }
            _ => false,
        }
    }

    /// Handle search bar submission
    fn handle_search_bar_submit(
        &mut self,
        text: &str,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        self.hide_search_bar(rq, context);
        hub.send(Event::SearchBarSubmit(text.to_string())).ok();
    }

    /// Check if search bar should auto-hide
    pub fn should_auto_hide_search_bar(&self) -> bool {
        self.get_search_bar_state().config.auto_hide
    }

    /// Get search bar animation duration
    pub fn get_search_bar_animation_duration(&self) -> u32 {
        self.get_search_bar_state().config.animation_duration
    }
}

/// Utility functions for search bar toggles
pub mod utils {
    use super::*;

    /// Create default search bar toggle config
    pub fn create_default_search_bar_config() -> SearchBarToggleConfig {
        SearchBarToggleConfig::default()
    }

    /// Format search query for display
    pub fn format_search_query(query: &str, max_length: usize) -> String {
        if query.len() <= max_length {
            return query.to_string();
        }

        let ellipsis = "...";
        let prefix_len = (max_length - ellipsis.len()) / 2;
        let suffix_len = max_length - ellipsis.len() - prefix_len;

        if query.len() <= prefix_len + suffix_len {
            return query.to_string();
        }

        let prefix = &query[..prefix_len];
        let suffix = &query[query.len() - suffix_len..];

        format!("{}{}{}", prefix, ellipsis, suffix)
    }

    /// Validate search query
    pub fn validate_search_query(query: &str) -> bool {
        !query.is_empty() && !query.contains(char::is_control)
    }

    /// Get search suggestions for input
    pub fn get_search_suggestions(input: &str, context: &Context) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Add current query as suggestion
        if !input.is_empty() {
            suggestions.push(input.to_string());
        }

        // Add recent searches (simplified)
        if let Some(ref recent) = context.history.recent {
            for path in recent.iter().take(5) {
                let path_str = path.to_string_lossy();
                if path_str.to_lowercase().contains(&input.to_lowercase()) {
                    suggestions.push(path_str.to_string());
                }
            }
        }

        suggestions
    }

    /// Check if search should be triggered
    pub fn should_trigger_search(input: &str, min_length: usize) -> bool {
        input.len() >= min_length
    }

    /// Get search result highlighting
    pub fn get_search_highlighting(text: &str, query: &str) -> Vec<(usize, usize)> {
        let mut highlights = Vec::new();
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();

        let mut start = 0;
        while let Some(pos) = text_lower[start..].find(&query_lower) {
            let absolute_pos = start + pos;
            highlights.push((absolute_pos, absolute_pos + query.len()));
            start = absolute_pos + query.len();
        }

        highlights
    }
}
