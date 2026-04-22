//! E-ink optimization layer for Plato
//!
//! This module provides e-ink display optimization including:
//! - Damage tracking for partial refresh
//! - Grayscale conversion with dithering
//! - Waveform mode selection
//! - Ghosting reduction
//! - Display controller abstraction

mod controller;
mod damage_tracker;
mod ghosting;
mod grayscale;
mod partial_refresh;
mod waveform;

#[cfg(test)]
mod regression_tests;
#[cfg(test)]
mod tests;

pub use controller::{EInkController, MxcController, SunxiController};
pub use damage_tracker::{DamageTracker, FrameBuffer};
pub use ghosting::GhostingReducer;
pub use grayscale::{DitheringMode, GrayscaleConverter};
pub use partial_refresh::PartialRefreshManager;
pub use waveform::{select_waveform, ContentType, UpdateType, WaveformMode};
