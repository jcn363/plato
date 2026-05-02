//! Plato Configuration Module
//!
//! This crate provides centralized configuration management for Plato,
//! including settings loading, validation, and saving.
//!
//! This is a thin wrapper that re-exports configuration types from plato-core.

pub use plato_core::settings::{load_settings, save_settings, ConfigManager, Settings};
