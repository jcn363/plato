# Crop Selection Implementation Plan for Plato Cover Editor

## Overview

This plan outlines the steps to enhance the interactive crop selection functionality in the Plato cover editor. Analysis of the existing code reveals that the basic crop selection logic is implemented, but there are opportunities to improve the user experience by adding visual feedback during the crop selection process.

## Current State Analysis

After examining the cover editor implementation in `/home/user/Desktop/plato/crates/core/src/view/cover_editor.rs`, I found:

### Existing Functionality:
1. **Crop Mode Infrastructure**: 
   - `EditorMode::CropMode` enum variant (line 22)
   - `CropState::Selecting` state tracking (lines 26-29)
   - `enter_crop_mode()` function (lines 98-102)
   - `apply_crop_rect()` function (lines 104-125) that performs the actual cropping

2. **Input Handling**:
   - Finger down/motion/up events handled in `handle_event()` (lines 233-274)
   - Proper state transitions for crop selection (start → motion → up → apply)

3. **Missing Visual Feedback**:
   - No visual indication of the crop rectangle during selection in the `render()` method
   - Users cannot see what area they are selecting before releasing their finger

## Implementation Plan

The primary enhancement needed is to add visual feedback during crop selection by drawing the crop rectangle in the render method when in `CropMode` with an active selection.

### Step-by-Step Implementation:

#### 1. Add Visual Feedback for Crop Selection
**Location**: `/home/user/Desktop/plato/crates/core/src/view/cover_editor.rs`
**Method**: `render()` function (lines 285-338)
**Action**: Add code to draw the crop rectangle when:
   - Mode is `EditorMode::CropMode`
   - `crop_state` is `CropState::Selecting`
   - There is a valid start and end position

#### 2. Implement Rectangle Drawing Logic
**Action**: In the render method, after drawing children but before drawing the image:
   - Extract start and end points from `crop_state`
   - Normalize coordinates to ensure proper rectangle formation
   - Create a `Rectangle` from the normalized points
   - Create a `BorderSpec` with appropriate thickness and color
   - Draw the rectangle outline using the framebuffer's `draw_rectangle_outline` method
   - Optionally add a semi-transparent fill for better visibility

#### 3. Ensure Proper Coordinate Handling
**Consideration**: The render method uses the view's coordinate system, so crop state coordinates (which are in screen space) should map directly without transformation.

#### 4. Add Visual Styling Constants
**Action**: Define appropriate colors for the crop rectangle (likely using existing theme colors or defining new constants in the file).

#### 5. Test the Implementation
**Verification Steps**:
   - Build the project for host target: `cargo build --target x86_64-unknown-linux-gnu`
   - Run the emulator: `./run-emulator.sh`
   - Navigate to cover editor
   - Select a book
   - Enter crop mode
   - Verify that a rectangle appears as you drag your finger
   - Verify that the crop applies correctly on finger up

## Technical Details

### Code Location for Changes:
File: `/home/user/Desktop/plato/crates/core/src/view/cover_editor.rs`
Function: `render()` (around line 315-316, after the children rendering and before the image rendering)

### Implementation Approach:
```rust
// Add after children rendering and before image rendering
if let EditorMode::CropMode = self.mode {
    if let CropState::Selecting { start, end } = &self.crop_state {
        // Normalize coordinates
        let x0 = start.0.min(end.0);
        let y0 = start.1.min(end.1);
        let x1 = start.0.max(end.0);
        let y1 = start.1.max(end.1);
        
        // Only draw if we have a meaningful selection
        if (x1 - x0) > 0 && (y1 - y0) > 0 {
            let crop_rect = Rectangle::new(pt!(x0, y0), pt!(x1, y1));
            
            // Create border spec for the outline
            let border = BorderSpec {
                thickness: 2,
                color: WHITE,
            };
            
            // Draw rectangle outline
            fb.draw_rectangle_outline(&crop_rect, &border);
            
            // Optionally add semi-transparent fill
            // let mut fill_color = WHITE;
            // fill_color.set_a(64); // 25% opacity white
            // fb.draw_blended_rectangle(&crop_rect, fill_color, 0.25);
        }
    }
}
```

### Required Imports:
Check that `pt!` macro, `WHITE` color, and `BorderSpec` are available (they are based on existing code analysis).

### Styling Considerations:
- Use 2px thickness for clear visibility
- Use solid white outline for clear visibility against both dark and light backgrounds
- Optionally add semi-transparent fill to show selected area while still seeing underlying image
- Ensure the drawing doesn't interfere with normal operation when not in crop mode

## Dependencies and Related Files

### Files to Examine:
1. `/home/user/Desktop/plato/crates/core/src/framebuffer/mod.rs` - For `draw_rectangle_outline` method signature and BorderSpec definition
2. `/home/user/Desktop/plato/crates/core/src/geom/helpers.rs` - For BorderSpec struct definition
3. `/home/user/Desktop/plato/crates/core/src/color.rs` - For color definitions and manipulation

### Methods to Use:
- `fb.draw_rectangle_outline(&rect, &border)` - Already available in Framebuffer trait
- `fb.draw_blended_rectangle` for semi-transparent fill (if desired)

## Quality Assurance

### Testing Strategy:
1. **Unit Testing**: Not applicable as this is UI interaction code
2. **Manual Testing**: 
   - Verify no regression in existing functionality (rotate, brightness, etc.)
   - Test crop selection with various image sizes
   - Test edge cases (very small selections, full image selections)
   - Verify performance impact is minimal

### Code Quality:
- Follow existing code style in the file
- Use proper error handling patterns (though drawing operations shouldn't fail)
- Maintain consistency with existing comment styles
- Keep changes minimal and focused

## References

- AGENTS.md: Plato's coding guidelines and conventions
- Existing cover editor implementation for patterns and style
- Framebuffer trait definition for available drawing methods
- Color handling patterns in the codebase
- Geom module for BorderSpec definition

## Conclusion

This plan focuses on enhancing the user experience of the crop selection feature by adding visual feedback. The core logic is already implemented; the missing piece is the visual indication of what is being selected. This follows the AGENTS.md principles of making minimal, focused changes that improve usability without altering core functionality.