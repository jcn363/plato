//! Reader Rendering Module
//!
//! Handles page rendering, animation, text extraction, and display updates.
//!
//! ## Methods to Move Here
//! - `render()` - Main rendering to framebuffer (~200 lines)
//! - `render_animation()` - Page transition animations (~80 lines)
//! - `render_current_page()` - Render specific page
//! - `render_results()` - Highlight search results
//! - `scale_page()` - Handle zoom scaling
//! - `crop_margins()` - Margin cropping logic
//! - `text_excerpt()` - Extract text from selection
//! - `selected_text()` - Get currently selected text
//! - `text_rect()` - Calculate text bounding box
//! - `selection_rect()` - Get selection rectangle
//!
//! ## Types
//! Uses `RenderChunk`, `Resource` from reader_core for page rendering state.

pub(crate) use super::reader_core::{AnimState, PageAnimKind, PageAnimation, RenderChunk};
