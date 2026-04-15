#[cfg(test)]
mod crop_selection_tests {
    use super::*;
    use crate::geom::{pt, Rectangle};
    use crate::geom::helpers::BorderSpec;

    #[test]
    fn test_coordinate_normalization() {
        // Test coordinate normalization for various input combinations
        let start = (100, 50);
        let end = (25, 150);
        
        // Normalize coordinates (should give min.x,min.y to max.x,max.y)
        let x0 = start.0.min(end.0);
        let y0 = start.1.min(end.1);
        let x1 = start.0.max(end.0);
        let y1 = start.1.max(end.1);
        
        assert_eq!(x0, 25);
        assert_eq!(y0, 50);
        assert_eq!(x1, 100);
        assert_eq!(y1, 150);
        
        // Test with reversed coordinates
        let start = (25, 50);
        let end = (100, 150);
        
        let x0 = start.0.min(end.0);
        let y0 = start.1.min(end.1);
        let x1 = start.0.max(end.0);
        let y1 = start.1.max(end.1);
        
        assert_eq!(x0, 25);
        assert_eq!(y0, 50);
        assert_eq!(x1, 100);
        assert_eq!(y1, 150);
    }

    #[test]
    fn test_crop_rectangle_creation() {
        // Test rectangle geometry creation from normalized coordinates
        let x0 = 25;
        let y0 = 50;
        let x1 = 100;
        let y1 = 150;
        
        let crop_rect = Rectangle::new(pt!(x0, y0), pt!(x1, y1));
        
        assert_eq!(crop_rect.min.x, 25);
        assert_eq!(crop_rect.min.y, 50);
        assert_eq!(crop_rect.max.x, 100);
        assert_eq!(crop_rect.max.y, 150);
        assert_eq!(crop_rect.width(), 75);
        assert_eq!(crop_rect.height(), 100);
    }

    #[test]
    fn test_minimum_selection_size() {
        // Test minimum size validation
        let small_rect = Rectangle::new(pt!(0, 0), pt!(5, 5));
        let valid_rect = Rectangle::new(pt!(0, 0), pt!(15, 15));
        
        assert!(small_rect.width() < MIN_CROP_SIZE as i32);
        assert!(small_rect.height() < MIN_CROP_SIZE as i32);
        assert!(valid_rect.width() >= MIN_CROP_SIZE as i32);
        assert!(valid_rect.height() >= MIN_CROP_SIZE as i32);
    }

    #[test]
    fn test_crop_state_transitions() {
        // Test crop state transitions
        let none_state = CropState::None;
        let selecting_state = CropState::Selecting { 
            start: (100, 100), 
            end: (200, 200) 
        };
        
        assert_eq!(none_state, CropState::None);
        assert!(none_state != selecting_state);
        
        if let CropState::Selecting { start, end } = selecting_state {
            assert_eq!(start, (100, 100));
            assert_eq!(end, (200, 200));
        } else {
            panic!("Expected Selecting state");
        }
    }

    #[test]
    fn test_border_spec_configuration() {
        // Test border specification configuration
        let border = BorderSpec {
            thickness: CROP_BORDER_THICKNESS,
            color: CROP_SELECTION_COLOR,
        };
        
        assert_eq!(border.thickness, CROP_BORDER_THICKNESS);
        assert_eq!(border.color, CROP_SELECTION_COLOR);
    }

    #[test]
    fn test_editor_mode_transitions() {
        // Test editor mode transitions
        let select_mode = EditorMode::SelectBook;
        let edit_mode = EditorMode::EditCover;
        let crop_mode = EditorMode::CropMode;
        
        assert_eq!(select_mode, EditorMode::SelectBook);
        assert_eq!(edit_mode, EditorMode::EditCover);
        assert_eq!(crop_mode, EditorMode::CropMode);
        assert!(select_mode != edit_mode);
        assert!(edit_mode != crop_mode);
        assert!(crop_mode != select_mode);
    }

    #[test]
    fn test_rectangle_intersection() {
        // Test rectangle intersection logic
        let rect1 = Rectangle::new(pt!(0, 0), pt!(100, 100));
        let rect2 = Rectangle::new(pt!(50, 50), pt!(150, 150));
        let rect3 = Rectangle::new(pt!(200, 200), pt!(300, 300));
        
        // Overlapping rectangles should intersect
        assert!(rect1.intersection(&rect2).is_some());
        
        // Non-overlapping rectangles should not intersect
        assert!(rect1.intersection(&rect3).is_none());
        assert!(rect2.intersection(&rect3).is_none());
    }

    #[test]
    fn test_configuration_constants() {
        // Test that configuration constants have expected values
        assert_eq!(MIN_CROP_SIZE, 10);
        assert_eq!(CROP_BORDER_THICKNESS, 2);
        assert_eq!(CROP_SHOW_OVERLAY, false);
        assert_eq!(CROP_OVERLAY_ALPHA, 0.25);
    }

    #[test]
    fn test_coordinate_boundary_validation() {
        // Test coordinate boundary checking
        let valid_start = (100, 100);
        let valid_end = (200, 200);
        let invalid_start = (-10, -10);
        let invalid_end = (5000, 5000);
        
        // Valid coordinates should create meaningful rectangles
        let x0 = valid_start.0.min(valid_end.0);
        let y0 = valid_start.1.min(valid_end.1);
        let x1 = valid_start.0.max(valid_end.0);
        let y1 = valid_start.1.max(valid_end.1);
        
        assert!(x0 >= 0);
        assert!(y0 >= 0);
        assert!(x1 > x0);
        assert!(y1 > y0);
        
        // Even invalid coordinates should normalize properly
        let x0 = invalid_start.0.min(invalid_end.0);
        let y0 = invalid_start.1.min(invalid_end.1);
        let x1 = invalid_start.0.max(invalid_end.0);
        let y1 = invalid_start.1.max(invalid_end.1);
        
        assert_eq!(x0, -10);
        assert_eq!(y0, -10);
        assert_eq!(x1, 5000);
        assert_eq!(y1, 5000);
    }
}
