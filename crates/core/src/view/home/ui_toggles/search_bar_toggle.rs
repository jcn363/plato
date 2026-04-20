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
#[derive(Debug, Clone, Default)]
pub struct SearchBarToggleState {
    pub _visible: bool,
    pub _active: bool,
    pub config: SearchBarToggleConfig,
}

impl Home {
    /// Toggle search bar visibility
    pub fn toggle_search_bar(
        &mut self,
        enable: Option<bool>,
        _update: bool,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(self.search_bar.is_none());

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
        let search_bar = SearchBar::new(rect, ViewId::SearchBar, "Search...", "", context);

        self.search_bar = Some(Box::new(search_bar) as Box<dyn View>);
        self.focus = Some(ViewId::SearchBar);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide search bar
    fn hide_search_bar(&mut self, rq: &mut RenderQueue, _context: &mut Context) {
        if self.search_bar.is_none() {
            return;
        }

        self.search_bar = None;
        self.focus = None;

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Calculate search bar rectangle
    fn calculate_search_bar_rect(&self, _context: &Context) -> Rectangle {
        let height = scale_by_dpi(BIG_BAR_HEIGHT, CURRENT_DEVICE.dpi) as i32;
        let width = self.rect.width() as i32;

        rect![0, 0, width, height]
    }

    /// Get search bar state
    fn get_search_bar_state(&self) -> SearchBarToggleState {
        SearchBarToggleState {
            _visible: self.search_bar.is_some(),
            _active: self.focus == Some(ViewId::SearchBar),
            config: SearchBarToggleConfig::default(),
        }
    }

    /// Update search bar configuration
    pub fn update_search_bar_config(
        &mut self,
        config: SearchBarToggleConfig,
        rq: &mut RenderQueue,
    ) {
        let was_visible = self.search_bar.is_some();
        let old_config = self.get_search_bar_state().config;

        // If history display setting changed and search bar is visible, refresh it
        if config.show_history != old_config.show_history && was_visible {
            // Refresh search bar to show/hide history
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }

        // Trigger refresh to reflect new settings
        if was_visible {
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
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
            Event::Submit(ViewId::SearchBar, text) => {
                self.handle_search_bar_submit(text, hub, rq, context);
                true
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
