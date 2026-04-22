//! Partial refresh management for e-ink displays
//!
//! Manages partial refresh regions and optimization strategies.

use crate::geom::Rectangle;
use crate::eink::damage_tracker::DamageTracker;
use crate::eink::waveform::{WaveformMode, select_waveform, ContentType, UpdateType};

/// Manages partial refresh operations
#[derive(Debug)]
pub struct PartialRefreshManager {
    damage_tracker: DamageTracker,
    max_partial_updates: u32,
    partial_update_count: u32,
    min_region_size: u32,
}

impl PartialRefreshManager {
    pub fn new(max_partial_updates: u32, min_region_size: u32) -> Self {
        Self {
            damage_tracker: DamageTracker::new(50),
            max_partial_updates,
            partial_update_count: 0,
            min_region_size,
        }
    }

    pub fn track_frame(&mut self, current: &crate::eink::damage_tracker::FrameBuffer) -> Vec<Rectangle> {
        let regions = self.damage_tracker.track_changes(current);

        if self.damage_tracker.should_full_refresh() {
            self.partial_update_count = 0;
            return vec![Rectangle::new(
                crate::geom::Point::new(0, 0),
                crate::geom::Point::new(current.width as i32, current.height as i32),
            )];
        }

        self.filter_small_regions(regions)
    }

    fn filter_small_regions(&self, regions: Vec<Rectangle>) -> Vec<Rectangle> {
        regions
            .into_iter()
            .filter(|r| r.width() * r.height() >= self.min_region_size as u32)
            .collect()
    }

    pub fn merge_adjacent_regions(&self, regions: Vec<Rectangle>) -> Vec<Rectangle> {
        if regions.is_empty() {
            return regions;
        }

        let mut merged = vec![regions[0]];
        let mut current = regions[0];

        for region in regions.iter().skip(1) {
            if self.are_adjacent(&current, region) {
                current = self.merge_rectangles(&current, region);
                merged.pop();
                merged.push(current);
            } else {
                merged.push(*region);
                current = *region;
            }
        }

        merged
    }

    fn are_adjacent(&self, r1: &Rectangle, r2: &Rectangle) -> bool {
        let margin = 10;
        let x_overlap = !(r1.max.x < r2.min.x - margin || r2.max.x < r1.min.x - margin);
        let y_overlap = !(r1.max.y < r2.min.y - margin || r2.max.y < r1.min.y - margin);
        x_overlap && y_overlap
    }

    fn merge_rectangles(&self, r1: &Rectangle, r2: &Rectangle) -> Rectangle {
        let min_x = r1.min.x.min(r2.min.x);
        let min_y = r1.min.y.min(r2.min.y);
        let max_x = r1.max.x.max(r2.max.x);
        let max_y = r1.max.y.max(r2.max.y);
        Rectangle::new(crate::geom::Point::new(min_x, min_y), crate::geom::Point::new(max_x, max_y))
    }

    pub fn should_force_full_refresh(&self) -> bool {
        self.partial_update_count >= self.max_partial_updates
    }

    pub fn increment_partial_count(&mut self) {
        self.partial_update_count += 1;
    }

    pub fn reset_partial_count(&mut self) {
        self.partial_update_count = 0;
    }

    pub fn get_waveform_for_regions(&self, regions: &[Rectangle], content: ContentType) -> WaveformMode {
        let update_type = if regions.len() == 1 {
            let region = &regions[0];
            let is_full_screen = region.width() * region.height() > 1000000;
            if is_full_screen {
                UpdateType::Full
            } else {
                UpdateType::Partial
            }
        } else {
            UpdateType::Partial
        };

        select_waveform(content, update_type)
    }

    pub fn reset(&mut self) {
        self.damage_tracker.reset();
        self.partial_update_count = 0;
    }
}

impl Default for PartialRefreshManager {
    fn default() -> Self {
        Self::new(10, 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eink::damage_tracker::FrameBuffer;

    #[test]
    fn test_manager_creation() {
        let manager = PartialRefreshManager::new(10, 100);
        assert_eq!(manager.max_partial_updates, 10);
        assert_eq!(manager.partial_update_count, 0);
    }

    #[test]
    fn test_should_force_full_refresh() {
        let mut manager = PartialRefreshManager::new(5, 100);
        assert!(!manager.should_force_full_refresh());

        for _ in 0..5 {
            manager.increment_partial_count();
        }
        assert!(manager.should_force_full_refresh());
    }

    #[test]
    fn test_filter_small_regions() {
        let manager = PartialRefreshManager::new(10, 1000);
        let regions = vec![
            Rectangle::from_coords(0, 0, 50, 50),
            Rectangle::from_coords(100, 100, 200, 200),
        ];
        let filtered = manager.filter_small_regions(regions);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_merge_adjacent_regions() {
        let manager = PartialRefreshManager::new(10, 100);
        let regions = vec![
            Rectangle::from_coords(0, 0, 100, 100),
            Rectangle::from_coords(95, 95, 200, 200),
        ];
        let merged = manager.merge_adjacent_regions(regions);
        assert_eq!(merged.len(), 1);
    }
}
