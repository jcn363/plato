//! Navigation Bar Toggle Module
//!
//! This module handles navigation bar visibility and interaction for the Home view.

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::unit::scale_by_dpi;
use crate::view::home::NavigationBar;
use crate::view::{Event, Hub, RenderData, RenderQueue, View, ViewId};
use crate::view::SMALL_BAR_HEIGHT;

use super::super::Home;

/// Navigation bar toggle configuration
#[derive(Debug, Clone)]
pub struct NavigationBarToggleConfig {
    pub auto_hide: bool,
    pub show_breadcrumbs: bool,
    pub animation_duration: u32,
}

impl Default for NavigationBarToggleConfig {
    fn default() -> Self {
        Self {
            auto_hide: false,
            show_breadcrumbs: true,
            animation_duration: 200,
        }
    }
}

/// Navigation bar toggle state
#[derive(Debug, Clone)]
pub struct NavigationBarToggleState {
    pub visible: bool,
    pub active: bool,
    pub config: NavigationBarToggleConfig,
}

impl Default for NavigationBarToggleState {
    fn default() -> Self {
        Self {
            visible: true,
            active: false,
            config: NavigationBarToggleConfig::default(),
        }
    }
}

impl Home {
    /// Toggle navigation bar visibility
    pub fn toggle_navigation_bar(
        &mut self,
        enable: Option<bool>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(!self.navigation_bar.is_some());
        
        if should_enable {
            self.show_navigation_bar(rq, context);
        } else {
            self.hide_navigation_bar(rq, context);
        }
    }

    /// Show navigation bar
    fn show_navigation_bar(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.navigation_bar.is_some() {
            return;
        }

        let rect = self.calculate_navigation_bar_rect(context);
        let navigation_bar = NavigationBar::new(rect, self.id, context);
        
        self.navigation_bar = Some(Box::new(navigation_bar) as Box<dyn View>);
        
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide navigation bar
    fn hide_navigation_bar(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.navigation_bar.is_none() {
            return;
        }

        self.navigation_bar = None;
        
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Calculate navigation bar rectangle
    fn calculate_navigation_bar_rect(&self, context: &Context) -> Rectangle {
        let height = scale_by_dpi(SMALL_BAR_HEIGHT, CURRENT_DEVICE.dpi) as i32;
        let width = self.rect.width() as i32;
        let y = if self.address_bar.is_some() {
            scale_by_dpi(crate::view::BIG_BAR_HEIGHT, CURRENT_DEVICE.dpi) as i32
        } else {
            0
        };
        
        rect![
            0,
            y,
            width,
            height
        ]
    }

    /// Get navigation bar state
    fn get_navigation_bar_state(&self) -> NavigationBarToggleState {
        NavigationBarToggleState {
            visible: self.navigation_bar.is_some(),
            active: false, // Navigation bar is never active
            config: NavigationBarToggleConfig::default(),
        }
    }

    /// Update navigation bar configuration
    pub fn update_navigation_bar_config(&mut self, config: NavigationBarToggleConfig) {
        // TODO: Implement navigation bar config update
        // This would require recreating the navigation bar if visible
    }

    /// Handle navigation bar events
    pub fn handle_navigation_bar_event(
        &mut self,
        event: &Event,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Close(ViewId::NavigationBar) => {
                self.hide_navigation_bar(rq, context);
                true
            }
            _ => false,
        }
    }

    /// Check if navigation bar should auto-hide
    pub fn should_auto_hide_navigation_bar(&self) -> bool {
        self.get_navigation_bar_state().config.auto_hide
    }

    /// Get navigation bar animation duration
    pub fn get_navigation_bar_animation_duration(&self) -> u32 {
        self.get_navigation_bar_state().config.animation_duration
    }

    /// Update navigation bar breadcrumbs
    pub fn update_navigation_bar_breadcrumbs(&mut self, rq: &mut RenderQueue) {
        if let Some(ref mut nav_bar) = self.navigation_bar {
            // TODO: Update breadcrumbs based on current directory
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }
}

/// Utility functions for navigation bar toggles
pub mod utils {
    use super::*;

    /// Create default navigation bar toggle config
    pub fn create_default_navigation_bar_config() -> NavigationBarToggleConfig {
        NavigationBarToggleConfig::default()
    }

    /// Format breadcrumbs for display
    pub fn format_breadcrumbs(path: &str, max_length: usize) -> String {
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        
        if parts.len() <= 3 {
            return path.to_string();
        }

        let mut result = String::new();
        
        // Always include root
        result.push('/');
        
        // Include middle parts with ellipsis
        result.push_str(".../");
        
        // Include last two parts
        if parts.len() >= 2 {
            result.push_str(parts[parts.len() - 2]);
            result.push('/');
            result.push_str(parts[parts.len() - 1]);
        }
        
        result
    }

    /// Calculate navigation bar height based on content
    pub fn calculate_navigation_bar_height(has_breadcrumbs: bool, has_buttons: bool) -> i32 {
        let base_height = scale_by_dpi(SMALL_BAR_HEIGHT, CURRENT_DEVICE.dpi) as i32;
        let extra_height = if has_breadcrumbs && has_buttons {
            scale_by_dpi(20.0, CURRENT_DEVICE.dpi) as i32
        } else if has_breadcrumbs || has_buttons {
            scale_by_dpi(10.0, CURRENT_DEVICE.dpi) as i32
        } else {
            0
        };
        
        base_height + extra_height
    }

    /// Get navigation bar button actions
    pub fn get_navigation_bar_actions() -> Vec<NavigationAction> {
        vec![
            NavigationAction::Back,
            NavigationAction::Forward,
            NavigationAction::Up,
            NavigationAction::Home,
            NavigationAction::Refresh,
        ]
    }

    /// Navigation bar actions
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NavigationAction {
        Back,
        Forward,
        Up,
        Home,
        Refresh,
        Settings,
    }
}
