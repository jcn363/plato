//! Hyphenation Module
//!
//! This module provides hyphenation functionality for text layout.

use rustc_hash::FxHashMap;

/// Trait for hyphenation implementations
pub trait Hyphenate {
    fn hyphenate<'a>(&'a self, word: &'a str) -> Box<dyn Iterator<Item = &'a str> + 'a>;
}

/// Simple hyphenator implementation
pub struct SimpleHyphenator {
    _language: String,
}

impl SimpleHyphenator {
    pub fn new(language: String) -> Self {
        if language.is_empty() {
            return Self {
                _language: String::new(),
            };
        }
        Self {
            _language: language,
        }
    }
}

impl Hyphenate for SimpleHyphenator {
    fn hyphenate<'a>(&'a self, word: &'a str) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        // Simple hyphenation: just return the word unchanged
        Box::new(std::iter::once(word))
    }
}

/// Hyphenation configuration
#[derive(Debug, Clone)]
pub struct HyphenationConfig {
    pub language: String,
    pub min_word_length: usize,
    pub min_prefix_length: usize,
    pub min_suffix_length: usize,
    pub hyphen_char: char,
}

impl Default for HyphenationConfig {
    fn default() -> Self {
        Self {
            language: "en-US".to_string(),
            min_word_length: 4,
            min_prefix_length: 2,
            min_suffix_length: 2,
            hyphen_char: '-',
        }
    }
}

/// Hyphenation engine
pub struct HyphenationEngine {
    config: HyphenationConfig,
    hyphenators: FxHashMap<String, Box<dyn Hyphenate>>,
}

impl HyphenationEngine {
    /// Create a new hyphenation engine
    pub fn new(config: HyphenationConfig) -> Self {
        Self {
            config,
            hyphenators: FxHashMap::default(),
        }
    }

    /// Hyphenate a word
    pub fn hyphenate_word(&self, word: &str) -> Vec<String> {
        if word.len() < self.config.min_word_length {
            return vec![word.to_string()];
        }

        if let Some(hyphenator) = self.hyphenators.get(&self.config.language) {
            hyphenator.hyphenate(word).map(|s| s.to_string()).collect()
        } else {
            vec![word.to_string()]
        }
    }

    /// Find hyphenation points in a word
    pub fn find_hyphenation_points(&self, word: &str) -> Vec<usize> {
        let hyphenated = self.hyphenate_word(word);
        let mut points = Vec::new();
        let mut current_pos = 0;

        for part in hyphenated {
            current_pos += part.len();
            if current_pos < word.len() {
                points.push(current_pos);
            }
        }

        points
    }

    /// Check if a word should be hyphenated
    pub fn should_hyphenate(&self, word: &str) -> bool {
        word.len() >= self.config.min_word_length
    }

    /// Update configuration
    pub fn update_config(&mut self, config: HyphenationConfig) {
        self.config = config;
    }

    /// Get configuration
    pub fn config(&self) -> &HyphenationConfig {
        &self.config
    }
}

/// Utility functions for hyphenation
pub mod utils {
    use super::*;

    /// Create a hyphenation engine with default settings
    pub fn create_default_hyphenator() -> HyphenationEngine {
        HyphenationEngine::new(HyphenationConfig::default())
    }

    /// Hyphenate text with line breaks
    pub fn hyphenate_text(text: &str, engine: &HyphenationEngine) -> String {
        text.split_whitespace()
            .map(|word| {
                if engine.should_hyphenate(word) {
                    engine
                        .hyphenate_word(word)
                        .join(&engine.config().hyphen_char.to_string())
                } else {
                    word.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Find optimal hyphenation points for a word
    pub fn find_optimal_hyphenation_points(word: &str, engine: &HyphenationEngine) -> Vec<usize> {
        let points = engine.find_hyphenation_points(word);

        // Filter points based on minimum prefix and suffix length
        points
            .into_iter()
            .filter(|&point| {
                point >= engine.config().min_prefix_length
                    && (word.len() - point) >= engine.config().min_suffix_length
            })
            .collect()
    }
}
