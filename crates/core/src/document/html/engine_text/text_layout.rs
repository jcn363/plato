//! Text Layout Module
//!
//! This module provides text layout algorithms and utilities for the HTML rendering engine.

use crate::geom::{Point, Rectangle};
use crate::document::html::layout::{TextElement, TextAlign, FontKind};
use crate::document::html::StyleData;
use std::collections::VecDeque;

/// Text layout configuration
#[derive(Debug, Clone)]
pub struct TextLayoutConfig {
    pub font_size: f32,
    pub line_height: f32,
    pub text_align: TextAlign,
    pub font_kind: FontKind,
    pub max_width: f32,
    pub letter_spacing: f32,
    pub word_spacing: f32,
}

impl Default for TextLayoutConfig {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            line_height: 1.2,
            text_align: TextAlign::Left,
            font_kind: FontKind::Serif,
            max_width: f32::INFINITY,
            letter_spacing: 0.0,
            word_spacing: 0.0,
        }
    }
}

/// Text layout result
#[derive(Debug, Clone)]
pub struct TextLayoutResult {
    pub lines: Vec<TextLine>,
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
}

/// A single line of laid out text
#[derive(Debug, Clone)]
pub struct TextLine {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
    pub elements: Vec<TextElement>,
}

/// Text layout engine
pub struct TextLayoutEngine {
    config: TextLayoutConfig,
}

impl TextLayoutEngine {
    /// Create a new text layout engine
    pub fn new(config: TextLayoutConfig) -> Self {
        Self { config }
    }

    /// Layout text with the current configuration
    pub fn layout_text(&self, text: &str, style: &StyleData) -> TextLayoutResult {
        let mut lines = Vec::new();
        let mut current_y = 0.0;
        let max_width = self.config.max_width;

        // Split text into words and layout them
        let words = self.split_into_words(text);
        let mut current_line_words = Vec::new();
        let mut current_line_width = 0.0;

        for word in &words {
            let word_width = self.measure_word(word);
            
            if current_line_width + word_width > max_width && !current_line_words.is_empty() {
                // Current line is full, create a line
                let line = self.create_line(&current_line_words, current_line_width, current_y);
                lines.push(line);
                current_y += self.config.line_height * self.config.font_size;
                
                // Start new line
                current_line_words.clear();
                current_line_width = 0.0;
            }
            
            current_line_words.push(word.clone());
            current_line_width += word_width + self.config.word_spacing;
        }

        // Add the last line if it has content
        if !current_line_words.is_empty() {
            let line = self.create_line(&current_line_words, current_line_width, current_y);
            lines.push(line);
        }

        let total_height = lines.len() as f32 * self.config.line_height * self.config.font_size;
        let baseline = self.config.font_size * 0.8; // Approximate baseline

        TextLayoutResult {
            lines,
            width: max_width,
            height: total_height,
            baseline,
        }
    }

    /// Split text into words
    fn split_into_words(&self, text: &str) -> Vec<String> {
        text.split_whitespace().map(|s| s.to_string()).collect()
    }

    /// Measure the width of a word
    fn measure_word(&self, word: &str) -> f32 {
        // Simple approximation: character count * average character width
        word.len() as f32 * self.config.font_size * 0.6
    }

    /// Create a text line from words
    fn create_line(&self, words: &[String], line_width: f32, y: f32) -> TextLine {
        let text = words.join(" ");
        let x = self.calculate_line_x(line_width);
        let height = self.config.line_height * self.config.font_size;
        let baseline = height * 0.8;

        TextLine {
            text,
            x,
            y,
            width: line_width,
            height,
            baseline,
            elements: Vec::new(), // TODO: Create TextElement instances
        }
    }

    /// Calculate the x position for a line based on text alignment
    fn calculate_line_x(&self, line_width: f32) -> f32 {
        match self.config.text_align {
            TextAlign::Left => 0.0,
            TextAlign::Center => (self.config.max_width - line_width) / 2.0,
            TextAlign::Right => self.config.max_width - line_width,
            TextAlign::Justify => 0.0, // Handled differently for justification
        }
    }

    /// Update the layout configuration
    pub fn update_config(&mut self, config: TextLayoutConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn config(&self) -> &TextLayoutConfig {
        &self.config
    }
}

/// Utility functions for text layout
pub mod utils {
    use super::*;

    /// Calculate the optimal line height for a given font size
    pub fn calculate_optimal_line_height(font_size: f32) -> f32 {
        font_size * 1.2 // Standard line height ratio
    }

    /// Calculate the width of a text string
    pub fn calculate_text_width(text: &str, font_size: f32) -> f32 {
        text.len() as f32 * font_size * 0.6 // Simple approximation
    }

    /// Wrap text to fit within a maximum width
    pub fn wrap_text(text: &str, max_width: f32, font_size: f32) -> Vec<String> {
        let engine = TextLayoutEngine::new(TextLayoutConfig {
            max_width,
            font_size,
            ..Default::default()
        });
        
        let result = engine.layout_text(text, &StyleData::default());
        result.lines.into_iter().map(|line| line.text).collect()
    }

    /// Truncate text to fit within a maximum width with ellipsis
    pub fn truncate_text(text: &str, max_width: f32, font_size: f32) -> String {
        if calculate_text_width(text, font_size) <= max_width {
            return text.to_string();
        }

        let mut truncated = String::new();
        let ellipsis_width = calculate_text_width("...", font_size);
        let mut current_width = 0.0;

        for ch in text.chars() {
            let char_width = calculate_text_width(&ch.to_string(), font_size);
            if current_width + char_width + ellipsis_width > max_width {
                break;
            }
            truncated.push(ch);
            current_width += char_width;
        }

        if truncated.is_empty() {
            return "...".to_string();
        }

        truncated.push_str("...");
        truncated
    }
}
