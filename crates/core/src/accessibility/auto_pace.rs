//! Auto-Pace Feature Implementation
//!
//! Implements automatic page turning with adjustable speed for hands-free reading.
//! Users can set a words-per-minute (WPM) rate and the reader will automatically
//! advance pages to maintain that reading pace.

use std::time::{Duration, Instant};

/// Auto-pace controller for automatic page turning
pub struct AutoPace {
    /// Words per minute target
    wpm: u32,
    /// Estimated words per page
    words_per_page: usize,
    /// Last page turn time
    last_turn: Option<Instant>,
    /// Whether auto-pace is active
    active: bool,
    /// Page turn interval calculated from WPM
    interval: Duration,
}

impl AutoPace {
    /// Create a new AutoPace controller
    pub fn new(wpm: u32) -> Self {
        let wpm = wpm.clamp(100, 600);
        let words_per_minute = wpm as f32;
        // Assume average 250 words per page
        let pages_per_minute = words_per_minute / 250.0;
        let interval_seconds = 60.0 / pages_per_minute;

        Self {
            wpm,
            words_per_page: 250,
            last_turn: None,
            active: false,
            interval: Duration::from_secs_f32(interval_seconds),
        }
    }

    /// Start auto-pace
    pub fn start(&mut self) {
        self.active = true;
        self.last_turn = Some(Instant::now());
    }

    /// Stop auto-pace
    pub fn stop(&mut self) {
        self.active = false;
        self.last_turn = None;
    }

    /// Check if auto-pace is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Update the WPM setting
    pub fn set_wpm(&mut self, wpm: u32) {
        self.wpm = wpm.clamp(100, 600);
        let words_per_minute = self.wpm as f32;
        let pages_per_minute = words_per_minute / self.words_per_page as f32;
        let interval_seconds = 60.0 / pages_per_minute;
        self.interval = Duration::from_secs_f32(interval_seconds);
    }

    /// Get current WPM
    pub fn wpm(&self) -> u32 {
        self.wpm
    }

    /// Check if it's time to turn the page
    pub fn should_turn_page(&self) -> bool {
        if !self.active {
            return false;
        }

        if let Some(last) = self.last_turn {
            last.elapsed() >= self.interval
        } else {
            false
        }
    }

    /// Signal that a page was turned
    pub fn page_turned(&mut self) {
        self.last_turn = Some(Instant::now());
    }

    /// Get the page turn interval
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Get remaining time until next page turn
    pub fn time_until_next_turn(&self) -> Option<Duration> {
        if !self.active {
            return None;
        }

        if let Some(last) = self.last_turn {
            let elapsed = last.elapsed();
            if elapsed >= self.interval {
                Some(Duration::from_secs(0))
            } else {
                Some(self.interval - elapsed)
            }
        } else {
            None
        }
    }

    /// Estimate words in text (simple whitespace splitting)
    pub fn estimate_words(text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// Update words per page estimate based on actual content
    pub fn update_words_per_page(&mut self, words: usize) {
        if words > 0 {
            self.words_per_page = words.clamp(50, 1000);
            // Recalculate interval
            let words_per_minute = self.wpm as f32;
            let pages_per_minute = words_per_minute / self.words_per_page as f32;
            let interval_seconds = 60.0 / pages_per_minute;
            self.interval = Duration::from_secs_f32(interval_seconds);
        }
    }
}

/// Calculate reading time for a given amount of text
///
/// # Arguments
/// * `text` - The text to analyze
/// * `wpm` - Words per minute reading speed
///
/// # Returns
/// Estimated reading time in seconds
pub fn calculate_reading_time(text: &str, wpm: u32) -> Duration {
    let word_count = AutoPace::estimate_words(text);
    let minutes = word_count as f32 / wpm as f32;
    Duration::from_secs_f32(minutes * 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_pace_creation() {
        let ap = AutoPace::new(300);
        assert_eq!(ap.wpm(), 300);
        assert!(!ap.is_active());
    }

    #[test]
    fn test_auto_pace_start_stop() {
        let mut ap = AutoPace::new(300);
        ap.start();
        assert!(ap.is_active());
        ap.stop();
        assert!(!ap.is_active());
    }

    #[test]
    fn test_auto_pace_wpm_clamp() {
        let ap = AutoPace::new(1000); // Above max
        assert_eq!(ap.wpm(), 600); // Should be clamped to max

        let ap = AutoPace::new(50); // Below min
        assert_eq!(ap.wpm(), 100); // Should be clamped to min
    }

    #[test]
    fn test_estimate_words() {
        let text = "Hello world this is a test";
        let words = AutoPace::estimate_words(text);
        assert_eq!(words, 6);
    }

    #[test]
    fn test_should_turn_page() {
        let mut ap = AutoPace::new(300);
        ap.start();

        // Should not turn immediately
        assert!(!ap.should_turn_page());

        // After waiting, should turn
        std::thread::sleep(ap.interval() + Duration::from_millis(100));
        assert!(ap.should_turn_page());
    }
}
