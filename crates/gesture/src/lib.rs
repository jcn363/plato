//! Plato Gesture Module
//!
//! This crate provides gesture handling for Plato.

pub use plato_core::gesture::{
    gesture_events, parse_gesture_events, platform_hold_delay_ms, platform_tap_jitter_mm,
    GestureEvent, TouchState,
};
