//! Plato E-Ink Module
//!
//! This crate provides e-ink display functionality for Plato.

pub use plato_core::eink::{
    select_waveform, ContentType, DamageTracker, DitheringMode, EInkController, FrameBuffer,
    GhostingReducer, GrayscaleConverter, MxcController, PartialRefreshManager, SunxiController,
    UpdateType, WaveformMode,
};
