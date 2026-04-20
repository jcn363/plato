#![warn(clippy::all)]

#[macro_use]
pub mod geom;

pub mod battery;
pub mod color;
pub mod consts;
pub mod context;
pub mod cover_editor;
pub mod device;
mod dictionary;
pub mod document;
pub mod font;
pub mod framebuffer;
pub mod frontlight;
pub mod gesture;
pub mod helpers;
pub mod i18n;
pub mod input;
pub mod library;
pub mod lightsensor;
pub mod metadata;
pub mod opds;
pub mod plugin;
pub mod rtc;
pub mod settings;
pub mod sync;
pub mod theme;
pub mod thumbnail;
mod unit;
pub mod update;
pub mod validation;
pub mod view;

/// Mock implementations for testing
///
/// Provides mock implementations of core traits (Framebuffer, Battery,
/// Frontlight, LightSensor, Document) for unit testing without hardware.
pub mod test_mocks;

pub use anyhow;
pub use chrono;
pub use globset;
pub use image;
pub use png;
pub use rand_core;
pub use rand_xoshiro;
pub use rustc_hash;
pub use serde;
pub use serde_json;
pub use walkdir;
