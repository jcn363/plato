//! Text Renderer Module
//!
//! This module provides text rendering functionality for the HTML engine.

use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::geom::Rectangle;
use rustc_hash::FxHashMap;

/// Text rendering configuration
#[derive(Debug, Clone)]
pub struct TextRenderConfig {
    pub font_size: f32,
    pub color: Color,
    pub background_color: Color,
    pub anti_aliasing: bool,
    pub subpixel_rendering: bool,
    pub hinting: bool,
}

impl Default for TextRenderConfig {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            color: crate::color::BLACK,
            background_color: crate::color::WHITE,
            anti_aliasing: true,
            subpixel_rendering: true,
            hinting: true,
        }
    }
}

/// Text rendering result
#[derive(Debug, Clone)]
pub struct TextRenderResult {
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
    pub glyph_count: usize,
}

/// Text renderer
pub struct TextRenderer {
    config: TextRenderConfig,
    glyph_cache: FxHashMap<u32, GlyphRenderData>,
    cache_hits: u64,
    cache_misses: u64,
}

/// Glyph rendering data
#[derive(Debug, Clone)]
pub struct GlyphRenderData {
    pub glyph_id: u32,
    pub width: f32,
    pub height: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub advance: f32,
    pub bitmap: Option<Vec<u8>>,
}

impl TextRenderer {
    /// Create a new text renderer
    pub fn new(config: TextRenderConfig) -> Self {
        Self {
            config,
            glyph_cache: FxHashMap::default(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// Render text to framebuffer
    pub fn render_text(
        &mut self,
        text: &str,
        framebuffer: &mut dyn Framebuffer,
        rect: Rectangle,
    ) -> TextRenderResult {
        let start_x = rect.min.x as f32;
        let mut x = start_x;
        let y_baseline = rect.min.y as f32 + self.config.font_size * 0.8; // Baseline
        let mut glyph_count = 0;

        for ch in text.chars() {
            let glyph_id = self.get_glyph_id(ch);
            let advance = self.get_or_create_glyph_data(glyph_id).advance;
            let bitmap = self.get_or_create_glyph_data(glyph_id).bitmap.clone();

            if let Some(ref bm) = bitmap {
                self.render_glyph(bm, framebuffer, x, y_baseline);
            }

            x += advance;
            glyph_count += 1;
        }

        let width = x - rect.min.x as f32;
        let height = self.config.font_size;
        let baseline = self.config.font_size * 0.8;

        TextRenderResult {
            width,
            height,
            baseline,
            glyph_count,
        }
    }

    /// Get glyph ID for a character
    fn get_glyph_id(&self, ch: char) -> u32 {
        ch as u32 // Simplified glyph mapping
    }

    /// Get or create glyph data
    ///
    /// Tracks cache hits and misses for performance monitoring.
    fn get_or_create_glyph_data(&mut self, glyph_id: u32) -> &GlyphRenderData {
        if !self.glyph_cache.contains_key(&glyph_id) {
            self.cache_misses += 1;
            let glyph_data = self.create_glyph_data(glyph_id);
            self.glyph_cache.insert(glyph_id, glyph_data);
        } else {
            self.cache_hits += 1;
        }

        self.glyph_cache
            .get(&glyph_id)
            .expect("glyph_id should be in cache after insertion")
    }

    /// Create glyph data
    fn create_glyph_data(&self, glyph_id: u32) -> GlyphRenderData {
        let width = self.config.font_size * 0.6;
        let height = self.config.font_size;
        let advance = width;

        GlyphRenderData {
            glyph_id,
            width,
            height,
            bearing_x: 0.0,
            bearing_y: height * 0.8,
            advance,
            bitmap: self.create_glyph_bitmap(glyph_id),
        }
    }

    /// Create glyph bitmap
    fn create_glyph_bitmap(&self, _glyph_id: u32) -> Option<Vec<u8>> {
        // Simplified bitmap creation
        let width = (self.config.font_size * 0.6) as usize;
        let height = self.config.font_size as usize;
        let mut bitmap = vec![0u8; width * height];

        // Create a simple rectangular glyph
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                if x == 0 || x == width - 1 || y == height - 1 {
                    bitmap[idx] = 255; // Black pixel
                } else {
                    bitmap[idx] = 0; // White pixel
                }
            }
        }

        Some(bitmap)
    }

    /// Render glyph to framebuffer
    fn render_glyph(&self, bitmap: &[u8], framebuffer: &mut dyn Framebuffer, x: f32, y: f32) {
        let start_x = x as i32;
        let start_y = y as i32;
        let width = (bitmap.len() as f32).sqrt() as i32;
        let height = width;

        for yi in 0..height {
            for xi in 0..width {
                let idx = (yi * width + xi) as usize;
                let alpha = bitmap[idx];

                if alpha > 0 {
                    let pixel_x = start_x + xi;
                    let pixel_y = start_y + yi;

                    // Set pixel with color and alpha
                    let color = self.config.color;
                    framebuffer.set_pixel(pixel_x as u32, pixel_y as u32, color);
                }
            }
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: TextRenderConfig) {
        self.config = config;
        // Clear cache when config changes
        self.glyph_cache.clear();
    }

    /// Get current configuration
    pub fn config(&self) -> &TextRenderConfig {
        &self.config
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache_hits = 0;
        self.cache_misses = 0;
        self.glyph_cache.clear();
    }

    /// Get cache statistics
    ///
    /// Returns (cache_size, cache_memory_bytes, hit_rate).
    pub fn cache_stats(&self) -> (usize, usize, f32) {
        let memory: usize = self
            .glyph_cache
            .values()
            .map(|g| g.bitmap.as_ref().map_or(0, |b| b.len()))
            .sum();

        let total_lookups = self.cache_hits + self.cache_misses;
        let hit_rate = if total_lookups > 0 {
            self.cache_hits as f32 / total_lookups as f32
        } else {
            0.0
        };

        (self.glyph_cache.len(), memory, hit_rate)
    }
}

/// Utility functions for text rendering
pub mod utils {
    use super::*;

    /// Create a text renderer with default settings
    pub fn create_default_renderer() -> TextRenderer {
        TextRenderer::new(TextRenderConfig::default())
    }

    /// Render text with a specific color
    pub fn render_colored_text(
        text: &str,
        color: Color,
        framebuffer: &mut dyn Framebuffer,
        rect: Rectangle,
    ) -> TextRenderResult {
        let mut renderer = TextRenderer::new(TextRenderConfig {
            color,
            ..Default::default()
        });
        renderer.render_text(text, framebuffer, rect)
    }

    /// Estimate text rendering bounds
    pub fn estimate_text_bounds(text: &str, font_size: f32) -> Rectangle {
        let width = text.len() as f32 * font_size * 0.6;
        let height = font_size;
        Rectangle::new(
            crate::geom::Point::new(0, 0),
            crate::geom::Point::new(width as i32, height as i32),
        )
    }

    /// Check if text fits in a rectangle
    pub fn text_fits_in_rect(text: &str, font_size: f32, rect: Rectangle) -> bool {
        let bounds = estimate_text_bounds(text, font_size);
        bounds.width() <= rect.width() && bounds.height() <= rect.height()
    }

    /// Calculate text rendering quality metrics
    pub fn calculate_rendering_quality(renderer: &TextRenderer) -> RenderingQuality {
        let (cache_size, cache_memory, hit_rate) = renderer.cache_stats();

        RenderingQuality {
            cache_hit_rate: hit_rate,
            cache_size,
            cache_memory_mb: cache_memory / (1024 * 1024),
            anti_aliasing_enabled: renderer.config().anti_aliasing,
            subpixel_rendering_enabled: renderer.config().subpixel_rendering,
        }
    }
}

/// Rendering quality metrics
#[derive(Debug, Clone)]
pub struct RenderingQuality {
    pub cache_hit_rate: f32,
    pub cache_size: usize,
    pub cache_memory_mb: usize,
    pub anti_aliasing_enabled: bool,
    pub subpixel_rendering_enabled: bool,
}
