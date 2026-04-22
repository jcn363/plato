//! Phase 5 Integration Tests: PDFPurr + E-Ink Layer
//!
//! Tests the complete rendering pipeline:
//! PDFPurr rendering -> RGBA output -> E-ink optimization -> Display

use crate::document::pdfpurr::{PdfPurrPixmap, PixmapFormat, FzRect, FzPoint};
use crate::eink::{FrameBuffer, GrayscaleConverter, DitheringMode, DamageTracker, WaveformMode, select_waveform, ContentType, UpdateType};
use crate::geom::Rectangle;

/// Simulates the complete rendering pipeline
fn simulate_render_pipeline(width: u32, height: u32, content_type: ContentType) -> Vec<u8> {
    // Step 1: Create simulated RGBA output from PDFPurr
    let mut rgba_data = vec![0u8; (width * height * 4) as usize];
    
    // Fill with simulated page content (white background with some "text")
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            
            // Simulate text lines every 20 pixels
            if y % 20 < 2 {
                // Dark line (simulated text)
                rgba_data[idx] = 50;
                rgba_data[idx + 1] = 50;
                rgba_data[idx + 2] = 50;
            } else {
                // White background
                rgba_data[idx] = 255;
                rgba_data[idx + 1] = 255;
                rgba_data[idx + 2] = 255;
            }
            rgba_data[idx + 3] = 255; // Alpha
        }
    }

    // Step 2: Convert to grayscale using e-ink layer
    let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);
    let grayscale = converter.convert(&rgba_data, width, height).unwrap();

    // Step 3: Select appropriate waveform
    let update_type = UpdateType::Full;
    let _waveform = select_waveform(content_type, update_type);

    grayscale
}

/// Tests the complete text document rendering pipeline
#[test]
fn test_text_document_pipeline() {
    let width = 600;
    let height = 800;

    let grayscale = simulate_render_pipeline(width, height, ContentType::Text);

    // Verify output size
    assert_eq!(grayscale.len(), (width * height) as usize);

    // Verify we have a mix of values (not all same)
    let unique_values: std::collections::HashSet<u8> = grayscale.iter().cloned().collect();
    assert!(unique_values.len() > 1, "Grayscale should have variety from dithering");

    // Text should mostly be dark (low values) or background (high values)
    let avg: u32 = grayscale.iter().map(|&v| v as u32).sum::<u32>() / grayscale.len() as u32;
    // Average should be skewed toward white (high values) due to more background
    assert!(avg > 8, "Text page average should be in upper half: got {}", avg);
}

/// Tests the image document rendering pipeline
#[test]
fn test_image_document_pipeline() {
    let width = 800;
    let height = 600;

    // Create a simulated image (gradient)
    let mut rgba_data = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let gray = ((x * 255 / width) as u8).min(255);
            rgba_data[idx] = gray;
            rgba_data[idx + 1] = gray;
            rgba_data[idx + 2] = gray;
            rgba_data[idx + 3] = 255;
        }
    }

    let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);
    let grayscale = converter.convert(&rgba_data, width, height).unwrap();

    assert_eq!(grayscale.len(), (width * height) as usize);

    // Gradient should have many different values
    let unique_values: std::collections::HashSet<u8> = grayscale.iter().cloned().collect();
    assert!(unique_values.len() >= 8, "Gradient should have multiple gray levels");
}

/// Tests damage tracking with simulated page turns
#[test]
fn test_page_turn_damage_tracking() {
    let mut tracker = DamageTracker::new(50);
    let width = 600;
    let height = 800;

    // First page
    let page1 = FrameBuffer::new(width, height);
    let regions1 = tracker.track_changes(&page1);
    
    // First page should trigger full refresh
    assert_eq!(regions1.len(), 1);
    assert!(tracker.should_full_refresh());

    // Second page (different content)
    let mut page2_data = vec![255u8; (width * height * 4) as usize];
    // Add different "content"
    for i in (0..page2_data.len()).step_by(40) {
        if i + 2 < page2_data.len() {
            page2_data[i] = 0;
            page2_data[i + 1] = 0;
            page2_data[i + 2] = 0;
        }
    }
    let page2 = FrameBuffer::from_data(width, height, page2_data).unwrap();
    
    let regions2 = tracker.track_changes(&page2);
    
    // Page turn should have significant damage
    let total_damage: u32 = regions2.iter().map(|r| r.width() * r.height()).sum();
    assert!(total_damage > 0, "Page turn should have damage regions");
}

/// Tests partial refresh with simulated scrolling
#[test]
fn test_scroll_partial_refresh() {
    let mut tracker = DamageTracker::new(50);
    let width = 600;
    let height = 800;

    // Initial frame - all white
    let frame1_data = vec![255u8; (width * height * 4) as usize];
    let frame1 = FrameBuffer::from_data(width, height, frame1_data).unwrap();
    tracker.track_changes(&frame1);

    // Simulate scroll - add new dark content at bottom 50 pixels
    let mut frame2_data = vec![255u8; (width * height * 4) as usize];
    for y in (height - 50)..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            frame2_data[idx] = 0;     // R
            frame2_data[idx + 1] = 0; // G
            frame2_data[idx + 2] = 0; // B
        }
    }
    
    let frame2 = FrameBuffer::from_data(width, height, frame2_data).unwrap();
    let regions = tracker.track_changes(&frame2);

    // Scroll should have partial damage (at bottom)
    assert!(!regions.is_empty());
    
    // Total damage should be just the bottom 50 rows (600 * 50 = 30000)
    let total_damage: u32 = regions.iter().map(|r| r.width() * r.height()).sum();
    let expected_damage = width * 50; // Only bottom 50 rows changed
    assert!(
        total_damage <= expected_damage + 1000, // Allow some tolerance
        "Scroll damage should be ~{} pixels, got {}",
        expected_damage,
        total_damage
    );
}

/// Tests waveform selection for different content types
#[test]
fn test_waveform_selection_integration() {
    // Text content scenarios
    assert_eq!(
        select_waveform(ContentType::Text, UpdateType::Full),
        WaveformMode::GC16,
        "Text full refresh should use GC16"
    );
    assert_eq!(
        select_waveform(ContentType::Text, UpdateType::Partial),
        WaveformMode::A2,
        "Text partial should use A2 for speed"
    );

    // Image content scenarios
    assert_eq!(
        select_waveform(ContentType::Image, UpdateType::Full),
        WaveformMode::GC16,
        "Image full refresh should use GC16"
    );
    assert_eq!(
        select_waveform(ContentType::Image, UpdateType::Fast),
        WaveformMode::DU,
        "Image fast update should use DU"
    );

    // Mixed content
    assert_eq!(
        select_waveform(ContentType::Mixed, UpdateType::Partial),
        WaveformMode::GL16,
        "Mixed partial should use GL16"
    );
}

/// Tests the conversion of PDFPurr output to FrameBuffer
#[test]
fn test_pdfpurr_to_framebuffer() {
    let width = 100;
    let height = 100;

    // Simulate PDFPurr output (tiny-skia Pixmap format)
    let pixmap_data: Vec<u8> = (0..width * height * 4)
        .map(|i| (i % 256) as u8)
        .collect();

    // Convert to FrameBuffer
    let fb = FrameBuffer::from_data(width, height, pixmap_data.clone()).unwrap();

    assert_eq!(fb.width, width);
    assert_eq!(fb.height, height);
    assert_eq!(fb.data.len(), pixmap_data.len());

    // Verify pixel access
    let pixel = fb.get_pixel(0, 0);
    assert!(pixel.is_some());
}

/// Tests full refresh detection threshold
#[test]
fn test_full_refresh_threshold() {
    let mut tracker = DamageTracker::new(30); // 30% threshold
    let width = 100;
    let height = 100;

    // First frame
    let frame1 = FrameBuffer::new(width, height);
    tracker.track_changes(&frame1);

    // Create frame with 40% changed (above 30% threshold)
    let mut frame2_data = vec![0u8; (width * height * 4) as usize];
    for i in 0..frame2_data.len() / 2 {
        frame2_data[i] = 255;
    }
    let frame2 = FrameBuffer::from_data(width, height, frame2_data).unwrap();
    tracker.track_changes(&frame2);

    // Should trigger full refresh
    assert!(tracker.should_full_refresh());
}

/// Tests different dithering modes on the same content
#[test]
fn test_dithering_modes_comparison() {
    let width = 100;
    let height = 100;

    // Create gradient content
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let gray = ((x + y) * 255 / (width + height)) as u8;
            rgba[idx] = gray;
            rgba[idx + 1] = gray;
            rgba[idx + 2] = gray;
            rgba[idx + 3] = 255;
        }
    }

    // Test each dithering mode
    let modes = vec![
        DitheringMode::None,
        DitheringMode::FloydSteinberg,
        DitheringMode::Ordered,
    ];

    for mode in modes {
        let converter = GrayscaleConverter::new(mode);
        let grayscale = converter.convert(&rgba, width, height).unwrap();
        
        assert_eq!(grayscale.len(), (width * height) as usize);
        
        // Each mode should produce valid 16-level output
        for &value in &grayscale {
            assert!(value <= 15, "Value {} exceeds 16-level range", value);
        }
    }
}

/// Tests performance of the complete pipeline
#[test]
fn test_pipeline_performance() {
    let width = 1404; // Elipsa width
    let height = 1872; // Elipsa height

    let start = std::time::Instant::now();

    // Simulate rendering
    let rgba = vec![128u8; (width * height * 4) as usize];
    
    // Convert
    let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);
    let _grayscale = converter.convert(&rgba, width, height).unwrap();
    
    // Track damage
    let mut tracker = DamageTracker::new(50);
    let fb = FrameBuffer::from_data(width, height, rgba).unwrap();
    let _regions = tracker.track_changes(&fb);

    let duration = start.elapsed();

    // Should complete in reasonable time (< 2s for large page in debug build)
    // Release builds should be much faster (< 200ms)
    assert!(
        duration.as_secs() < 2,
        "Pipeline too slow: {:?} for {}x{}",
        duration,
        width,
        height
    );
}

/// Tests memory efficiency with repeated renders
#[test]
fn test_memory_efficiency() {
    let width = 600;
    let height = 800;

    // Simulate multiple page renders
    for i in 0..10 {
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        // Vary content slightly
        rgba[i * 1000] = 255;

        let converter = GrayscaleConverter::new(DitheringMode::FloydSteinberg);
        let _grayscale = converter.convert(&rgba, width, height).unwrap();

        // rgba is dropped here, memory should be reclaimed
    }

    // If we get here without OOM, memory management is working
}

/// Tests edge case: very small content area
#[test]
fn test_small_content_area() {
    let width = 10;
    let height = 10;

    let rgba = vec![255u8; (width * height * 4) as usize];
    let converter = GrayscaleConverter::new(DitheringMode::None);
    let grayscale = converter.convert(&rgba, width, height).unwrap();

    assert_eq!(grayscale.len(), 100);

    // All white should map to value 15
    for &value in &grayscale {
        assert_eq!(value, 15, "White should map to level 15");
    }
}

/// Tests edge case: single pixel document
#[test]
fn test_single_pixel() {
    let rgba = vec![128u8, 128, 128, 255];
    let converter = GrayscaleConverter::new(DitheringMode::None);
    let grayscale = converter.convert(&rgba, 1, 1).unwrap();

    assert_eq!(grayscale.len(), 1);
    // With gamma 2.2, mid-gray (128) is brightened to ~11 in 16-level
    // Linear would be ~7-8, gamma 2.2 curve pushes it higher
    assert!(grayscale[0] >= 10 && grayscale[0] <= 12, "Single pixel mid-gray with gamma 2.2: got {}", grayscale[0]);
}
