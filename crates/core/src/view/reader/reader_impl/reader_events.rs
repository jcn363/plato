//! Reader Event Handling
//!
//! This module handles user input events for the Reader view.
//! It processes touch events, keyboard input, and device button events,
//! dispatching to appropriate handlers based on the current state.

use crate::context::Context;
use crate::input::{ButtonStatus, DeviceEvent};
use crate::view::{Hub, RenderQueue};

use super::reader::Reader;

impl Reader {
    /// Handle device event
    pub fn handle_device_event(
        &mut self,
        device_event: DeviceEvent,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &Context,
    ) {
        if let DeviceEvent::Button { code, status, .. } = device_event {
            if status == ButtonStatus::Pressed {
                self.held_buttons.insert(code);
            } else {
                self.held_buttons.remove(&code);
            }
        }
        self.queue_partial_update(rq);
    }

    /// Handle keyboard
    pub fn handle_keyboard(
        &mut self,
        _key: crate::view::key::KeyKind,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &Context,
    ) {
        self.queue_partial_update(rq);
    }
}
