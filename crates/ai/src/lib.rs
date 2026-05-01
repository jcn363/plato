//! Plato AI - Local-first, privacy-focused AI features for e-readers
//!
//! This crate provides AI capabilities including:
//! - Chapter summarization
//! - Context-aware chat with spoiler protection
//! - Semantic search via vector embeddings
//! - Reading analytics and insights

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_errors_doc,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::struct_excessive_bools,
    clippy::missing_const_for_fn
)]

pub mod cache;
pub mod providers;
pub mod settings;
pub mod traits;
pub mod embedding;

pub use settings::AiSettings;
pub use traits::LLMProvider;

use anyhow::Error;
use serde::{Deserialize, Serialize};

/// Result type for AI operations
pub type AiResult<T> = Result<T, Error>;

/// AI-generated content with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub content: String,
    pub model: String,
    pub provider: String,
    pub timestamp: i64,
    pub cached: bool,
}

/// Context for AI requests (respects spoiler protection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiContext {
    pub document_path: String,
    pub current_page: usize,
    pub total_pages: usize,
    pub text_before_current_page: String,
    pub reading_position_percent: f32,
}

impl AiContext {
    /// Create a new AI context with spoiler protection
    #[must_use]
    pub fn new(document_path: String, current_page: usize, total_pages: usize) -> Self {
        Self {
            document_path,
            current_page,
            total_pages,
            text_before_current_page: String::new(),
            reading_position_percent: if total_pages > 0 {
                (current_page as f32 / total_pages as f32) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Check if a page is safe to include (not a spoiler)
    #[must_use]
    pub fn is_page_safe(&self, page: usize) -> bool {
        page <= self.current_page
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_context_creation() {
        let ctx = AiContext::new("/books/test.epub".into(), 5, 20);
        assert_eq!(ctx.current_page, 5);
        assert_eq!(ctx.total_pages, 20);
        assert!(ctx.is_page_safe(5));
        assert!(ctx.is_page_safe(4));
        assert!(!ctx.is_page_safe(6));
    }

    #[test]
    fn test_reading_position_calculation() {
        let ctx = AiContext::new("/books/test.epub".into(), 5, 20);
        assert!((ctx.reading_position_percent - 25.0).abs() < 0.01);
    }
}
