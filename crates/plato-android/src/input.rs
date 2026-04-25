#![cfg_attr(not(target_os = "android"), allow(dead_code, unused_imports))]

#[cfg(target_os = "android")]
use android_activity::input::{MotionEvent, MotionAction};
use plato_core::geom::Point;
use plato_core::input::{DeviceEvent, FingerStatus};
use std::sync::mpsc::Sender;
use std::time::SystemTime;

/// Translate a MotionEvent into Plato DeviceEvent::Finger events
#[cfg(target_os = "android")]
pub fn translate_motion_event(event: &MotionEvent, tx: &Sender<DeviceEvent>) {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    // Iterate over all pointers in the motion event
    for pointer in event.pointers() {
        let x = pointer.x() as i32;
        let y = pointer.y() as i32;
        let position = Point::new(x, y);
        // android-activity 0.5 doesn't have pointer.id(), use pointer_index instead
        let id = pointer.pointer_index() as i32;

        let status = match event.action() {
            MotionAction::Down | MotionAction::PointerDown => FingerStatus::Down,
            MotionAction::Move => FingerStatus::Motion,
            MotionAction::Up | MotionAction::PointerUp | MotionAction::Cancel => FingerStatus::Up,
            _ => continue, // Skip other actions
        };

        let device_event = DeviceEvent::Finger {
            id,
            time,
            status,
            position,
        };

        // Send the event to the gesture processor
        if let Err(e) = tx.send(device_event) {
            log::error!("Failed to send finger event: {}", e);
        }
    }
}
