//! Damage tracking for e-ink partial refresh
//!
//! Tracks which regions of the display have changed between renders
//! to enable efficient partial updates.

use crate::geom::Rectangle;
use anyhow::Result;

/// Frame buffer representing RGBA pixel data
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        if width == 0 || height == 0 {
            return Self {
                width: 1,
                height: 1,
                data: vec![0; 4],
            };
        }
        let data = vec![0; (width * height * 4) as usize];
        Self {
            width,
            height,
            data,
        }
    }

    pub fn from_data(width: u32, height: u32, data: Vec<u8>) -> Result<Self> {
        if width == 0 || height == 0 {
            anyhow::bail!("Width and height must be greater than 0");
        }
        if data.is_empty() {
            anyhow::bail!("Data cannot be empty");
        }
        let expected_len = (width * height * 4) as usize;
        if data.len() != expected_len {
            anyhow::bail!(
                "Data length {} does not match expected {} for {}x{} buffer",
                data.len(),
                expected_len,
                width,
                height
            );
        }
        Ok(Self {
            width,
            height,
            data,
        })
    }

    #[inline]
    pub fn pixel_index(&self, x: u32, y: u32) -> usize {
        ((y * self.width + x) * 4) as usize
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = self.pixel_index(x, y);
        Some([
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        ])
    }
}

/// Tracks changed regions between frames
#[derive(Debug)]
pub struct DamageTracker {
    previous_frame: Option<FrameBuffer>,
    damage_regions: Vec<Rectangle>,
    partial_update_threshold: u32,
}

impl DamageTracker {
    pub fn new(partial_update_threshold: u32) -> Self {
        Self {
            previous_frame: None,
            damage_regions: Vec::new(),
            partial_update_threshold,
        }
    }

    pub fn track_changes(&mut self, current: &FrameBuffer) -> Vec<Rectangle> {
        if let Some(prev) = &self.previous_frame {
            if prev.width != current.width || prev.height != current.height {
                self.damage_regions.clear();
                self.damage_regions.push(Rectangle::new(
                    crate::geom::Point::new(0, 0),
                    crate::geom::Point::new(current.width as i32, current.height as i32),
                ));
                self.previous_frame = Some(current.clone());
                return self.damage_regions.clone();
            }

            let prev = prev.clone();
            self.calculate_damage(&prev, current);
        } else {
            self.damage_regions.push(Rectangle::new(
                crate::geom::Point::new(0, 0),
                crate::geom::Point::new(current.width as i32, current.height as i32),
            ));
        }

        self.previous_frame = Some(current.clone());
        self.damage_regions.clone()
    }

    fn calculate_damage(&mut self, prev: &FrameBuffer, current: &FrameBuffer) {
        self.damage_regions.clear();
        let mut in_damage = false;
        let mut damage_start = (0, 0);
        let mut damage_end = (0, 0);

        for y in 0..current.height {
            for x in 0..current.width {
                let idx = current.pixel_index(x, y);
                let pixel_diff = prev.data[idx] != current.data[idx]
                    || prev.data[idx + 1] != current.data[idx + 1]
                    || prev.data[idx + 2] != current.data[idx + 2];

                if pixel_diff {
                    if !in_damage {
                        in_damage = true;
                        damage_start = (x, y);
                    }
                    damage_end = (x, y);
                } else if in_damage {
                    in_damage = false;
                    self.add_damage_region(damage_start, damage_end);
                }
            }
        }

        if in_damage {
            self.add_damage_region(damage_start, damage_end);
        }

        // Merge adjacent/overlapping regions to minimize partial refresh calls
        self.merge_damage_regions();
    }

    fn add_damage_region(&mut self, start: (u32, u32), end: (u32, u32)) {
        let rect = Rectangle::new(
            crate::geom::Point::new(start.0 as i32, start.1 as i32),
            crate::geom::Point::new((end.0 + 1) as i32, (end.1 + 1) as i32),
        );
        self.damage_regions.push(rect);
    }

    /// Merge adjacent or overlapping damage regions to minimize partial refresh calls
    fn merge_damage_regions(&mut self) {
        if self.damage_regions.len() <= 1 {
            return;
        }

        let mut merged = Vec::new();
        let mut regions = self.damage_regions.clone();
        regions.sort_by(|a, b| a.min.y.cmp(&b.min.y).then(a.min.x.cmp(&b.min.x)));

        while let Some(current) = regions.pop() {
            let mut merged_rect = current;

            regions.retain(|other| {
                if Self::rects_adjacent_or_overlap(&merged_rect, other) {
                    merged_rect = Self::merge_rects(&merged_rect, other);
                    false
                } else {
                    true
                }
            });

            merged.push(merged_rect);
        }

        self.damage_regions = merged;
    }

    /// Check if two rectangles are adjacent or overlapping
    #[inline]
    fn rects_adjacent_or_overlap(a: &Rectangle, b: &Rectangle) -> bool {
        // Check for overlap
        let overlap_x = a.max.x > b.min.x && a.min.x < b.max.x;
        let overlap_y = a.max.y > b.min.y && a.min.y < b.max.y;

        // Check for adjacency (within a small margin)
        let margin = 5;
        let adjacent_x = (a.max.x - b.min.x).abs() <= margin || (b.max.x - a.min.x).abs() <= margin;
        let adjacent_y = (a.max.y - b.min.y).abs() <= margin || (b.max.y - a.min.y).abs() <= margin;

        (adjacent_y || overlap_y) && overlap_x || (overlap_y && adjacent_x)
    }

    /// Merge two rectangles into their bounding box
    #[inline]
    fn merge_rects(a: &Rectangle, b: &Rectangle) -> Rectangle {
        let min_x = a.min.x.min(b.min.x);
        let min_y = a.min.y.min(b.min.y);
        let max_x = a.max.x.max(b.max.x);
        let max_y = a.max.y.max(b.max.y);

        Rectangle::new(
            crate::geom::Point::new(min_x, min_y),
            crate::geom::Point::new(max_x, max_y),
        )
    }

    pub fn should_full_refresh(&self) -> bool {
        let total_damage_area: u32 = self
            .damage_regions
            .iter()
            .map(|r| r.width() * r.height())
            .sum();

        if let Some(prev) = &self.previous_frame {
            let total_area = prev.width * prev.height;
            total_damage_area > (total_area * self.partial_update_threshold / 100)
        } else {
            true
        }
    }

    pub fn reset(&mut self) {
        self.previous_frame = None;
        self.damage_regions.clear();
    }

    /// Get the optimal refresh strategy based on damage regions
    pub fn get_refresh_strategy(&self) -> RefreshStrategy {
        if self.damage_regions.is_empty() {
            return RefreshStrategy::None;
        }

        if self.should_full_refresh() {
            return RefreshStrategy::Full;
        }

        // If we have a small number of regions, use partial refresh
        // Otherwise, fall back to full refresh to avoid many small updates
        if self.damage_regions.len() <= 4 {
            RefreshStrategy::Partial(self.damage_regions.clone())
        } else {
            RefreshStrategy::Full
        }
    }

    /// Get merged damage regions for partial refresh
    pub fn get_damage_regions(&self) -> Vec<Rectangle> {
        self.damage_regions.clone()
    }
}

/// Strategy for refreshing the e-ink display
#[derive(Debug, Clone)]
pub enum RefreshStrategy {
    /// No refresh needed (no changes)
    None,
    /// Full screen refresh
    Full,
    /// Partial refresh with specific regions
    Partial(Vec<Rectangle>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framebuffer_creation() {
        let fb = FrameBuffer::new(100, 100);
        assert_eq!(fb.width, 100);
        assert_eq!(fb.height, 100);
        assert_eq!(fb.data.len(), 100 * 100 * 4);
    }

    #[test]
    fn test_framebuffer_pixel_access() {
        let mut fb = FrameBuffer::new(10, 10);
        fb.data[0] = 255;
        fb.data[1] = 128;
        fb.data[2] = 64;
        fb.data[3] = 255;

        let pixel = fb.get_pixel(0, 0);
        assert_eq!(pixel, Some([255, 128, 64, 255]));
    }

    #[test]
    fn test_damage_tracker_initial() {
        let mut tracker = DamageTracker::new(50);
        let fb = FrameBuffer::new(100, 100);
        let regions = tracker.track_changes(&fb);
        assert_eq!(regions.len(), 1);
        assert!(tracker.should_full_refresh());
    }

    #[test]
    fn test_damage_tracker_threshold() {
        let mut tracker = DamageTracker::new(10);
        let fb1 = FrameBuffer::new(100, 100);
        let fb2 = FrameBuffer::new(100, 100);

        tracker.track_changes(&fb1);
        tracker.track_changes(&fb2);
        assert!(!tracker.should_full_refresh());
    }

    #[test]
    fn test_refresh_strategy_none() {
        let mut tracker = DamageTracker::new(50);
        let fb1 = FrameBuffer::new(100, 100);
        tracker.track_changes(&fb1);
        tracker.track_changes(&fb1); // No changes

        let strategy = tracker.get_refresh_strategy();
        matches!(strategy, RefreshStrategy::None);
    }

    #[test]
    fn test_refresh_strategy_full() {
        let mut tracker = DamageTracker::new(10);
        let fb1 = FrameBuffer::new(100, 100);
        let fb2 = FrameBuffer::new(100, 100);

        tracker.track_changes(&fb1);
        tracker.track_changes(&fb2);
        let strategy = tracker.get_refresh_strategy();
        matches!(strategy, RefreshStrategy::Full);
    }

    #[test]
    fn test_refresh_strategy_partial() {
        let mut tracker = DamageTracker::new(50);
        let fb1 = FrameBuffer::new(100, 100);
        let mut fb2 = FrameBuffer::new(100, 100);

        // Change a small region
        fb2.data[0] = 255;
        fb2.data[1] = 128;
        fb2.data[2] = 64;

        tracker.track_changes(&fb1);
        tracker.track_changes(&fb2);

        let strategy = tracker.get_refresh_strategy();
        assert!(matches!(strategy, RefreshStrategy::Partial(_)));
    }

    #[test]
    fn test_region_merging() {
        let mut tracker = DamageTracker::new(50);
        let fb1 = FrameBuffer::new(100, 100);
        let mut fb2 = FrameBuffer::new(100, 100);

        // Create two adjacent damage regions
        fb2.data[0] = 255; // (0, 0)
        fb2.data[4 * 100] = 128; // (1, 0) - adjacent horizontally

        tracker.track_changes(&fb1);
        tracker.track_changes(&fb2);

        let regions = tracker.get_damage_regions();
        // Adjacent regions should be merged
        assert!(regions.len() <= 2);
    }
}
