//! Gesture Recognition System
//!
//! This module provides touch gesture recognition for Plato, converting raw input events
//! into meaningful gestures for navigation and interaction.
//!
//! ## Supported Gestures
//!
//! - **Tap**: Single touch tap with jitter tolerance
//! - **MultiTap**: Two-finger tap
//! - **Swipe**: Directional swipe (up/down/left/right)
//! - **SlantedSwipe**: Diagonal swipe
//! - **MultiSwipe**: Two-finger swipe
//! - **Arrow**: Line gesture in a direction
//! - **MultiArrow**: Two-finger line gesture
//! - **Corner**: Corner tap gesture
//! - **MultiCorner**: Two-finger corner tap
//! - **Pinch**: Pinch-to-zoom gesture
//! - **Spread**: Spread-to-zoom gesture
//! - **Hold**: Long press gesture
//! - **Rotate**: Two-finger rotation
//!
//! ## Algorithm
//!
//! The system uses O(1) algorithms throughout:
//! - `elbow()` samples 2 fixed points (1/3 and 2/3 of stroke)
//! - `nearest_segment_point()` uses pure vector math
//! - State machine processes events in O(1) per input event

mod handlers;
mod platform;
mod processing;
mod types;

pub use platform::{platform_hold_delay_ms, platform_tap_jitter_mm};
pub use types::{GestureEvent, TouchState};

use crate::geom::Point;
use crate::input::DeviceEvent;
use crate::view::Event;
use rustc_hash::FxHashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

// Re-export gesture constants from canonical source in consts::gesture
// per Single Source of Truth rule.
pub use crate::consts::gesture::{
    HOLD_DELAY_LONG, HOLD_DELAY_SHORT, HOLD_JITTER_MM, TAP_JITTER_MM,
};

pub fn gesture_events(rx: Receiver<DeviceEvent>) -> Receiver<Event> {
    let (ty, ry) = mpsc::channel();
    thread::spawn(move || {
        let rx = rx;
        let ty = ty;
        parse_gesture_events(&rx, &ty);
    });
    ry
}

pub fn parse_gesture_events(rx: &Receiver<DeviceEvent>, ty: &Sender<Event>) {
    use crate::consts::gesture::HOLD_JITTER_MM;
    use crate::device::CURRENT_DEVICE;
    use crate::input::{ButtonCode, ButtonStatus, FingerStatus};
    use crate::unit::mm_to_px;

    let contacts: Arc<Mutex<FxHashMap<i32, TouchState>>> =
        Arc::new(Mutex::new(FxHashMap::default()));
    let buttons: Arc<Mutex<FxHashMap<ButtonCode, f64>>> =
        Arc::new(Mutex::new(FxHashMap::default()));
    let segments: Arc<Mutex<Vec<Vec<Point>>>> = Arc::new(Mutex::new(Vec::new()));
    let tap_jitter = mm_to_px(platform_tap_jitter_mm(), CURRENT_DEVICE.dpi);
    let hold_jitter = mm_to_px(HOLD_JITTER_MM, CURRENT_DEVICE.dpi);

    while let Ok(evt) = rx.recv() {
        ty.send(Event::Device(evt)).ok();
        match evt {
            DeviceEvent::Finger {
                status: FingerStatus::Down,
                position,
                id,
                time,
            } => {
                handlers::handle_finger_down(
                    &contacts,
                    &segments,
                    ty.clone(),
                    id,
                    position,
                    time,
                    hold_jitter,
                );
            }
            DeviceEvent::Finger {
                status: FingerStatus::Motion,
                position,
                id,
                ..
            } => {
                handlers::handle_finger_motion(&contacts, id, position);
            }
            DeviceEvent::Finger {
                status: FingerStatus::Up,
                position,
                id,
                ..
            } => {
                handlers::handle_finger_up(
                    &contacts,
                    &segments,
                    ty.clone(),
                    id,
                    position,
                    tap_jitter,
                );
            }
            DeviceEvent::Button {
                status: ButtonStatus::Pressed,
                code,
                time,
            } => {
                handlers::handle_button_pressed(&buttons, ty.clone(), code, time);
            }
            DeviceEvent::Button {
                status: ButtonStatus::Released,
                code,
                ..
            } => {
                handlers::handle_button_released(&buttons, code);
            }
            _ => (),
        }
    }
}
