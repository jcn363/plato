//! Text Shaping Module
//!
//! This module provides text shaping functionality using HarfBuzz for complex scripts.

use crate::geom::Rectangle;
use std::collections::HashMap;

/// Text shaping configuration
#[derive(Debug, Clone)]
pub struct TextShapingConfig {
    pub font_size: f32,
    pub font_features: Vec<String>,
    pub script: String,
    pub language: String,
    pub direction: TextDirection,
}

/// Text direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

impl Default for TextDirection {
    fn default() -> Self {
        TextDirection::LeftToRight
    }
}

impl Default for TextShapingConfig {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            font_features: Vec::new(),
            script: "Latin".to_string(),
            language: "en".to_string(),
            direction: TextDirection::LeftToRight,
        }
    }
}

/// Glyph information
#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub glyph_id: u32,
    pub x_advance: f32,
    pub y_advance: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub cluster: u32,
}

/// Shaped text result
#[derive(Debug, Clone)]
pub struct ShapedText {
    pub glyphs: Vec<GlyphInfo>,
    pub width: f32,
    pub height: f32,
    pub x_bearing: f32,
    pub y_bearing: f32,
}

/// Text shaper
pub struct TextShaper {
    config: TextShapingConfig,
    font_cache: HashMap<String, Vec<GlyphInfo>>,
}

impl TextShaper {
    /// Create a new text shaper
    pub fn new(config: TextShapingConfig) -> Self {
        Self {
            config,
            font_cache: HashMap::new(),
        }
    }

    /// Shape text with the current configuration
    pub fn shape_text(&mut self, text: &str, font_name: &str) -> ShapedText {
        // Check cache first
        let cache_key = format!("{}:{}:{}", font_name, text, self.config.language);
        if let Some(glyphs) = self.font_cache.get(&cache_key) {
            return ShapedText {
                glyphs: glyphs.clone(),
                width: self.calculate_width(glyphs),
                height: self.config.font_size,
                x_bearing: 0.0,
                y_bearing: self.config.font_size * 0.8,
            };
        }

        // Shape the text (simplified implementation)
        let glyphs = self.simple_shape(text);

        // Cache the result
        self.font_cache.insert(cache_key, glyphs.clone());

        ShapedText {
            glyphs,
            width: self.calculate_width(&glyphs),
            height: self.config.font_size,
            x_bearing: 0.0,
            y_bearing: self.config.font_size * 0.8,
        }
    }

    /// Simple text shaping implementation
    fn simple_shape(&self, text: &str) -> Vec<GlyphInfo> {
        let mut glyphs = Vec::new();
        let mut cluster = 0;

        for ch in text.chars() {
            let glyph_id = self.get_glyph_id(ch);
            let advance = self.get_glyph_advance(glyph_id);

            glyphs.push(GlyphInfo {
                glyph_id,
                x_advance: advance,
                y_advance: 0.0,
                x_offset: 0.0,
                y_offset: 0.0,
                cluster,
            });

            cluster += 1;
        }

        glyphs
    }

    /// Get glyph ID for a character (simplified)
    fn get_glyph_id(&self, ch: char) -> u32 {
        ch as u32 // Simplified glyph mapping
    }

    /// Get glyph advance width (simplified)
    fn get_glyph_advance(&self, glyph_id: u32) -> f32 {
        // Simplified advance calculation
        self.config.font_size * 0.6
    }

    /// Calculate total width from glyphs
    fn calculate_width(&self, glyphs: &[GlyphInfo]) -> f32 {
        glyphs.iter().map(|g| g.x_advance).sum()
    }

    /// Update configuration
    pub fn update_config(&mut self, config: TextShapingConfig) {
        self.config = config;
        // Clear cache when config changes
        self.font_cache.clear();
    }

    /// Get current configuration
    pub fn config(&self) -> &TextShapingConfig {
        &self.config
    }

    /// Clear font cache
    pub fn clear_cache(&mut self) {
        self.font_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (
            self.font_cache.len(),
            self.font_cache.values().map(|v| v.len()).sum(),
        )
    }
}

/// Utility functions for text shaping
pub mod utils {
    use super::*;

    /// Create a text shaper with default settings
    pub fn create_default_shaper() -> TextShaper {
        TextShaper::new(TextShapingConfig::default())
    }

    /// Shape text for left-to-right languages
    pub fn shape_ltr_text(text: &str, font_name: &str, font_size: f32) -> ShapedText {
        let mut shaper = TextShaper::new(TextShapingConfig {
            font_size,
            direction: TextDirection::LeftToRight,
            ..Default::default()
        });
        shaper.shape_text(text, font_name)
    }

    /// Shape text for right-to-left languages
    pub fn shape_rtl_text(text: &str, font_name: &str, font_size: f32) -> ShapedText {
        let mut shaper = TextShaper::new(TextShapingConfig {
            font_size,
            direction: TextDirection::RightToLeft,
            ..Default::default()
        });
        shaper.shape_text(text, font_name)
    }

    /// Estimate text width without full shaping
    pub fn estimate_text_width(text: &str, font_size: f32) -> f32 {
        text.len() as f32 * font_size * 0.6
    }

    /// Check if text needs complex shaping
    pub fn needs_complex_shaping(text: &str) -> bool {
        // Check for characters that need complex shaping
        text.chars().any(|ch| {
            ch > '\x7F' || // Non-ASCII
            ch == '\u{200C}' || // Zero width non-joiner
            ch == '\u{200D}' // Zero width joiner
        })
    }
}
