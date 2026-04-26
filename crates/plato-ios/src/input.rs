//! iOS input event translation
//!
//! This module provides the iOS-specific input event translation from UITouch
//! to Plato's DeviceEvent system.

#![cfg(feature = "ios")]
#![deny(warnings)]

use plato_core::geom::Point;
use plato_core::input::{DeviceEvent, FingerStatus};
use std::sync::mpsc::Sender;
use std::time::SystemTime;

/// Translate a `UITouch` event into Plato `DeviceEvent::Finger` events
/// This is called from Swift when touch events occur
pub fn translate_touch_event(
    finger_id: i32,
    x: f32,
    y: f32,
    phase: i32, // 0=Began, 1=Moved, 2=Ended, 3=Cancelled
    tx: &Sender<DeviceEvent>,
) {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    let position = Point::new(x as i32, y as i32);

    let status = match phase {
        0 => FingerStatus::Down,
        1 => FingerStatus::Motion,
        2 | 3 => FingerStatus::Up,
        _ => return, // Skip unknown phases
    };

    let device_event = DeviceEvent::Finger {
        id: finger_id,
        time,
        status,
        position,
    };

    // Send the event to the gesture processor
    if let Err(e) = tx.send(device_event) {
        log::error!("Failed to send finger event: {e}");
    }
}

/// Translate a gesture recognizer event
/// For MVP, we handle basic tap gestures
#[allow(dead_code)] // Reserved for future iOS gesture recognizer integration (UIGestureRecognizer for pinch, tap)
#[must_use]
pub fn translate_gesture_event(
    gesture_type: i32, // 0=Tap, 1=Pinch, etc.
    x: f32,
    y: f32,
    factor: f32, // For pinch gestures
) -> Option<plato_core::gesture::GestureEvent> {
    let position = Point::new(x as i32, y as i32);

    match gesture_type {
        0 => Some(plato_core::gesture::GestureEvent::Tap(position)),
        1 => Some(plato_core::gesture::GestureEvent::Pinch {
            center: position,
            axis: plato_core::geom::Axis::Horizontal, // Placeholder
            factor,
        }),
        _ => None,
    }
}
