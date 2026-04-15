//! Settings Toggle Module
//!
//! This module handles settings menu visibility and interaction for the Home view.

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::view::menu::{Menu, MenuKind};
use crate::view::{Event, Hub, RenderData, RenderQueue, View};
use crate::view::{EntryId, ViewId};

use super::super::Home;

/// Settings toggle configuration
#[derive(Debug, Clone)]
pub struct SettingsToggleConfig {
    pub auto_save: bool,
    pub show_advanced: bool,
    pub animation_duration: u32,
}

impl Default for SettingsToggleConfig {
    fn default() -> Self {
        Self {
            auto_save: true,
            show_advanced: false,
            animation_duration: 250,
        }
    }
}

/// Settings toggle state
#[derive(Debug, Clone)]
pub struct SettingsToggleState {
    pub visible: bool,
    pub active: bool,
    pub config: SettingsToggleConfig,
}

impl Default for SettingsToggleState {
    fn default() -> Self {
        Self {
            visible: false,
            active: false,
            config: SettingsToggleConfig::default(),
        }
    }
}

impl Home {
    /// Toggle settings menu visibility
    pub fn toggle_settings_menu(
        &mut self,
        enable: Option<bool>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(!self.settings_menu.is_some());
        
        if should_enable {
            self.show_settings_menu(rq, context);
        } else {
            self.hide_settings_menu(rq, context);
        }
    }

    /// Show settings menu
    fn show_settings_menu(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.settings_menu.is_some() {
            return;
        }

        let rect = self.calculate_settings_menu_rect(context);
        let menu = self.create_settings_menu(rect, context);
        
        self.settings_menu = Some(Box::new(menu) as Box<dyn View>);
        self.focus = Some(ViewId::SettingsMenu);
        
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide settings menu
    fn hide_settings_menu(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.settings_menu.is_none() {
            return;
        }

        self.settings_menu = None;
        self.focus = None;
        
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Calculate settings menu rectangle
    fn calculate_settings_menu_rect(&self, context: &Context) -> Rectangle {
        let screen_width = context.display.dims.0 as i32;
        let screen_height = context.display.dims.1 as i32;
        
        let width = (screen_width as f32 * 0.6) as i32;
        let height = (screen_height as f32 * 0.8) as i32;
        let x = (screen_width - width) / 2;
        let y = (screen_height - height) / 2;
        
        rect![x, y, width, height]
    }

    /// Create settings menu
    fn create_settings_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let mut menu = Menu::new(rect, context);
        
        // Add basic settings
        menu.add_entry(crate::view::menu_entry::MenuEntry::new(
            "Font Settings".to_string(),
            self.id,
            Some("font_settings".to_string()),
        ));
        
        menu.add_entry(crate::view::menu_entry::MenuEntry::new(
            "Display Settings".to_string(),
            self.id,
            Some("display_settings".to_string()),
        ));
        
        menu.add_entry(crate::view::menu_entry::MenuEntry::new(
            "Reading Settings".to_string(),
            self.id,
            Some("reading_settings".to_string()),
        ));
        
        // Add advanced settings if enabled
        if self.get_settings_state().config.show_advanced {
            menu.add_entry(crate::view::menu_entry::MenuEntry::new(
                "Advanced Settings".to_string(),
                self.id,
                Some("advanced_settings".to_string()),
            ));
        }
        
        menu.add_separator();
        menu.add_entry(crate::view::menu_entry::MenuEntry::new(
            "About".to_string(),
            self.id,
            Some("about".to_string()),
        ));
        
        menu.add_entry(crate::view::menu_entry::MenuEntry::new(
            "Help".to_string(),
            self.id,
            Some("help".to_string()),
        ));
        
        menu
    }

    /// Get settings state
    fn get_settings_state(&self) -> SettingsToggleState {
        SettingsToggleState {
            visible: self.settings_menu.is_some(),
            active: self.focus == Some(ViewId::SettingsMenu),
            config: SettingsToggleConfig::default(),
        }
    }

    /// Update settings configuration
    pub fn update_settings_config(&mut self, config: SettingsToggleConfig) {
        // TODO: Implement settings config update
        // This would require recreating the settings menu if visible
    }

    /// Handle settings menu events
    pub fn handle_settings_menu_event(
        &mut self,
        event: &Event,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Close(ViewId::SettingsMenu) => {
                self.hide_settings_menu(rq, context);
                true
            }
            Event::Select(name) => {
                self.handle_settings_selection(name, hub, rq, context);
                true
            }
            _ => false,
        }
    }

    /// Handle settings menu selection
    fn handle_settings_selection(
        &mut self,
        name: &str,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        match name {
            "font_settings" => {
                // TODO: Send appropriate font settings event
                self.hide_settings_menu(rq, context);
            }
            "display_settings" => {
                // TODO: Send appropriate display settings event
                self.hide_settings_menu(rq, context);
            }
            "reading_settings" => {
                // TODO: Send appropriate reading settings event
                self.hide_settings_menu(rq, context);
            }
            "advanced_settings" => {
                // TODO: Send appropriate advanced settings event
                self.hide_settings_menu(rq, context);
            }
            "about" => {
                // TODO: Send appropriate about event
                self.hide_settings_menu(rq, context);
            }
            "help" => {
                // TODO: Send appropriate help event
                self.hide_settings_menu(rq, context);
            }
            _ => {}
        }
    }

    /// Check if settings should auto-save
    pub fn should_auto_save_settings(&self) -> bool {
        self.get_settings_state().config.auto_save
    }

    /// Get settings animation duration
    pub fn get_settings_animation_duration(&self) -> u32 {
        self.get_settings_state().config.animation_duration
    }
}

/// Utility functions for settings toggles
pub mod utils {
    use super::*;

    /// Create default settings toggle config
    pub fn create_default_settings_config() -> SettingsToggleConfig {
        SettingsToggleConfig::default()
    }

    /// Get available settings categories
    pub fn get_settings_categories() -> Vec<SettingsCategory> {
        vec![
            SettingsCategory::Font,
            SettingsCategory::Display,
            SettingsCategory::Reading,
            SettingsCategory::Advanced,
            SettingsCategory::About,
            SettingsCategory::Help,
        ]
    }

    /// Settings categories
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SettingsCategory {
        Font,
        Display,
        Reading,
        Advanced,
        About,
        Help,
    }

    /// Validate settings value
    pub fn validate_settings_value(category: SettingsCategory, key: &str, value: &str) -> bool {
        match (category, key) {
            (SettingsCategory::Font, "font_size") => value.parse::<f32>().is_ok(),
            (SettingsCategory::Display, "brightness") => value.parse::<f32>().is_ok(),
            (SettingsCategory::Reading, "page_margin") => value.parse::<i32>().is_ok(),
            _ => true, // Default to valid for unknown settings
        }
    }

    /// Format settings value for display
    pub fn format_settings_value(category: SettingsCategory, key: &str, value: &str) -> String {
        match (category, key) {
            (SettingsCategory::Font, "font_size") => {
                if let Ok(size) = value.parse::<f32>() {
                    format!("{:.1}pt", size)
                } else {
                    value.to_string()
                }
            }
            (SettingsCategory::Display, "brightness") => {
                if let Ok(brightness) = value.parse::<f32>() {
                    format!("{:.0}%", brightness * 100.0)
                } else {
                    value.to_string()
                }
            }
            _ => value.to_string(),
        }
    }
}

// Settings event types
#[derive(Debug, Clone)]
pub enum FontSettings {}
#[derive(Debug, Clone)]
pub enum DisplaySettings {}
#[derive(Debug, Clone)]
pub enum ReadingSettings {}
#[derive(Debug, Clone)]
pub enum AdvancedSettings {}
#[derive(Debug, Clone)]
pub enum About {}
#[derive(Debug, Clone)]
pub enum Help {}
