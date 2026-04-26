//! Reading time estimation module
//!
//! This module provides functionality for estimating reading time based on
//! document word count and configurable reading speed. It supports different
//! reading speeds for various document types and languages.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Average words per minute for different reading speeds
pub const SLOW_WPM: u32 = 150;
pub const AVERAGE_WPM: u32 = 250;
pub const FAST_WPM: u32 = 350;

/// Estimated words per page for different document formats
pub const WORDS_PER_PAGE_EPUB: u32 = 280;
pub const WORDS_PER_PAGE_PDF: u32 = 320;
pub const WORDS_PER_PAGE_HTML: u32 = 300;

/// Reading speed configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ReadingSpeed {
    /// Slow reading (150 WPM) - for detailed study
    Slow,
    /// Average reading (250 WPM) - normal reading
    #[default]
    Average,
    /// Fast reading (350 WPM) - skimming
    Fast,
    /// Custom WPM value
    Custom(u32),
}

impl ReadingSpeed {
    /// Get the words per minute value
    #[inline]
    #[must_use]
    pub const fn wpm(&self) -> u32 {
        match self {
            ReadingSpeed::Slow => SLOW_WPM,
            ReadingSpeed::Average => AVERAGE_WPM,
            ReadingSpeed::Fast => FAST_WPM,
            ReadingSpeed::Custom(wpm) => *wpm,
        }
    }

    /// Get the WPM value as a descriptive string
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            ReadingSpeed::Slow => "slow (150 WPM)",
            ReadingSpeed::Average => "average (250 WPM)",
            ReadingSpeed::Fast => "fast (350 WPM)",
            ReadingSpeed::Custom(_wpm) => {
                // For custom values, we can't return a &'static str
                // So we use a const description
                "custom"
            }
        }
    }
}

/// Calculate estimated reading time from word count
///
/// # Arguments
/// * `word_count` - The number of words in the document
/// * `speed` - The reading speed to use for estimation
///
/// # Returns
/// A `Duration` representing the estimated reading time
#[inline]
#[must_use]
pub fn estimate_from_word_count(word_count: u32, speed: ReadingSpeed) -> Duration {
    let minutes = word_count as f32 / speed.wpm() as f32;
    Duration::from_secs((minutes * 60.0).ceil() as u64)
}

/// Calculate estimated reading time from page count
///
/// Uses format-specific words per page estimates when the format is known.
///
/// # Arguments
/// * `page_count` - The number of pages
/// * `format` - The document format (optional)
/// * `speed` - The reading speed to use for estimation
///
/// # Returns
/// A `Duration` representing the estimated reading time
#[inline]
#[must_use]
pub fn estimate_from_page_count(
    page_count: u32,
    format: Option<&str>,
    speed: ReadingSpeed,
) -> Duration {
    let words_per_page = match format {
        Some("epub") => WORDS_PER_PAGE_EPUB,
        Some("pdf") => WORDS_PER_PAGE_PDF,
        Some("html") | Some("htm") => WORDS_PER_PAGE_HTML,
        _ => WORDS_PER_PAGE_EPUB, // Default to EPUB estimate
    };
    let word_count = page_count * words_per_page;
    estimate_from_word_count(word_count, speed)
}

/// Format a duration as a human-readable string
///
/// Returns strings like:
/// - "< 1 min" for very short readings
/// - "5 min" for short readings
/// - "1h 30m" for longer readings
/// - "3h" for multi-hour readings
#[must_use]
pub fn format_duration(duration: &Duration) -> String {
    let total_minutes = duration.as_secs() / 60;

    if total_minutes == 0 {
        return "< 1 min".to_string();
    }

    if total_minutes < 60 {
        return format!("{} min", total_minutes);
    }

    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    if minutes == 0 {
        format!("{}h", hours)
    } else {
        format!("{}h {}m", hours, minutes)
    }
}

/// Estimated remaining reading time based on progress
///
/// # Arguments
/// * `total_time` - The total estimated reading time
/// * `progress` - The reading progress as a fraction (0.0 to 1.0)
///
/// # Returns
/// A `Duration` representing the remaining reading time
#[inline]
#[must_use]
pub fn remaining_time(total_time: Duration, progress: f32) -> Duration {
    let progress = progress.clamp(0.0, 1.0);
    let remaining_fraction = 1.0 - progress;
    Duration::from_secs((total_time.as_secs_f32() * remaining_fraction) as u64)
}

/// Count words in text content
///
/// This is a simple word counter that splits on whitespace.
/// For more accurate counting in specific formats, use format-specific tools.
#[inline]
#[must_use]
pub fn count_words(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_from_word_count() {
        // 2500 words at 250 WPM = 10 minutes
        let time = estimate_from_word_count(2500, ReadingSpeed::Average);
        assert_eq!(time, Duration::from_secs(600));

        // 2500 words at 150 WPM = 16.67 minutes = 1000s
        let time_slow = estimate_from_word_count(2500, ReadingSpeed::Slow);
        assert_eq!(time_slow, Duration::from_secs(1000));
    }

    #[test]
    fn test_estimate_from_page_count() {
        // 100 PDF pages at 250 WPM
        // 100 * 320 words / 250 WPM = 128 minutes
        let time = estimate_from_page_count(100, Some("pdf"), ReadingSpeed::Average);
        assert_eq!(time, Duration::from_secs(7680));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(&Duration::from_secs(30)), "< 1 min");
        assert_eq!(format_duration(&Duration::from_secs(300)), "5 min");
        assert_eq!(format_duration(&Duration::from_secs(3600)), "1h");
        assert_eq!(format_duration(&Duration::from_secs(5400)), "1h 30m");
        assert_eq!(format_duration(&Duration::from_secs(7200)), "2h");
    }

    #[test]
    fn test_remaining_time() {
        let total = Duration::from_secs(3600); // 1 hour
        assert_eq!(remaining_time(total, 0.0), total);
        assert_eq!(remaining_time(total, 0.5), Duration::from_secs(1800));
        assert_eq!(remaining_time(total, 1.0), Duration::from_secs(0));
        assert_eq!(remaining_time(total, 1.5), Duration::from_secs(0)); // Clamped
        assert_eq!(remaining_time(total, -0.5), Duration::from_secs(3600)); // Clamped
    }

    #[test]
    fn test_count_words() {
        assert_eq!(count_words("Hello world"), 2);
        assert_eq!(count_words("The quick brown fox jumps over the lazy dog"), 9);
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("   "), 0);
    }

    #[test]
    fn test_reading_speed_wpm() {
        assert_eq!(ReadingSpeed::Slow.wpm(), 150);
        assert_eq!(ReadingSpeed::Average.wpm(), 250);
        assert_eq!(ReadingSpeed::Fast.wpm(), 350);
        assert_eq!(ReadingSpeed::Custom(500).wpm(), 500);
    }
}
