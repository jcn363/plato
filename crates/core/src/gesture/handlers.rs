//! Gesture Event Handlers
//!
//! Functions for handling finger and button events in the gesture recognition system.

use crate::geom::Point;
use crate::input::ButtonCode;
use crate::view::Event;
use rustc_hash::FxHashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

use super::platform::platform_hold_delay_ms;
use super::types::TouchState;

pub fn handle_finger_down(
    contacts: &Arc<Mutex<FxHashMap<i32, TouchState>>>,
    segments: &Arc<Mutex<Vec<Vec<Point>>>>,
    ty: Sender<Event>,
    id: i32,
    position: Point,
    time: f64,
    hold_jitter: f32,
) {
    let mut ct = contacts.lock().expect("contacts lock poisoned");
    ct.insert(
        id,
        TouchState {
            time,
            held: false,
            positions: vec![position],
        },
    );
    spawn_finger_hold_detector(
        contacts.clone(),
        segments.clone(),
        ty,
        id,
        position,
        time,
        hold_jitter,
    );
}

pub fn handle_finger_motion(
    contacts: &Arc<Mutex<FxHashMap<i32, TouchState>>>,
    id: i32,
    position: Point,
) {
    let mut ct = contacts.lock().expect("contacts lock poisoned");
    if let Some(ref mut ts) = ct.get_mut(&id) {
        ts.positions.push(position);
    }
}

pub fn handle_finger_up(
    contacts: &Arc<Mutex<FxHashMap<i32, TouchState>>>,
    segments: &Arc<Mutex<Vec<Vec<Point>>>>,
    ty: Sender<Event>,
    id: i32,
    position: Point,
    tap_jitter: f32,
) {
    let mut ct = contacts.lock().expect("contacts lock poisoned");
    let mut sg = segments.lock().expect("segments lock poisoned");
    if let Some(mut ts) = ct.remove(&id) {
        if !ts.held {
            ts.positions.push(position);
            sg.push(ts.positions);
        }
    }
    if ct.is_empty() && !sg.is_empty() {
        super::processing::process_segments(&mut sg, ty, tap_jitter);
    }
}

pub fn handle_button_pressed(
    buttons: &Arc<Mutex<FxHashMap<ButtonCode, f64>>>,
    ty: Sender<Event>,
    code: ButtonCode,
    time: f64,
) {
    let mut bt = buttons.lock().expect("buttons lock poisoned");
    bt.insert(code, time);
    spawn_button_hold_detector(buttons.clone(), ty, code, time);
}

pub fn handle_button_released(buttons: &Arc<Mutex<FxHashMap<ButtonCode, f64>>>, code: ButtonCode) {
    let mut bt = buttons.lock().expect("buttons lock poisoned");
    bt.remove(&code);
}

fn spawn_finger_hold_detector(
    contacts: Arc<Mutex<FxHashMap<i32, TouchState>>>,
    segments: Arc<Mutex<Vec<Vec<Point>>>>,
    ty: Sender<Event>,
    id: i32,
    position: Point,
    time: f64,
    hold_jitter: f32,
) {
    thread::spawn(move || {
        let mut held = false;
        let hold_delay_short = std::time::Duration::from_millis(platform_hold_delay_ms());
        thread::sleep(hold_delay_short);
        {
            let mut ct = contacts.lock().expect("contacts lock poisoned");
            let sg = segments.lock().expect("segments lock poisoned");
            if ct.len() > 1 || !sg.is_empty() {
                return;
            }
            if let Some(ts) = ct.get(&id) {
                let tp = &ts.positions;
                if (ts.time - time).abs() < f64::EPSILON
                    && (tp[tp.len() - 1] - position).length() < hold_jitter
                    && (tp[tp.len() / 2] - position).length() < hold_jitter
                {
                    held = true;
                    ty.send(Event::Gesture(super::types::GestureEvent::HoldFingerShort(
                        position, id,
                    )))
                    .ok();
                }
            }
            if held {
                if let Some(ts) = ct.get_mut(&id) {
                    ts.held = true;
                }
            } else {
                return;
            }
        }
        let hold_delay_long = std::time::Duration::from_millis(
            crate::consts::gesture::HOLD_DELAY_LONG.as_millis() as u64,
        );
        thread::sleep(hold_delay_long - hold_delay_short);
        {
            let mut ct = contacts.lock().expect("contacts lock poisoned");
            let sg = segments.lock().expect("segments lock poisoned");
            if ct.len() > 1 || !sg.is_empty() {
                return;
            }
            if let Some(ts) = ct.get_mut(&id) {
                let tp = &ts.positions;
                if (ts.time - time).abs() < f64::EPSILON
                    && (tp[tp.len() - 1] - position).length() < hold_jitter
                    && (tp[tp.len() / 2] - position).length() < hold_jitter
                {
                    ty.send(Event::Gesture(super::types::GestureEvent::HoldFingerLong(
                        position, id,
                    )))
                    .ok();
                }
            }
        }
    });
}

fn spawn_button_hold_detector(
    buttons: Arc<Mutex<FxHashMap<ButtonCode, f64>>>,
    ty: Sender<Event>,
    code: ButtonCode,
    time: f64,
) {
    thread::spawn(move || {
        let hold_delay_short = std::time::Duration::from_millis(platform_hold_delay_ms());
        let hold_delay_long = std::time::Duration::from_millis(
            crate::consts::gesture::HOLD_DELAY_LONG.as_millis() as u64,
        );
        thread::sleep(hold_delay_short);
        {
            let bt = buttons.lock().expect("buttons lock poisoned");
            if let Some(&initial_time) = bt.get(&code) {
                if initial_time == time {
                    ty.send(Event::Gesture(super::types::GestureEvent::HoldButtonShort(
                        code,
                    )))
                    .ok();
                }
            }
        }
        thread::sleep(hold_delay_long - hold_delay_short);
        {
            let bt = buttons.lock().expect("buttons lock poisoned");
            if let Some(&initial_time) = bt.get(&code) {
                if (initial_time - time).abs() < f64::EPSILON {
                    ty.send(Event::Gesture(super::types::GestureEvent::HoldButtonLong(
                        code,
                    )))
                    .ok();
                }
            }
        }
    });
}
