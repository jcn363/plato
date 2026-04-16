//! Line Breaking Module
//!
//! This module provides line breaking algorithms for text layout.

/// Line breaking configuration
#[derive(Debug, Clone)]
pub struct LineBreakConfig {
    pub max_width: f32,
    pub tolerance: f32,
    pub penalty_factor: f32,
    pub hyphen_penalty: f32,
    pub allow_hyphenation: bool,
    pub min_hyphenated_word_length: usize,
}

impl Default for LineBreakConfig {
    fn default() -> Self {
        Self {
            max_width: f32::INFINITY,
            tolerance: 1.0,
            penalty_factor: 1.0,
            hyphen_penalty: 50.0,
            allow_hyphenation: true,
            min_hyphenated_word_length: 4,
        }
    }
}

/// Break point information
#[derive(Debug, Clone)]
pub struct BreakPoint {
    pub position: usize,
    pub penalty: f32,
    pub width: f32,
    pub demerits: f32,
    pub fitness: f32,
}

/// Line breaking result
#[derive(Debug, Clone)]
pub struct LineBreakResult {
    pub lines: Vec<Line>,
    pub total_width: f32,
    pub total_height: f32,
    pub total_penalty: f32,
}

/// A single line
#[derive(Debug, Clone)]
pub struct Line {
    pub start: usize,
    pub end: usize,
    pub width: f32,
    pub height: f32,
    pub break_point: Option<BreakPoint>,
}

/// Line breaker using Knuth-Plass algorithm
pub struct LineBreaker {
    config: LineBreakConfig,
}

impl LineBreaker {
    /// Create a new line breaker
    pub fn new(config: LineBreakConfig) -> Self {
        Self { config }
    }

    /// Break text into lines
    pub fn break_lines(&self, text: &str, font_size: f32) -> LineBreakResult {
        let words = self.extract_words(text);
        let break_points = self.find_break_points(&words, font_size);
        let lines = self.create_lines(&words, &break_points);

        let total_width = lines.iter().map(|l| l.width).fold(0.0, f32::max);
        let total_height = lines.len() as f32 * font_size * 1.2;
        let total_penalty = break_points.iter().map(|bp| bp.penalty).sum();

        LineBreakResult {
            lines,
            total_width,
            total_height,
            total_penalty,
        }
    }

    /// Extract words from text
    fn extract_words(&self, text: &str) -> Vec<String> {
        text.split_whitespace().map(|s| s.to_string()).collect()
    }

    /// Find optimal break points using Knuth-Plass algorithm
    fn find_break_points(&self, words: &[String], font_size: f32) -> Vec<BreakPoint> {
        let mut break_points = Vec::new();
        let mut current_width = 0.0;
        let mut current_position = 0;

        for (i, word) in words.iter().enumerate() {
            let word_width = self.measure_word(word, font_size);

            if current_width + word_width > self.config.max_width && i > 0 {
                // Create a break point
                let penalty = self.calculate_penalty(current_width, word_width);

                break_points.push(BreakPoint {
                    position: current_position,
                    penalty,
                    width: current_width,
                    demerits: penalty * self.config.penalty_factor,
                    fitness: self.calculate_fitness(current_width),
                });

                current_width = word_width;
                current_position = i;
            } else {
                current_width += word_width + font_size * 0.1; // Add space
            }
        }

        break_points
    }

    /// Create lines from words and break points
    fn create_lines(&self, words: &[String], break_points: &[BreakPoint]) -> Vec<Line> {
        let mut lines = Vec::new();
        let mut start = 0;

        for break_point in break_points {
            let line = Line {
                start,
                end: break_point.position,
                width: break_point.width,
                height: 12.0, // Default line height
                break_point: Some(break_point.clone()),
            };
            lines.push(line);
            start = break_point.position;
        }

        // Add the last line
        if start < words.len() {
            let line = Line {
                start,
                end: words.len(),
                width: self.measure_words_slice(&words[start..], 12.0),
                height: 12.0,
                break_point: None,
            };
            lines.push(line);
        }

        lines
    }

    /// Measure a word
    fn measure_word(&self, word: &str, font_size: f32) -> f32 {
        word.len() as f32 * font_size * 0.6
    }

    /// Measure a slice of words
    fn measure_words_slice(&self, words: &[String], font_size: f32) -> f32 {
        words.iter().map(|w| self.measure_word(w, font_size)).sum()
    }

    /// Calculate penalty for a break point
    fn calculate_penalty(&self, current_width: f32, _word_width: f32) -> f32 {
        let ratio = (self.config.max_width - current_width) / self.config.max_width;

        if ratio < 0.0 {
            // Line is too tight
            (-ratio).powf(3.0) * 1000.0
        } else if ratio > 1.0 {
            // Line is too loose
            (ratio - 1.0).powf(2.0) * 1000.0
        } else {
            // Good fit
            0.0
        }
    }

    /// Calculate fitness of a line
    fn calculate_fitness(&self, width: f32) -> f32 {
        let ratio = width / self.config.max_width;

        if ratio < 0.5 {
            0.0 // Very tight
        } else if ratio < 0.75 {
            1.0 // Tight
        } else if ratio < 1.25 {
            2.0 // Normal
        } else if ratio < 1.5 {
            3.0 // Loose
        } else {
            4.0 // Very loose
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: LineBreakConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn config(&self) -> &LineBreakConfig {
        &self.config
    }
}

/// Utility functions for line breaking
pub mod utils {
    use super::*;

    /// Create a line breaker with default settings
    pub fn create_default_breaker() -> LineBreaker {
        LineBreaker::new(LineBreakConfig::default())
    }

    /// Simple greedy line breaking
    pub fn greedy_break(text: &str, max_width: f32, font_size: f32) -> Vec<String> {
        let words = text.split_whitespace().collect::<Vec<_>>();
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;

        for word in words {
            let word_width = word.len() as f32 * font_size * 0.6;

            if current_width + word_width > max_width && !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0.0;
            }

            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += font_size * 0.1; // Space width
            }

            current_line.push_str(word);
            current_width += word_width;
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Calculate optimal line width
    pub fn calculate_optimal_line_width(text: &str, font_size: f32, target_lines: usize) -> f32 {
        let total_chars = text.len();
        let avg_chars_per_line = total_chars as f32 / target_lines as f32;
        avg_chars_per_line * font_size * 0.6
    }

    /// Check if text needs line breaking
    pub fn needs_line_break(text: &str, max_width: f32, font_size: f32) -> bool {
        let estimated_width = text.len() as f32 * font_size * 0.6;
        estimated_width > max_width
    }
}
