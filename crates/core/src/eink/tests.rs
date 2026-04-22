//! Comprehensive integration tests for e-ink optimization layer
//!
//! Phase 5 Testing: Validates all Phase 1 components work together correctly.

use crate::eink::*;
use crate::geom::{Point, Rectangle};

// ============================================================================
// Integration Tests: Full pipeline scenarios
// ============================================================================

#[test]
fn test_full_refresh_pipeline() {
    // Simulate a complete refresh cycle
    let mut damage_tracker = DamageTracker::new(50);
    let mut ghosting_reducer = GhostingReducer::new(10, 60);
    let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);

    // First frame - should trigger full refresh
    let frame1 = FrameBuffer::new(100, 100);
    let regions = damage_tracker.track_changes(&frame1);
    
    assert_eq!(regions.len(), 1);
    assert!(damage_tracker.should_full_refresh());
    // Ghosting reducer starts at 0, shouldn't force refresh until threshold
    assert!(!ghosting_reducer.should_force_full_refresh());
    
    ghosting_reducer.register_full_refresh();
    
    // Convert to grayscale
    let grayscale = converter.convert(&frame1.data, 100, 100).unwrap();
    assert_eq!(grayscale.len(), 100 * 100);
}

#[test]
fn test_partial_refresh_pipeline() {
    let mut damage_tracker = DamageTracker::new(50);
    let mut ghosting_reducer = GhostingReducer::default();
    let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);

    // First frame
    let frame1 = FrameBuffer::new(100, 100);
    damage_tracker.track_changes(&frame1);
    ghosting_reducer.register_full_refresh();

    // Second frame with small change
    let mut frame2_data = vec![0u8; 100 * 100 * 4];
    // Change a single pixel
    frame2_data[0] = 255;
    frame2_data[1] = 255;
    frame2_data[2] = 255;
    let frame2 = FrameBuffer::from_data(100, 100, frame2_data).unwrap();
    
    let regions = damage_tracker.track_changes(&frame2);
    ghosting_reducer.register_partial_update();

    // Should have partial damage regions
    assert!(!regions.is_empty());
    assert!(!damage_tracker.should_full_refresh());
    assert!(!ghosting_reducer.should_force_full_refresh());

    // Convert changed regions
    let grayscale = converter.convert(&frame2.data, 100, 100).unwrap();
    assert_eq!(grayscale.len(), 100 * 100);
}

#[test]
fn test_ghosting_reduction_trigger() {
    let mut ghosting_reducer = GhostingReducer::new(5, 60);
    let mut damage_tracker = DamageTracker::new(50);

    // Initial full refresh
    let frame = FrameBuffer::new(100, 100);
    damage_tracker.track_changes(&frame);
    ghosting_reducer.register_full_refresh();
    assert!(!ghosting_reducer.should_force_full_refresh());

    // Simulate 5 partial updates
    for _ in 0..5 {
        ghosting_reducer.register_partial_update();
    }

    // Should now force full refresh
    assert!(ghosting_reducer.should_force_full_refresh());
}

// ============================================================================
// Waveform Selection Integration Tests
// ============================================================================

#[test]
fn test_waveform_selection_scenarios() {
    use waveform::{ContentType, UpdateType, WaveformMode};

    // Text with full update -> GC16 (quality)
    assert_eq!(
        select_waveform(ContentType::Text, UpdateType::Full),
        WaveformMode::GC16
    );

    // Text with partial update -> A2 (fast)
    assert_eq!(
        select_waveform(ContentType::Text, UpdateType::Partial),
        WaveformMode::A2
    );

    // Image with full update -> GC16 (quality)
    assert_eq!(
        select_waveform(ContentType::Image, UpdateType::Full),
        WaveformMode::GC16
    );

    // Image with partial -> GL16 (balanced)
    assert_eq!(
        select_waveform(ContentType::Image, UpdateType::Partial),
        WaveformMode::GL16
    );

    // UI fast update -> A2 (very fast)
    assert_eq!(
        select_waveform(ContentType::UI, UpdateType::Fast),
        WaveformMode::A2
    );
}

// ============================================================================
// Damage Tracker Edge Cases
// ============================================================================

#[test]
fn test_damage_tracker_size_change() {
    let mut tracker = DamageTracker::new(50);

    // First frame 100x100
    let frame1 = FrameBuffer::new(100, 100);
    let regions1 = tracker.track_changes(&frame1);
    assert_eq!(regions1.len(), 1);

    // Second frame 200x200 - should trigger full refresh
    let frame2 = FrameBuffer::new(200, 200);
    let regions2 = tracker.track_changes(&frame2);
    
    assert_eq!(regions2.len(), 1);
    let region = &regions2[0];
    assert_eq!(region.width(), 200);
    assert_eq!(region.height(), 200);
}

#[test]
fn test_damage_tracker_no_change() {
    let mut tracker = DamageTracker::new(50);

    // Two identical frames
    let frame1 = FrameBuffer::new(100, 100);
    tracker.track_changes(&frame1);

    let frame2 = FrameBuffer::new(100, 100);
    let regions = tracker.track_changes(&frame2);

    // Should have no damage regions
    assert!(regions.is_empty());
    assert!(!tracker.should_full_refresh());
}

#[test]
fn test_damage_tracker_full_screen_change() {
    let mut tracker = DamageTracker::new(50);

    // First frame
    let frame1 = FrameBuffer::new(100, 100);
    tracker.track_changes(&frame1);

    // Second frame - completely different
    let frame2_data: Vec<u8> = (0..100 * 100 * 4).map(|i| i as u8).collect();
    let frame2 = FrameBuffer::from_data(100, 100, frame2_data).unwrap();
    
    let regions = tracker.track_changes(&frame2);
    
    // Should detect large change area
    let total_area: u32 = regions.iter().map(|r| r.width() * r.height()).sum();
    assert!(total_area > 0);
}

// ============================================================================
// Grayscale Conversion Tests
// ============================================================================

#[test]
fn test_grayscale_black_white() {
    let converter = GrayscaleConverter::new(DitheringMode::None);

    // Black pixel
    let rgba: Vec<u8> = (0..4).map(|i| [0u8, 0, 0, 255][i]).collect();
    let gray = converter.convert(&rgba, 1, 1).unwrap();
    assert_eq!(gray[0], 0); // Should be black (0)

    // White pixel
    let rgba: Vec<u8> = (0..4).map(|i| [255u8, 255, 255, 255][i]).collect();
    let gray = converter.convert(&rgba, 1, 1).unwrap();
    assert_eq!(gray[0], 15); // Should be white (15 in 16-level)
}

#[test]
fn test_grayscale_16_level_quantization() {
    // Use gamma=1.0 for linear mapping
    let converter = GrayscaleConverter::with_gamma(DitheringMode::None, 1.0).unwrap();

    // Create gradient from black to white
    let mut rgba = Vec::new();
    for i in 0..16 {
        let gray_value = (i * 255 / 15) as u8;
        rgba.extend_from_slice(&[gray_value, gray_value, gray_value, 255]);
    }

    let grayscale = converter.convert(&rgba, 16, 1).unwrap();

    // Each value should map to appropriate 16-level value
    for (i, &value) in grayscale.iter().enumerate() {
        assert_eq!(value as usize, i);
    }
}

#[test]
fn test_floyd_steinberg_dithering() {
    let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);

    // Create a smooth gradient
    let width = 100;
    let height = 10;
    let mut rgba = Vec::new();
    for _y in 0..height {
        for x in 0..width {
            let gray = (x * 255 / width) as u8;
            rgba.extend_from_slice(&[gray, gray, gray, 255]);
        }
    }

    let grayscale = converter.convert(&rgba, width, height).unwrap();
    
    // Should maintain approximate average brightness
    let sum: u32 = grayscale.iter().map(|&v| v as u32).sum();
    let avg = sum / grayscale.len() as u32;
    
    // Average should be around mid-range (7-8 in 0-15 scale)
    assert!(avg >= 5 && avg <= 10);
}

#[test]
fn test_ordered_dithering() {
    let converter = GrayscaleConverter::new(DitheringMode::Ordered);

    let rgba: Vec<u8> = (0..(16 * 16)).flat_map(|_| [128u8, 128, 128, 255]).collect();
    let grayscale = converter.convert(&rgba, 16, 16).unwrap();

    // Ordered dithering should produce a pattern
    // Values shouldn't all be the same
    let unique_values: std::collections::HashSet<u8> = grayscale.iter().cloned().collect();
    assert!(unique_values.len() > 1, "Ordered dithering should produce variety");
}

#[test]
fn test_grayscale_gamma_correction() {
    let converter = GrayscaleConverter::with_gamma(DitheringMode::None, 2.2).unwrap();

    // Mid-gray with gamma correction
    let mid_gray = vec![128u8, 128, 128, 255];
    let gray = converter.convert(&mid_gray, 1, 1).unwrap();
    
    // With gamma 2.2, mid-gray (128) is brightened
    // Linear: 128 -> ~7-8 in 16-level
    // Gamma corrected: ~187 -> ~11 in 16-level (brighter)
    assert!(gray[0] > 7, "Gamma 2.2 should brighten mid-gray, got {}", gray[0]);
}

// ============================================================================
// Partial Refresh Manager Integration Tests
// ============================================================================

#[test]
fn test_partial_refresh_manager_integration() {
    let mut manager = PartialRefreshManager::new(10, 100);
    let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);

    // First frame
    let frame1 = FrameBuffer::new(100, 100);
    let regions = manager.track_frame(&frame1);
    
    // Should return full screen for first frame
    assert_eq!(regions.len(), 1);

    // Simulate conversion
    let grayscale = converter.convert(&frame1.data, 100, 100).unwrap();
    assert_eq!(grayscale.len(), 100 * 100);
}

#[test]
fn test_region_filtering() {
    let manager = PartialRefreshManager::new(10, 1000);

    // Create regions of different sizes
    let regions = vec![
        Rectangle::from_coords(0, 0, 10, 10),      // Small: 100 pixels
        Rectangle::from_coords(20, 20, 50, 50),    // Medium: 900 pixels
        Rectangle::from_coords(100, 100, 200, 200), // Large: 10000 pixels
    ];

    let filtered = manager.filter_small_regions(regions);
    
    // Should only keep regions >= 1000 pixels
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].width(), 100);
    assert_eq!(filtered[0].height(), 100);
}

// ============================================================================
// Controller Mock Tests
// ============================================================================

#[test]
fn test_sunxi_controller_mock() {
    let controller = SunxiController::default().unwrap();
    
    assert_eq!(controller.get_controller_name(), "sunxi-disp2");

    let data = vec![0u8; 100];
    let region = Rectangle::from_coords(0, 0, 10, 10);
    
    // Mock implementation should accept valid data
    assert!(controller.update(region, &data, WaveformMode::GC16).is_ok());
}

#[test]
fn test_mxc_controller_mock() {
    let controller = MxcController::default().unwrap();
    
    assert_eq!(controller.get_controller_name(), "mxc-epdc");

    let data = vec![0u8; 100];
    let region = Rectangle::from_coords(0, 0, 10, 10);
    
    assert!(controller.update(region, &data, WaveformMode::GC16).is_ok());
}

#[test]
fn test_controller_empty_data_rejection() {
    let sunxi = SunxiController::default().unwrap();
    let mxc = MxcController::default().unwrap();
    
    let region = Rectangle::from_coords(0, 0, 10, 10);
    
    // Both controllers should reject empty data
    assert!(sunxi.update(region, &[], WaveformMode::GC16).is_err());
    assert!(mxc.update(region, &[], WaveformMode::GC16).is_err());
}

// ============================================================================
// Performance Benchmarks
// ============================================================================

#[test]
fn test_large_buffer_performance() {
    let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);
    let mut tracker = DamageTracker::new(50);

    // Simulate large e-ink display (e.g., Elipsa 1404x1872)
    let width = 1404;
    let height = 1872;
    let data = vec![128u8; width * height * 4];

    let start = std::time::Instant::now();
    
    let grayscale = converter.convert(&data, width as u32, height as u32).unwrap();
    let frame = FrameBuffer::from_data(width as u32, height as u32, data).unwrap();
    tracker.track_changes(&frame);
    
    let duration = start.elapsed();
    
    // Should complete in reasonable time (< 2 seconds for large buffer in debug)
    assert!(duration.as_secs() < 2, "Large buffer conversion too slow: {:?}", duration);
    assert_eq!(grayscale.len(), width * height);
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
fn test_rapid_partial_updates() {
    let mut manager = PartialRefreshManager::new(100, 100);
    let mut ghosting_reducer = GhostingReducer::new(50, 60);

    // Simulate rapid updates (like scrolling)
    for i in 0..50 {
        let mut data = vec![0u8; 100 * 100 * 4];
        // Change a different region each time
        let offset = (i % 10) * 100 * 4;
        if offset < data.len() {
            data[offset] = 255;
        }
        
        let frame = FrameBuffer::from_data(100, 100, data).unwrap();
        let _regions = manager.track_frame(&frame);
        
        if manager.should_force_full_refresh() {
            ghosting_reducer.register_full_refresh();
            manager.reset_partial_count();
        } else {
            ghosting_reducer.register_partial_update();
            manager.increment_partial_count();
        }
    }
}

#[test]
fn test_damage_tracker_reset() {
    let mut tracker = DamageTracker::new(50);

    // First frame
    let frame1 = FrameBuffer::new(100, 100);
    tracker.track_changes(&frame1);

    // Reset
    tracker.reset();

    // Next frame should be treated as first frame again
    let frame2 = FrameBuffer::new(100, 100);
    let regions = tracker.track_changes(&frame2);
    
    assert_eq!(regions.len(), 1);
    assert!(tracker.should_full_refresh());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_framebuffer_invalid_data() {
    // Too little data
    let result = FrameBuffer::from_data(100, 100, vec![0u8; 100]);
    assert!(result.is_err());

    // Too much data
    let result = FrameBuffer::from_data(10, 10, vec![0u8; 10000]);
    assert!(result.is_err());
}

#[test]
fn test_grayscale_invalid_buffer() {
    let converter = GrayscaleConverter::new(DitheringMode::None);
    
    // Wrong buffer size
    let result = converter.convert(&[0u8; 100], 10, 10);
    assert!(result.is_err());
}

#[test]
fn test_waveform_invalid_string() {
    assert!(WaveformMode::from_str("INVALID").is_err());
    assert!(WaveformMode::from_str("").is_err());
}

#[test]
fn test_ghosting_reducer_invalid_settings() {
    let mut reducer = GhostingReducer::default();
    
    assert!(reducer.set_max_partial_updates(0).is_err());
    assert!(reducer.set_full_refresh_interval(0).is_err());
    
    assert!(reducer.set_max_partial_updates(5).is_ok());
    assert!(reducer.set_full_refresh_interval(30).is_ok());
}
