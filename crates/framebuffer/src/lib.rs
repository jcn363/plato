//! Plato Framebuffer Module
//!
//! This crate provides framebuffer management for Plato.

pub use plato_core::framebuffer::{
    Display, KoboFramebuffer1, KoboFramebuffer2, Pixmap, SoftwareFramebuffer, UpdateMode,
};

// Desktop framebuffer only available on non-ARM targets (Linux desktop)
#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
pub use plato_core::framebuffer::DesktopFramebuffer;
