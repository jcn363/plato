//! Address Bar Toggle Module
//!
//! This module handles address bar visibility and interaction for the Home view.

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::unit::scale_by_dpi;
use crate::view::home::AddressBar;
use crate::view::BIG_BAR_HEIGHT;
use crate::view::{Event, Hub, RenderData, RenderQueue, View, ViewId};

use super::super::Home;

/// Address bar toggle configuration
#[derive(Debug, Clone)]
pub struct AddressBarToggleConfig {
    pub auto_hide: bool,
    pub show_path: bool,
    pub animation_duration: u32,
}

impl Default for AddressBarToggleConfig {
    fn default() -> Self {
        Self {
            auto_hide: true,
            show_path: true,
            animation_duration: 200,
        }
    }
}

/// Address bar toggle state
#[derive(Debug, Clone)]
pub struct AddressBarToggleState {
    pub visible: bool,
    pub active: bool,
    pub config: AddressBarToggleConfig,
}

impl Default for AddressBarToggleState {
    fn default() -> Self {
        Self {
            visible: false,
            active: false,
            config: AddressBarToggleConfig::default(),
        }
    }
}

impl Home {
    /// Toggle address bar visibility
    pub fn toggle_address_bar(
        &mut self,
        enable: Option<bool>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(!self.address_bar.is_some());

        if should_enable {
            self.show_address_bar(rq, context);
        } else {
            self.hide_address_bar(rq, context);
        }
    }

    /// Show address bar
    fn show_address_bar(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.address_bar.is_some() {
            return;
        }

        let rect = self.calculate_address_bar_rect(context);
        let address_bar = AddressBar::new(rect, self.id, context);

        self.address_bar = Some(Box::new(address_bar) as Box<dyn View>);
        self.focus = Some(ViewId::AddressBar);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide address bar
    fn hide_address_bar(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.address_bar.is_none() {
            return;
        }

        self.address_bar = None;
        self.focus = None;

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Calculate address bar rectangle
    fn calculate_address_bar_rect(&self, context: &Context) -> Rectangle {
        let height = scale_by_dpi(BIG_BAR_HEIGHT, CURRENT_DEVICE.dpi) as i32;
        let width = self.rect.width() as i32;

        rect![0, 0, width, height]
    }

    /// Get address bar state
    fn get_address_bar_state(&self) -> AddressBarToggleState {
        AddressBarToggleState {
            visible: self.address_bar.is_some(),
            active: self.focus == Some(ViewId::AddressBar),
            config: AddressBarToggleConfig::default(),
        }
    }

    /// Update address bar configuration
    pub fn update_address_bar_config(&mut self, config: AddressBarToggleConfig) {
        // TODO: Implement address bar config update
        // This would require recreating the address bar if visible
    }

    /// Handle address bar events
    pub fn handle_address_bar_event(
        &mut self,
        event: &Event,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Submit(text) => {
                if self.focus == Some(ViewId::AddressBar) {
                    self.handle_address_bar_submit(text, hub, rq, context);
                    true
                } else {
                    false
                }
            }
            Event::Close(ViewId::AddressBar) => {
                self.hide_address_bar(rq, context);
                true
            }
            _ => false,
        }
    }

    /// Handle address bar submission
    fn handle_address_bar_submit(
        &mut self,
        text: &str,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        self.hide_address_bar(rq, context);
        hub.send(Event::AddressBarSubmit(text.to_string())).ok();
    }

    /// Check if address bar should auto-hide
    pub fn should_auto_hide_address_bar(&self) -> bool {
        self.get_address_bar_state().config.auto_hide
    }

    /// Get address bar animation duration
    pub fn get_address_bar_animation_duration(&self) -> u32 {
        self.get_address_bar_state().config.animation_duration
    }
}

/// Utility functions for address bar toggles
pub mod utils {
    use super::*;

    /// Create default address bar toggle config
    pub fn create_default_address_bar_config() -> AddressBarToggleConfig {
        AddressBarToggleConfig::default()
    }

    /// Format path for display in address bar
    pub fn format_path_for_display(path: &str, max_length: usize) -> String {
        if path.len() <= max_length {
            return path.to_string();
        }

        let ellipsis = "...";
        let prefix_len = (max_length - ellipsis.len()) / 2;
        let suffix_len = max_length - ellipsis.len() - prefix_len;

        if path.len() <= prefix_len + suffix_len {
            return path.to_string();
        }

        let prefix = &path[..prefix_len];
        let suffix = &path[path.len() - suffix_len..];

        format!("{}{}{}", prefix, ellipsis, suffix)
    }

    /// Validate address bar input
    pub fn validate_address_bar_input(input: &str) -> bool {
        !input.is_empty() && !input.contains(char::is_control)
    }

    /// Get address bar suggestions for input
    pub fn get_address_bar_suggestions(input: &str, context: &Context) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Add current directory suggestion
        if !input.is_empty() {
            suggestions.push(input.to_string());
        }

        // Add recent directories (simplified)
        if let Some(ref recent) = context.history.recent {
            for path in recent.iter().take(5) {
                if path.to_string_lossy().contains(input) {
                    suggestions.push(path.to_string_lossy().to_string());
                }
            }
        }

        suggestions
    }
}
