//! Damage tracking for e-ink partial refresh
//!
//! Tracks which regions of the display have changed between renders
//! to enable efficient partial updates.

use anyhow::Result;
use crate::geom::Rectangle;

/// Frame buffer representing RGBA pixel data
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let data = vec![0; (width * height * 4) as usize];
        Self { width, height, data }
    }

    pub fn from_data(width: u32, height: u32, data: Vec<u8>) -> Result<Self> {
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
        Ok(Self { width, height, data })
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
    }

    fn add_damage_region(&mut self, start: (u32, u32), end: (u32, u32)) {
        let rect = Rectangle::new(
            crate::geom::Point::new(start.0 as i32, start.1 as i32),
            crate::geom::Point::new((end.0 + 1) as i32, (end.1 + 1) as i32),
        );
        self.damage_regions.push(rect);
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
        let tracker = DamageTracker::new(50);
        let fb = FrameBuffer::new(100, 100);
        let regions = tracker.track_changes(&fb);
        assert_eq!(regions.len(), 1);
        assert!(tracker.should_full_refresh());
    }

    #[test]
    fn test_damage_tracker_threshold() {
        let tracker = DamageTracker::new(10);
        let mut fb1 = FrameBuffer::new(100, 100);
        let fb2 = FrameBuffer::new(100, 100);

        tracker.track_changes(&fb1);
        tracker.track_changes(&fb2);
        assert!(!tracker.should_full_refresh());
    }
}
