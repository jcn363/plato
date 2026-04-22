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
mod tests;

pub use controller::{EInkController, SunxiController, MxcController};
pub use damage_tracker::{DamageTracker, FrameBuffer};
pub use ghosting::GhostingReducer;
pub use grayscale::{GrayscaleConverter, DitheringMode};
pub use partial_refresh::PartialRefreshManager;
pub use waveform::{WaveformMode, select_waveform, ContentType, UpdateType};
