//! Settings Toggle Module
//!
//! This module handles settings menu visibility and interaction for the Home view.

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::view::menu::{Menu, MenuKind};
use crate::view::{EntryId, EntryKind, Event, Hub, RenderData, RenderQueue, View, ViewId};

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
#[derive(Debug, Clone, Default)]
pub struct SettingsToggleState {
    pub _visible: bool,
    pub _active: bool,
    pub config: SettingsToggleConfig,
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
        let should_enable = enable.unwrap_or(self.settings_menu.is_none());

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
    fn hide_settings_menu(&mut self, rq: &mut RenderQueue, _context: &mut Context) {
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
        let mut entries = vec![
            EntryKind::Command("Font Settings".to_string(), EntryId::SystemInfo),
            EntryKind::Command("Display Settings".to_string(), EntryId::SystemInfo),
            EntryKind::Command("Reading Settings".to_string(), EntryId::SystemInfo),
        ];

        // Add advanced settings if enabled
        if self.get_settings_state().config.show_advanced {
            entries.push(EntryKind::Command(
                "Advanced Settings".to_string(),
                EntryId::SystemInfo,
            ));
        }

        entries.push(EntryKind::Separator);
        entries.push(EntryKind::Command("About".to_string(), EntryId::SystemInfo));
        entries.push(EntryKind::Command("Help".to_string(), EntryId::SystemInfo));

        Menu::new(
            rect,
            ViewId::SettingsMenu,
            MenuKind::DropDown,
            entries,
            context,
        )
    }

    /// Get settings state
    fn get_settings_state(&self) -> SettingsToggleState {
        SettingsToggleState {
            _visible: self.settings_menu.is_some(),
            _active: self.focus == Some(ViewId::SettingsMenu),
            config: SettingsToggleConfig::default(),
        }
    }

    /// Update settings configuration
    pub fn update_settings_config(
        &mut self,
        config: SettingsToggleConfig,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let was_visible = self.settings_menu.is_some();
        let old_config = self.get_settings_state().config;

        // If advanced settings visibility changed and menu is open, recreate it
        if config.show_advanced != old_config.show_advanced && was_visible {
            self.hide_settings_menu(rq, context);
            self.show_settings_menu(rq, context);
        }

        // Trigger refresh to reflect new settings
        if was_visible {
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
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
            Event::Select(ref entry_id) => {
                self.handle_settings_selection(entry_id, hub, rq, context);
                true
            }
            _ => false,
        }
    }

    /// Handle settings menu selection
    fn handle_settings_selection(
        &mut self,
        entry_id: &EntryId,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        match entry_id {
            EntryId::About => {
                // Send event to show about dialog
                hub.send(Event::Show(ViewId::AboutDialog)).ok();
                self.hide_settings_menu(rq, context);
            }
            EntryId::SystemInfo => {
                // Show system info dialog with comprehensive info
                hub.send(Event::Show(ViewId::SystemInfo)).ok();
                self.hide_settings_menu(rq, context);
            }
            _ => {
                // Handle any other settings entry IDs
                self.hide_settings_menu(rq, context);
            }
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
