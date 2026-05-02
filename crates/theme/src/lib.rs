//! Plato Theme Module
//!
//! This crate provides theme management for Plato.

pub use plato_core::theme::{
    auto_threshold, background, foreground, is_dark_mode, is_sepia_mode, sepia_background,
    sepia_foreground, set_auto_threshold, set_dark_mode, set_theme_mode, theme_mode,
    update_from_light_sensor, update_from_schedule,
};
