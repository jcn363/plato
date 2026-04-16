//! Reader Input and Gesture Processing
//!
//! This module handles all input processing and gesture recognition for the Reader view,
//! including touch events, button presses, and keyboard input.

use crate::context::Context;
use crate::geom::Point;
use crate::gesture::GestureEvent;
use crate::input::{ButtonCode, ButtonStatus, DeviceEvent, FingerStatus};
use crate::view::{Event, Hub, Id, RenderQueue};

/// Input event types for the Reader
#[derive(Debug, Clone)]
pub enum ReaderInputEvent {
    TouchStart(Point),
    TouchMove(Point),
    TouchEnd(Point),
    ButtonPress(ButtonCode),
    ButtonRelease(ButtonCode),
    KeyboardInput(String),
}

/// Gesture types recognized by the Reader
#[derive(Debug, Clone)]
pub enum ReaderGesture {
    Tap(Point),
    DoubleTap(Point),
    LongPress(Point),
    Swipe(Point, Point),
    Pinch(Point, Point, f32),
    Pan(Point, Point),
}

/// Input handler for the Reader view
pub struct ReaderInputHandler {
    pub id: Id,
    pub held_buttons: std::collections::HashSet<ButtonCode>,
    pub current_gesture: Option<ReaderGesture>,
    pub gesture_start: Option<Point>,
}

impl ReaderInputHandler {
    /// Create a new input handler
    pub fn new(id: Id) -> Self {
        Self {
            id,
            held_buttons: std::collections::HashSet::new(),
            current_gesture: None,
            gesture_start: None,
        }
    }

    /// Process a device event and convert it to reader input
    pub fn handle_device_event(&mut self, event: DeviceEvent) -> Vec<ReaderInputEvent> {
        match event {
            DeviceEvent::Finger {
                status, position, ..
            } => match status {
                FingerStatus::Down => vec![ReaderInputEvent::TouchStart(position)],
                FingerStatus::Move | FingerStatus::Motion => {
                    vec![ReaderInputEvent::TouchMove(position)]
                }
                FingerStatus::Up => vec![ReaderInputEvent::TouchEnd(position)],
            },
            DeviceEvent::Button { code, status, .. } => match status {
                ButtonStatus::Pressed => {
                    self.held_buttons.insert(code);
                    vec![ReaderInputEvent::ButtonPress(code)]
                }
                ButtonStatus::Released | ButtonStatus::Repeated => {
                    self.held_buttons.remove(&code);
                    vec![ReaderInputEvent::ButtonRelease(code)]
                }
            },
            DeviceEvent::Keyboard { code, .. } => {
                vec![ReaderInputEvent::KeyboardInput(code.to_string())]
            }
            _ => vec![],
        }
    }

    /// Recognize gestures from input events
    pub fn recognize_gesture(&mut self, events: &[ReaderInputEvent]) -> Option<ReaderGesture> {
        // Simple gesture recognition logic
        for event in events {
            match event {
                ReaderInputEvent::TouchStart(pos) => {
                    self.gesture_start = Some(*pos);
                }
                ReaderInputEvent::TouchEnd(pos) => {
                    if let Some(start) = self.gesture_start.take() {
                        let distance = start.distance_to(*pos);
                        if distance < 10.0 {
                            return Some(ReaderGesture::Tap(*pos));
                        } else {
                            return Some(ReaderGesture::Swipe(start, *pos));
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Handle a gesture and generate appropriate events
    pub fn handle_gesture(
        &mut self,
        gesture: ReaderGesture,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Vec<Event> {
        match gesture {
            ReaderGesture::Tap(pos) => {
                vec![Event::Tap(pos)]
            }
            ReaderGesture::Swipe(start, end) => {
                vec![Event::Swipe(start, end)]
            }
            ReaderGesture::DoubleTap(pos) => {
                vec![Event::DoubleTap(pos)]
            }
            ReaderGesture::LongPress(pos) => {
                vec![Event::Hold(pos)]
            }
            ReaderGesture::Pinch(center, _, scale) => {
                vec![Event::Gesture(GestureEvent::Pinch {
                    axis: crate::geom::Axis::Horizontal,
                    center,
                    factor: *scale,
                })]
            }
            ReaderGesture::Pan(start, end) => {
                vec![Event::Gesture(GestureEvent::Swipe {
                    dir: if end.x > start.x {
                        crate::geom::Dir::East
                    } else {
                        crate::geom::Dir::West
                    },
                    start,
                    end,
                })]
            }
        }
    }

    /// Check if a button is currently held
    pub fn is_button_held(&self, code: ButtonCode) -> bool {
        self.held_buttons.contains(&code)
    }

    /// Get all currently held buttons
    pub fn get_held_buttons(&self) -> Vec<ButtonCode> {
        self.held_buttons.iter().cloned().collect()
    }
}
