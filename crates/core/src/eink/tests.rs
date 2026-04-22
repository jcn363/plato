//! Comprehensive test suite for E-ink optimization layer
//! 
//! Tests all Phase 1 completed features:
//! - Damage tracking
//! - Grayscale conversion with dithering
//! - Waveform selection
//! - Ghosting reduction
//! - Partial refresh management

use super::*;
use crate::geom::{Point, Rectangle};

#[cfg(test)]
mod damage_tracker_tests {
    use super::*;

    #[test]
    fn test_framebuffer_creation() {
        let fb = FrameBuffer::new(100, 100);
        assert_eq!(fb.width, 100);
        assert_eq!(fb.height, 100);
        assert_eq!(fb.data.len(), 100 * 100 * 4);
    }

    #[test]
    fn test_framebuffer_from_data() {
        let data = vec![255u8; 100 * 100 * 4];
        let fb = FrameBuffer::from_data(100, 100, data).unwrap();
        assert_eq!(fb.width, 100);
        assert_eq!(fb.height, 100);
    }

    #[test]
    fn test_framebuffer_from_data_invalid_length() {
        let data = vec![255u8; 100]; // Wrong length
        let result = FrameBuffer::from_data(100, 100, data);
        assert!(result.is_err());
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
    fn test_framebuffer_pixel_out_of_bounds() {
        let fb = FrameBuffer::new(10, 10);
        assert_eq!(fb.get_pixel(10, 0), None);
        assert_eq!(fb.get_pixel(0, 10), None);
    }

    #[test]
    fn test_damage_tracker_initial_frame() {
        let tracker = DamageTracker::new(50);
        let fb = FrameBuffer::new(100, 100);
        let regions = tracker.track_changes(&fb);
        assert_eq!(regions.len(), 1);
        assert!(tracker.should_full_refresh());
    }

    #[test]
    fn test_damage_tracker_no_changes() {
        let tracker = DamageTracker::new(50);
        let fb = FrameBuffer::new(100, 100);
        tracker.track_changes(&fb);
        
        // Track same frame again
        let regions = tracker.track_changes(&fb);
        assert_eq!(regions.len(), 0);
        assert!(!tracker.should_full_refresh());
    }

    #[test]
    fn test_damage_tracker_with_changes() {
        let tracker = DamageTracker::new(50);
        let mut fb1 = FrameBuffer::new(100, 100);
        let mut fb2 = FrameBuffer::new(100, 100);
        
        // Modify a pixel in fb2
        fb2.data[0] = 255; // R
        fb2.data[1] = 0;   // G
        fb2.data[2] = 0;   // B
        fb2.data[3] = 255; // A
        
        tracker.track_changes(&fb1);
        let regions = tracker.track_changes(&fb2);
        assert_eq!(regions.len(), 1);
        
        let region = &regions[0];
        assert_eq!(region.min.x, 0);
        assert_eq!(region.min.y, 0);
        assert_eq!(region.max.x, 1);
        assert_eq!(region.max.y, 1);
    }

    #[test]
    fn test_damage_tracker_threshold() {
        let tracker = DamageTracker::new(10); // 10% threshold
        let mut fb1 = FrameBuffer::new(100, 100);
        let mut fb2 = FrameBuffer::new(100, 100);
        
        // Change 5% of pixels
        for i in 0..(100 * 100 * 4 / 20) {
            fb2.data[i] = 255;
        }
        
        tracker.track_changes(&fb1);
        tracker.track_changes(&fb2);
        assert!(!tracker.should_full_refresh()); // 5% < 10% threshold
    }

    #[test]
    fn test_damage_tracker_exceeds_threshold() {
        let tracker = DamageTracker::new(10); // 10% threshold
        let mut fb1 = FrameBuffer::new(100, 100);
        let mut fb2 = FrameBuffer::new(100, 100);
        
        // Change 15% of pixels
        for i in 0..(100 * 100 * 4 / 7) {
            fb2.data[i] = 255;
        }
        
        tracker.track_changes(&fb1);
        tracker.track_changes(&fb2);
        assert!(tracker.should_full_refresh()); // 15% > 10% threshold
    }

    #[test]
    fn test_damage_tracker_resolution_change() {
        let tracker = DamageTracker::new(50);
        let fb1 = FrameBuffer::new(100, 100);
        let fb2 = FrameBuffer::new(200, 150);
        
        tracker.track_changes(&fb1);
        let regions = tracker.track_changes(&fb2);
        assert_eq!(regions.len(), 1);
        assert!(tracker.should_full_refresh());
    }

    #[test]
    fn test_damage_tracker_reset() {
        let tracker = DamageTracker::new(50);
        let fb = FrameBuffer::new(100, 100);
        tracker.track_changes(&fb);
        
        tracker.reset();
        assert!(tracker.previous_frame.is_none());
        assert!(tracker.damage_regions.is_empty());
    }
}

#[cfg(test)]
mod grayscale_tests {
    use super::*;

    #[test]
    fn test_grayscale_converter_creation() {
        let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);
        assert!(matches!(converter.dithering_mode(), DitheringMode::FloydSteinberg));
    }

    #[test]
    fn test_rgba_to_grayscale_basic() {
        let converter = GrayscaleConverter::new(DitheringMode::None);
        let rgba_data = vec![255, 0, 0, 255, 0, 255, 0, 255]; // Red, Green
        let result = converter.convert_to_grayscale(&rgba_data, 2, 1);
        
        assert_eq!(result.len(), 2); // 2 grayscale values
        // Red (255,0,0) -> ~76, Green (0,255,0) -> ~150
        assert!((result[0] as i32 - 76).abs() < 2);
        assert!((result[1] as i32 - 150).abs() < 2);
    }

    #[test]
    fn test_grayscale_quantization() {
        let converter = GrayscaleConverter::new(DitheringMode::None);
        let gray_values = vec![0, 16, 32, 48, 64, 80, 96, 112, 128, 144, 160, 176, 192, 208, 224, 240, 255];
        let quantized = converter.quantize_to_16_level(&gray_values);
        
        // Should map to 0-15 range
        for &val in &quantized {
            assert!(val <= 15);
        }
    }

    #[test]
    fn test_floyd_steinberg_dithering() {
        let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);
        let rgba_data = vec![128, 128, 128, 255]; // Medium gray
        let result = converter.convert_to_grayscale(&rgba_data, 1, 1);
        
        assert_eq!(result.len(), 1);
        assert!(result[0] <= 15); // Should be quantized to 16-level
    }

    #[test]
    fn test_gamma_correction() {
        let converter = GrayscaleConverter::new(DitheringMode::None);
        let linear_values = vec![0, 64, 128, 192, 255];
        let corrected = converter.apply_gamma_correction(&linear_values, 2.2);
        
        // Gamma correction should make values brighter
        for i in 0..linear_values.len() {
            if linear_values[i] > 0 && linear_values[i] < 255 {
                assert!(corrected[i] >= linear_values[i]);
            }
        }
    }
}

#[cfg(test)]
mod waveform_tests {
    use super::*;

    #[test]
    fn test_waveform_selection_text() {
        let mode = select_waveform(ContentType::Text, UpdateType::Partial);
        assert!(matches!(mode, WaveformMode::A2)); // Fast monochrome for text
    }

    #[test]
    fn test_waveform_selection_image() {
        let mode = select_waveform(ContentType::Image, UpdateType::Full);
        assert!(matches!(mode, WaveformMode::GC16)); // High quality for images
    }

    #[test]
    fn test_waveform_selection_mixed() {
        let mode = select_waveform(ContentType::Mixed, UpdateType::Partial);
        assert!(matches!(mode, WaveformMode::GL16)); // Grayscale for mixed content
    }

    #[test]
    fn test_waveform_selection_fast_update() {
        let mode = select_waveform(ContentType::Text, UpdateType::Fast);
        assert!(matches!(mode, WaveformMode::DU)); // Direct update for speed
    }
}

#[cfg(test)]
mod ghosting_tests {
    use super::*;

    #[test]
    fn test_ghosting_reducer_creation() {
        let reducer = GhostingReducer::new(5, 3); // 5 partial updates, 3 full refreshes
        assert_eq!(reducer.partial_update_count(), 0);
        assert_eq!(reducer.full_refresh_count(), 0);
    }

    #[test]
    fn test_ghosting_reducer_partial_updates() {
        let mut reducer = GhostingReducer::new(5, 3);
        
        for i in 0..4 {
            reducer.record_partial_update();
            assert_eq!(reducer.partial_update_count(), i + 1);
            assert!(!reducer.should_force_full_refresh());
        }
        
        // 5th partial update should trigger full refresh
        reducer.record_partial_update();
        assert!(reducer.should_force_full_refresh());
    }

    #[test]
    fn test_ghosting_reducer_full_refresh() {
        let mut reducer = GhostingReducer::new(5, 3);
        
        // Trigger full refresh condition
        for _ in 0..5 {
            reducer.record_partial_update();
        }
        assert!(reducer.should_force_full_refresh());
        
        reducer.record_full_refresh();
        assert_eq!(reducer.partial_update_count(), 0);
        assert_eq!(reducer.full_refresh_count(), 1);
        assert!(!reducer.should_force_full_refresh());
    }

    #[test]
    fn test_ghosting_reducer_periodic_refresh() {
        let mut reducer = GhostingReducer::new(10, 3);
        
        // Record partial updates but not enough to trigger
        for _ in 0..5 {
            reducer.record_partial_update();
        }
        
        // Simulate periodic full refresh
        reducer.record_full_refresh();
        assert_eq!(reducer.partial_update_count(), 0);
    }
}

#[cfg(test)]
mod partial_refresh_tests {
    use super::*;

    #[test]
    fn test_partial_refresh_manager_creation() {
        let manager = PartialRefreshManager::new(20, 100); // 20% threshold, 100px minimum
        assert_eq!(manager.partial_update_threshold(), 20);
        assert_eq!(manager.minimum_region_size(), 100);
    }

    #[test]
    fn test_region_merging() {
        let manager = PartialRefreshManager::new(20, 100);
        let region1 = Rectangle::new(Point::new(0, 0), Point::new(50, 50));
        let region2 = Rectangle::new(Point::new(40, 40), Point::new(90, 90));
        
        let merged = manager.merge_adjacent_regions(&[region1, region2]);
        assert_eq!(merged.len(), 1); // Should merge into one region
    }

    #[test]
    fn test_small_region_filtering() {
        let manager = PartialRefreshManager::new(20, 1000);
        let small_region = Rectangle::new(Point::new(0, 0), Point::new(10, 10)); // 100px
        
        let filtered = manager.filter_small_regions(&[small_region]);
        assert_eq!(filtered.len(), 0); // Should filter out small region
    }

    #[test]
    fn test_optimize_regions() {
        let manager = PartialRefreshManager::new(20, 100);
        let regions = vec![
            Rectangle::new(Point::new(0, 0), Point::new(50, 50)),
            Rectangle::new(Point::new(100, 100), Point::new(150, 150)),
        ];
        
        let optimized = manager.optimize_regions(&regions);
        assert_eq!(optimized.len(), 2); // Should keep both regions
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_end_to_end_pipeline() {
        // Create test RGBA data
        let width = 100;
        let height = 100;
        let mut rgba_data = vec![128u8; width * height * 4];
        
        // Add some variation
        for i in 0..width * height {
            let idx = i * 4;
            rgba_data[idx] = (i % 256) as u8;
        }
        
        // Create framebuffer
        let fb = FrameBuffer::from_data(width as u32, height as u32, rgba_data).unwrap();
        
        // Track damage
        let mut tracker = DamageTracker::new(50);
        let regions = tracker.track_changes(&fb);
        assert!(!regions.is_empty());
        
        // Convert to grayscale
        let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);
        let grayscale = converter.convert_to_grayscale(&fb.data, width, height);
        assert_eq!(grayscale.len(), width * height);
        
        // Select waveform
        let waveform = select_waveform(ContentType::Mixed, UpdateType::Partial);
        assert!(matches!(waveform, WaveformMode::GL16));
        
        // Check if full refresh needed
        assert!(tracker.should_full_refresh()); // First frame always full refresh
    }

    #[test]
    fn test_multiple_frame_updates() {
        let mut tracker = DamageTracker::new(30);
        let converter = GrayscaleConverter::new(DitheringMode::None);
        let mut ghosting_reducer = GhostingReducer::new(5, 2);
        
        // Create initial frame
        let fb1 = FrameBuffer::new(100, 100);
        tracker.track_changes(&fb1);
        ghosting_reducer.record_full_refresh();
        
        // Create second frame with small changes
        let mut fb2 = FrameBuffer::new(100, 100);
        for i in 0..100 {
            fb2.data[i * 4] = 255; // Change some pixels
        }
        
        let regions = tracker.track_changes(&fb2);
        assert!(regions.len() > 0);
        assert!(!tracker.should_full_refresh()); // Small changes
        
        ghosting_reducer.record_partial_update();
        assert!(!ghosting_reducer.should_force_full_refresh());
        
        // Continue with more partial updates
        for _ in 0..4 {
            ghosting_reducer.record_partial_update();
        }
        assert!(ghosting_reducer.should_force_full_refresh());
    }

    #[test]
    fn test_performance_characteristics() {
        use std::time::Instant;
        
        let width = 800;
        let height = 600;
        let fb = FrameBuffer::new(width, height);
        let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);
        
        // Measure grayscale conversion performance
        let start = Instant::now();
        let _grayscale = converter.convert_to_grayscale(&fb.data, width, height);
        let duration = start.elapsed();
        
        // Should complete within reasonable time (adjust threshold as needed)
        assert!(duration.as_millis() < 100, "Grayscale conversion too slow: {:?}", duration);
    }
}
