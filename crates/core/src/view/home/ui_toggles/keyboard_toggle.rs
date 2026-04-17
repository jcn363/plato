//! Keyboard Toggle Module
//!
//! This module handles keyboard visibility and interaction for the Home view.

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::unit::scale_by_dpi;
use crate::view::keyboard::Keyboard;
use crate::view::{Event, Hub, RenderData, RenderQueue, View, ViewId};

use super::super::Home;

/// Keyboard toggle configuration
#[derive(Debug, Clone)]
pub struct KeyboardToggleConfig {
    pub auto_hide: bool,
    pub animation_duration: u32,
    pub position: KeyboardPosition,
}

/// Keyboard position options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardPosition {
    Bottom,
    Top,
    Floating,
}

impl Default for KeyboardToggleConfig {
    fn default() -> Self {
        Self {
            auto_hide: true,
            animation_duration: 200,
            position: KeyboardPosition::Bottom,
        }
    }
}

/// Keyboard toggle state
#[derive(Debug, Clone)]
pub struct KeyboardToggleState {
    pub visible: bool,
    pub active: bool,
    pub config: KeyboardToggleConfig,
}

impl Default for KeyboardToggleState {
    fn default() -> Self {
        Self {
            visible: false,
            active: false,
            config: KeyboardToggleConfig::default(),
        }
    }
}

impl Home {
    /// Toggle keyboard visibility
    pub fn toggle_keyboard(
        &mut self,
        enable: Option<bool>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(!self.keyboard.is_some());

        if should_enable {
            self.show_keyboard(rq, context);
        } else {
            self.hide_keyboard(rq, context);
        }
    }

    /// Show keyboard
    fn show_keyboard(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.keyboard.is_some() {
            return;
        }

        let mut rect = self.calculate_keyboard_rect(context);
        let keyboard = Keyboard::new(&mut rect, false, context);

        self.keyboard = Some(Box::new(keyboard) as Box<dyn View>);
        self.focus = Some(ViewId::Keyboard);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide keyboard
    fn hide_keyboard(&mut self, rq: &mut RenderQueue, _context: &mut Context) {
        if self.keyboard.is_none() {
            return;
        }

        self.keyboard = None;
        self.focus = None;

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Calculate keyboard rectangle
    fn calculate_keyboard_rect(&self, context: &Context) -> Rectangle {
        let screen_height = context.display.dims.1 as i32;
        let keyboard_height = scale_by_dpi(240.0, CURRENT_DEVICE.dpi) as i32;

        match self.get_keyboard_state().config.position {
            KeyboardPosition::Bottom => {
                rect![
                    0,
                    screen_height - keyboard_height,
                    self.rect.width() as i32,
                    keyboard_height
                ]
            }
            KeyboardPosition::Top => {
                rect![0, 0, self.rect.width() as i32, keyboard_height]
            }
            KeyboardPosition::Floating => {
                let width = scale_by_dpi(600.0, CURRENT_DEVICE.dpi) as i32;
                let height = keyboard_height;
                let x = (self.rect.width() as i32 - width) / 2;
                let y = (self.rect.height() as i32 - height) / 2;

                rect![x, y, width, height]
            }
        }
    }

    /// Get keyboard state
    fn get_keyboard_state(&self) -> KeyboardToggleState {
        KeyboardToggleState {
            visible: self.keyboard.is_some(),
            active: self.focus == Some(ViewId::Keyboard),
            config: KeyboardToggleConfig::default(),
        }
    }

    /// Update keyboard configuration
    pub fn update_keyboard_config(&mut self, _config: KeyboardToggleConfig) {
        // TODO: Implement keyboard config update
        // This would require recreating the keyboard if visible
    }

    /// Handle keyboard events
    pub fn handle_keyboard_event(
        &mut self,
        event: &Event,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Submit(ViewId::Keyboard, text) => {
                self.handle_keyboard_submit(&text, hub, rq, context);
                true
            }
            Event::Close(ViewId::Keyboard) => {
                self.hide_keyboard(rq, context);
                true
            }
            _ => false,
        }
    }

    /// Handle keyboard text submission
    fn handle_keyboard_submit(
        &mut self,
        _text: &str,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        // Handle keyboard submission - just hide keyboard for now
        self.hide_keyboard(rq, context);
    }

    /// Check if keyboard should auto-hide
    pub fn should_auto_hide_keyboard(&self) -> bool {
        self.get_keyboard_state().config.auto_hide
    }

    /// Get keyboard animation duration
    pub fn get_keyboard_animation_duration(&self) -> u32 {
        self.get_keyboard_state().config.animation_duration
    }
}

/// Utility functions for keyboard toggles
#[allow(dead_code)] // Reserved for future keyboard utilities
pub mod utils {
    use super::*;

    /// Create default keyboard toggle config
    pub fn create_default_keyboard_config() -> KeyboardToggleConfig {
        KeyboardToggleConfig::default()
    }

    /// Calculate optimal keyboard size for screen
    pub fn calculate_optimal_keyboard_size(screen_width: i32, screen_height: i32) -> (i32, i32) {
        let width = (screen_width as f32 * 0.9) as i32;
        let height = (screen_height as f32 * 0.3) as i32;
        (width, height)
    }

    /// Check if keyboard should be shown for input
    pub fn should_show_keyboard_for_input(input_type: InputType) -> bool {
        matches!(
            input_type,
            InputType::Text | InputType::Search | InputType::Url
        )
    }

    /// Input types that trigger keyboard
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum InputType {
        Text,
        Search,
        Url,
        Numeric,
        Password,
    }
}
