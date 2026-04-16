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

pub use font_cache::{FontCache, FontCacheEntry};
pub use hyphenation::{HyphenationConfig, SimpleHyphenator};
pub use line_breaker::LineBreaker;
pub use text_layout::{TextLayoutConfig, TextLayoutEngine, TextLayoutResult};
pub use text_renderer::{TextRenderer, TextRenderConfig};
pub use text_shaping::{TextDirection, TextShaper, TextShapingConfig};
