//! HTML Engine Text Module
//!
//! This module provides text layout, hyphenation, and shaping functionality
//! for the HTML rendering engine.

pub mod font_cache;
pub mod hyphenation;
pub mod line_breaker;
pub mod text_layout;
pub mod text_renderer;
pub mod text_shaping;

pub use font_cache::*;
pub use hyphenation::*;
pub use line_breaker::*;
pub use text_layout::*;
pub use text_renderer::*;
pub use text_shaping::*;
