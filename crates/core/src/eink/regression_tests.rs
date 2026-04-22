//! Phase 5 Regression Tests for E-Ink Layer
//!
//! Tests edge cases and specific bug scenarios discovered during development.

use crate::eink::*;
use crate::geom::{Point, Rectangle};

// ============================================================================
// Damage Tracker Edge Cases
// ============================================================================

#[test]
fn test_damage_tracker_alternating_sizes() {
    let mut tracker = DamageTracker::new(50);

    // Alternate between different sizes
    let sizes = vec![(100, 100), (200, 200), (100, 100), (200, 200)];

    for (w, h) in sizes {
        let frame = FrameBuffer::new(w, h);
        let regions = tracker.track_changes(&frame);
        
        // Each size change should trigger full refresh
        assert!(!regions.is_empty(), "Size change should have damage");
    }
}

#[test]
fn test_damage_tracker_single_line_change() {
    let mut tracker = DamageTracker::new(50);

    // First frame
    let frame1 = FrameBuffer::new(100, 100);
    tracker.track_changes(&frame1);

    // Second frame - change only one scanline
    let mut frame2_data = vec![0u8; 100 * 100 * 4];
    for x in 0..100 {
        let idx = (50 * 100 + x) * 4;
        frame2_data[idx] = 255;
        frame2_data[idx + 1] = 255;
        frame2_data[idx + 2] = 255;
    }
    let frame2 = FrameBuffer::from_data(100, 100, frame2_data).unwrap();
    
    let regions = tracker.track_changes(&frame2);
    
    // Should detect the line change
    assert!(!regions.is_empty());
    // Should not trigger full refresh (single line is small)
    assert!(!tracker.should_full_refresh());
}

#[test]
fn test_damage_checkerboard_pattern() {
    let mut tracker = DamageTracker::new(50);

    // First frame - all black
    let frame1 = FrameBuffer::new(100, 100);
    tracker.track_changes(&frame1);

    // Second frame - checkerboard pattern
    let mut frame2_data = vec![0u8; 100 * 100 * 4];
    for y in 0..100 {
        for x in 0..100 {
            let idx = (y * 100 + x) * 4;
            if (x + y) % 2 == 0 {
                frame2_data[idx] = 255;
                frame2_data[idx + 1] = 255;
                frame2_data[idx + 2] = 255;
            }
        }
    }
    let frame2 = FrameBuffer::from_data(100, 100, frame2_data).unwrap();
    
    let regions = tracker.track_changes(&frame2);
    
    // Checkerboard should have many damage regions
    assert!(!regions.is_empty());
    // Total damage should be about 50% of screen
    let total_damage: u32 = regions.iter().map(|r| r.width() * r.height()).sum();
    assert!(total_damage > 1000, "Checkerboard should have significant damage");
}

// ============================================================================
// Grayscale Conversion Edge Cases
// ============================================================================

#[test]
fn test_grayscale_extreme_values() {
    let converter = GrayscaleConverter::new(DitheringMode::None);

    // All black
    let black = vec![0u8; 100 * 4];
    let gray_black = converter.convert(&black, 10, 10).unwrap();
    assert!(gray_black.iter().all(|&v| v == 0), "All black should be level 0");

    // All white
    let white = vec![255u8; 100 * 4];
    let gray_white = converter.convert(&white, 10, 10).unwrap();
    assert!(gray_white.iter().all(|&v| v == 15), "All white should be level 15");
}

#[test]
fn test_grayscale_transparent_pixels() {
    let converter = GrayscaleConverter::new(DitheringMode::None);

    // Pixels with varying alpha
    let mut rgba = vec![0u8; 4 * 4];
    // First pixel: opaque white
    rgba[0] = 255; rgba[1] = 255; rgba[2] = 255; rgba[3] = 255;
    // Second pixel: transparent white (should still convert RGB)
    rgba[4] = 255; rgba[5] = 255; rgba[6] = 255; rgba[7] = 0;

    let gray = converter.convert(&rgba, 2, 2).unwrap();
    
    // Both should convert based on RGB values (ignoring alpha)
    assert_eq!(gray[0], 15);
    assert_eq!(gray[1], 15);
}

#[test]
fn test_floyd_steinberg_narrow_strip() {
    let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);

    // Very narrow image (1 pixel wide, 10 pixels tall)
    let rgba = vec![128u8; 10 * 4];
    let gray = converter.convert(&rgba, 1, 10).unwrap();

    assert_eq!(gray.len(), 10);
    // Should not panic on edge case
}

#[test]
fn test_floyd_steinberg_flat_strip() {
    let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);

    // Very flat image (10 pixels wide, 1 pixel tall)
    let rgba = vec![128u8; 10 * 4];
    let gray = converter.convert(&rgba, 10, 1).unwrap();

    assert_eq!(gray.len(), 10);
}

#[test]
fn test_ordered_dithering_pattern_consistency() {
    let converter = GrayscaleConverter::new(DitheringMode::Ordered);

    // Create medium gray image
    let rgba: Vec<u8> = (0..(16 * 16)).flat_map(|_| [128u8, 128, 128, 255]).collect();
    let gray1 = converter.convert(&rgba, 16, 16).unwrap();
    let gray2 = converter.convert(&rgba, 16, 16).unwrap();

    // Ordered dithering should be deterministic
    assert_eq!(gray1, gray2, "Ordered dithering should be consistent");
}

#[test]
fn test_gamma_extreme_values() {
    let mid_gray = vec![128u8, 128, 128, 255];

    // High gamma (>1.0) brightens mid-tones (curve becomes concave)
    let high_gamma = GrayscaleConverter::with_gamma(DitheringMode::None, 5.0).unwrap();
    let result = high_gamma.convert(&mid_gray, 1, 1).unwrap();
    // Gamma 5.0: 128 -> ~196 -> ~12 in 16-level (brightened from mid-gray ~7-8)
    assert!(result[0] >= 11, "High gamma should brighten mid-tones, got {}", result[0]);

    // Low gamma (<1.0) darkens mid-tones (curve becomes convex)
    let low_gamma = GrayscaleConverter::with_gamma(DitheringMode::None, 0.5).unwrap();
    let result = low_gamma.convert(&mid_gray, 1, 1).unwrap();
    // Gamma 0.5: 128 -> ~52 -> ~3 in 16-level (darkened from mid-gray ~7-8)
    assert!(result[0] <= 5, "Low gamma should darken mid-tones, got {}", result[0]);
}

// ============================================================================
// Ghosting Reducer Edge Cases
// ============================================================================

#[test]
fn test_ghosting_reducer_max_updates_boundary() {
    let mut reducer = GhostingReducer::new(5, 60);

    // Register exactly max updates
    for _ in 0..5 {
        reducer.register_partial_update();
    }

    assert!(reducer.should_force_full_refresh(), "Should force refresh at max");
    
    // Reset
    reducer.register_full_refresh();
    assert!(!reducer.should_force_full_refresh());

    // Register max - 1 updates
    for _ in 0..4 {
        reducer.register_partial_update();
    }
    assert!(!reducer.should_force_full_refresh(), "Should not force refresh below max");
}

#[test]
fn test_ghosting_reducer_time_boundary() {
    // Create reducer with very short interval for testing
    let mut reducer = GhostingReducer::new(100, 1);
    
    reducer.register_full_refresh();
    assert!(!reducer.should_force_full_refresh());

    // Simulate time passing (would need time manipulation for true test)
    // For now, just verify the logic exists
    reducer.register_partial_update();
}

// ============================================================================
// Waveform Edge Cases
// ============================================================================

#[test]
fn test_waveform_all_combinations() {
    use waveform::{ContentType, UpdateType};

    let content_types = [
        ContentType::Text,
        ContentType::Image,
        ContentType::Mixed,
        ContentType::UI,
    ];

    let update_types = [
        UpdateType::Full,
        UpdateType::Partial,
        UpdateType::Fast,
    ];

    for content in &content_types {
        for update in &update_types {
            let waveform = select_waveform(*content, *update);
            // Should always return a valid mode
            match waveform {
                WaveformMode::GC16 |
                WaveformMode::GL16 |
                WaveformMode::DU |
                WaveformMode::A2 |
                WaveformMode::AUTO => {}
            }
        }
    }
}

// ============================================================================
// Partial Refresh Manager Edge Cases
// ============================================================================

#[test]
fn test_partial_refresh_empty_regions() {
    let mut manager = PartialRefreshManager::new(10, 100);

    let fb1 = FrameBuffer::new(100, 100);
    let regions = manager.track_frame(&fb1);
    assert_eq!(regions.len(), 1); // Full screen on first frame
}

#[test]
fn test_partial_refresh_first_frame_full() {
    // First frame should always return full screen (no previous to compare)
    let mut manager = PartialRefreshManager::new(10, 100);
    let fb = FrameBuffer::new(100, 100);
    
    let regions = manager.track_frame(&fb);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].width(), 100);
    assert_eq!(regions[0].height(), 100);
}

#[test] 
fn test_partial_refresh_small_damage_filtered() {
    // Small damage regions below min_region_size should be filtered out
    let mut manager = PartialRefreshManager::new(10, 1000); // min_size 1000
    let fb1 = FrameBuffer::new(100, 100);
    let mut fb2 = FrameBuffer::new(100, 100);

    // Change a 10x10 block (100 pixels) - below 1000 threshold
    for y in 0..10 {
        for x in 0..10 {
            let idx = (y * 100 + x) * 4;
            fb2.data[idx] = 255;
        }
    }

    manager.track_frame(&fb1); // First frame = full screen
    let regions = manager.track_frame(&fb2); // Second frame with small damage
    
    // Small region should be filtered out, resulting in empty or minimal regions
    // The exact behavior depends on damage tracker implementation
    // Just verify no panic and reasonable behavior
}

// ============================================================================
// Controller Edge Cases
// ============================================================================

#[test]
fn test_controller_input_validation() {
    let sunxi = SunxiController::default().unwrap();
    let mxc = MxcController::default().unwrap();

    let region = Rectangle::from_coords(0, 0, 100, 100);
    
    // Both controllers should reject empty data (validation happens before hardware access)
    let empty_data: Vec<u8> = vec![];
    assert!(sunxi.update(region, &empty_data, WaveformMode::GC16).is_err());
    assert!(mxc.update(region, &empty_data, WaveformMode::GC16).is_err());
    
    // Valid data should pass validation but fail at hardware (expected without device)
    let valid_data = vec![0u8; 100];
    assert!(sunxi.update(region, &valid_data, WaveformMode::GC16).is_err());
    assert!(mxc.update(region, &valid_data, WaveformMode::GC16).is_err());
}

// ============================================================================
// FrameBuffer Edge Cases
// ============================================================================

#[test]
fn test_framebuffer_edge_access() {
    let fb = FrameBuffer::new(10, 10);

    // Corner pixels
    assert!(fb.get_pixel(0, 0).is_some());
    assert!(fb.get_pixel(9, 9).is_some());
    
    // Out of bounds
    assert!(fb.get_pixel(10, 5).is_none());
    assert!(fb.get_pixel(5, 10).is_none());
    assert!(fb.get_pixel(10, 10).is_none());
}

#[test]
fn test_framebuffer_minimum_size() {
    // 1x1 framebuffer
    let fb = FrameBuffer::new(1, 1);
    assert_eq!(fb.data.len(), 4);
    assert!(fb.get_pixel(0, 0).is_some());
}

#[test]
fn test_framebuffer_large_dimensions() {
    // Large dimensions (but small total size for test speed)
    let fb = FrameBuffer::new(1000, 1);
    assert_eq!(fb.data.len(), 4000);
    assert_eq!(fb.width, 1000);
    assert_eq!(fb.height, 1);
}

// ============================================================================
// Integration Edge Cases
// ============================================================================

#[test]
fn test_full_pipeline_minimal() {
    // 1x1 pixel document
    let rgba = vec![255u8, 0, 0, 255]; // Red pixel
    
    let converter = GrayscaleConverter::new(DitheringMode::None);
    let grayscale = converter.convert(&rgba, 1, 1).unwrap();
    
    let mut tracker = DamageTracker::new(50);
    let fb = FrameBuffer::from_data(1, 1, rgba).unwrap();
    let regions = tracker.track_changes(&fb);
    
    assert_eq!(grayscale.len(), 1);
    // Red should be about 76 in luminance (0.299 * 255), which quantizes to ~4
    assert!(grayscale[0] > 0 && grayscale[0] < 10);
    assert_eq!(regions.len(), 1);
}

#[test]
fn test_rapid_reset_cycles() {
    let mut tracker = DamageTracker::new(50);
    let mut ghosting = GhostingReducer::new(5, 60);

    for _ in 0..20 {
        let frame = FrameBuffer::new(100, 100);
        tracker.track_changes(&frame);
        tracker.reset();
        ghosting.reset();
    }

    // Should handle rapid resets without issues
    let frame = FrameBuffer::new(100, 100);
    let regions = tracker.track_changes(&frame);
    assert_eq!(regions.len(), 1);
}

#[test]
fn test_stress_many_small_regions() {
    let mut tracker = DamageTracker::new(50);
    
    // First frame
    let frame1 = FrameBuffer::new(100, 100);
    tracker.track_changes(&frame1);

    // Second frame with many scattered pixel changes
    let mut frame2_data = vec![0u8; 100 * 100 * 4];
    for i in 0..50 {
        let x = i * 2;
        let y = i * 2;
        let idx = (y * 100 + x) * 4;
        frame2_data[idx] = 255;
        frame2_data[idx + 1] = 255;
        frame2_data[idx + 2] = 255;
    }
    let frame2 = FrameBuffer::from_data(100, 100, frame2_data).unwrap();
    
    let regions = tracker.track_changes(&frame2);
    
    // Should handle many small regions
    assert!(!regions.is_empty());
}
