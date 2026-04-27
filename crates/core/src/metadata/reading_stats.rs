//! Reading progress statistics and tracking
//!
//! Provides reading speed calculation, time estimates, and progress tracking.

use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::info::ReaderInfo;

/// Page turn event with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PageTurnEvent {
    /// Page number
    pub page: usize,
    /// Timestamp when page was turned
    pub timestamp: NaiveDateTime,
    /// Session duration in seconds since last page turn
    pub session_duration_seconds: u64,
}

impl Default for PageTurnEvent {
    fn default() -> Self {
        Self {
            page: 0,
            timestamp: Local::now().naive_local(),
            session_duration_seconds: 0,
        }
    }
}

/// Reading statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ReadingStatistics {
    /// Page turn history (timestamp -> event)
    pub page_turns: BTreeMap<NaiveDateTime, PageTurnEvent>,
    /// Total pages read
    pub total_pages_read: usize,
    /// Total reading time in seconds
    pub total_reading_time_seconds: u64,
    /// Average reading speed (pages per minute)
    pub average_pages_per_minute: f32,
    /// Average reading speed (words per minute)
    pub average_words_per_minute: f32,
    /// Estimated words per page (for calculation)
    pub estimated_words_per_page: u32,
    /// Reading streak (consecutive days)
    pub reading_streak_days: u32,
    /// Last reading date
    pub last_reading_date: Option<NaiveDateTime>,
}

impl ReadingStatistics {
    /// Create new reading statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a page turn event
    pub fn record_page_turn(&mut self, page: usize, estimated_words_per_page: u32) {
        let now = Local::now().naive_local();
        let session_duration = self.calculate_session_duration(&now);

        let event = PageTurnEvent {
            page,
            timestamp: now,
            session_duration_seconds: session_duration,
        };

        self.page_turns.insert(now, event);
        self.total_pages_read += 1;
        self.total_reading_time_seconds += session_duration;
        self.estimated_words_per_page = estimated_words_per_page;
        self.last_reading_date = Some(now);

        self.recalculate_averages();
        self.update_reading_streak();
    }

    /// Calculate reading speed in pages per minute
    pub fn pages_per_minute(&self) -> f32 {
        if self.total_reading_time_seconds == 0 {
            return 0.0;
        }
        let minutes = self.total_reading_time_seconds as f32 / 60.0;
        self.total_pages_read as f32 / minutes
    }

    /// Calculate reading speed in words per minute
    pub fn words_per_minute(&self) -> f32 {
        if self.total_reading_time_seconds == 0 || self.estimated_words_per_page == 0 {
            return 0.0;
        }
        let total_words = self.total_pages_read as f32 * self.estimated_words_per_page as f32;
        let minutes = self.total_reading_time_seconds as f32 / 60.0;
        total_words / minutes
    }

    /// Estimate time to finish remaining pages
    pub fn estimate_time_to_finish(&self, remaining_pages: usize) -> u64 {
        let ppm = self.pages_per_minute();
        if ppm <= 0.0 {
            return 0;
        }
        let minutes = remaining_pages as f32 / ppm;
        (minutes * 60.0) as u64
    }

    /// Get reading progress percentage
    pub fn progress_percentage(&self, current_page: usize, total_pages: usize) -> f32 {
        if total_pages == 0 {
            return 0.0;
        }
        (current_page as f32 / total_pages as f32) * 100.0
    }

    /// Calculate session duration since last page turn
    fn calculate_session_duration(&self, now: &NaiveDateTime) -> u64 {
        if let Some(last_timestamp) = self.page_turns.keys().max() {
            let duration = *now - *last_timestamp;
            duration.num_seconds().max(0) as u64
        } else {
            0
        }
    }

    /// Recalculate average reading speeds
    fn recalculate_averages(&mut self) {
        self.average_pages_per_minute = self.pages_per_minute();
        self.average_words_per_minute = self.words_per_minute();
    }

    /// Update reading streak
    fn update_reading_streak(&mut self) {
        let today = Local::now().naive_local().date();

        if let Some(last_date) = self.last_reading_date {
            let last_date_naive = last_date.date();
            let days_diff = (today - last_date_naive).num_days();

            if days_diff == 0 {
                // Same day, streak continues
            } else if days_diff == 1 {
                // Consecutive day, increment streak
                self.reading_streak_days += 1;
            } else {
                // Streak broken, reset
                self.reading_streak_days = 1;
            }
        } else {
            // First reading session
            self.reading_streak_days = 1;
        }
    }

    /// Get reading statistics from ReaderInfo
    pub fn from_reader_info(reader_info: &ReaderInfo) -> Self {
        Self {
            total_reading_time_seconds: reader_info.reading_time_seconds,
            ..Default::default()
        }
    }

    /// Clear old page turn history (keep last N days)
    pub fn prune_old_history(&mut self, days_to_keep: i64) {
        let cutoff = Local::now().naive_local() - chrono::Duration::days(days_to_keep);
        self.page_turns.retain(|timestamp, _| *timestamp > cutoff);
    }
}

#[cfg(test)]
mod reading_stats_tests {
    use super::*;

    #[test]
    fn test_reading_stats_new() {
        let stats = ReadingStatistics::new();
        assert_eq!(stats.total_pages_read, 0);
        assert_eq!(stats.total_reading_time_seconds, 0);
        assert_eq!(stats.average_pages_per_minute, 0.0);
        assert_eq!(stats.average_words_per_minute, 0.0);
        assert_eq!(stats.reading_streak_days, 0);
    }

    #[test]
    fn test_record_page_turn() {
        let mut stats = ReadingStatistics::new();
        stats.record_page_turn(1, 300);

        assert_eq!(stats.total_pages_read, 1);
        assert!(stats.last_reading_date.is_some());
    }

    #[test]
    fn test_pages_per_minute_zero_time() {
        let stats = ReadingStatistics::new();
        assert_eq!(stats.pages_per_minute(), 0.0);
    }

    #[test]
    fn test_words_per_minute_zero_time() {
        let stats = ReadingStatistics::new();
        assert_eq!(stats.words_per_minute(), 0.0);
    }

    #[test]
    fn test_estimate_time_to_finish_zero_speed() {
        let stats = ReadingStatistics::new();
        assert_eq!(stats.estimate_time_to_finish(100), 0);
    }

    #[test]
    fn test_progress_percentage_zero_total() {
        let stats = ReadingStatistics::new();
        assert_eq!(stats.progress_percentage(50, 0), 0.0);
    }

    #[test]
    fn test_progress_percentage() {
        let stats = ReadingStatistics::new();
        assert_eq!(stats.progress_percentage(50, 100), 50.0);
    }

    #[test]
    fn test_prune_old_history() {
        let mut stats = ReadingStatistics::new();

        // Add some page turns
        let old_date = Local::now().naive_local() - chrono::Duration::days(10);
        let event = PageTurnEvent {
            page: 1,
            timestamp: old_date,
            session_duration_seconds: 60,
        };
        stats.page_turns.insert(old_date, event);

        stats.prune_old_history(7);

        assert!(stats.page_turns.is_empty());
    }

    #[test]
    fn test_from_reader_info() {
        let mut reader_info = ReaderInfo::default();
        reader_info.reading_time_seconds = 3600; // 1 hour

        let stats = ReadingStatistics::from_reader_info(&reader_info);
        assert_eq!(stats.total_reading_time_seconds, 3600);
    }

    #[test]
    fn test_page_turn_event_default() {
        let event = PageTurnEvent::default();
        assert_eq!(event.page, 0);
        assert_eq!(event.session_duration_seconds, 0);
    }
}
