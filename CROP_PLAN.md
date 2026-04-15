# Crop Selection Implementation Plan for Plato Cover Editor

## Overview

This document outlines the comprehensive implementation plan for enhancing the interactive crop selection functionality in the Plato cover editor. The plan follows Plato's architectural principles and coding standards as defined in AGENTS.md.

## Project Context

Plato is a document reader for Kobo e-readers, written in Rust. The crop selection feature is part of the cover editor component located in `crates/core/src/view/cover_editor.rs`. This enhancement focuses on improving user experience through visual feedback during crop selection operations.

## Current State Analysis

### Existing Functionality

After examining the cover editor implementation in `crates/core/src/view/cover_editor.rs`, the following infrastructure is already in place:

#### Core Infrastructure
- **EditorMode::CropMode**: Enum variant for crop selection mode (line 22)
- **CropState::Selecting**: State tracking for active selection (lines 26-29)
- **enter_crop_mode()**: Function to enter crop mode (lines 98-102)
- **apply_crop_rect()**: Function that performs actual cropping operation (lines 104-125)

#### Input Handling System
- **Touch Event Processing**: Finger down/motion/up events in `handle_event()` (lines 233-274)
- **State Machine**: Proper state transitions (start -> motion -> up -> apply)
- **Coordinate Tracking**: Screen space coordinate management

#### Identified Gaps
- **Visual Feedback**: No visual indication of crop rectangle during selection
- **User Experience**: Users cannot see selected area before releasing touch
- **Real-time Preview**: Missing live preview of crop area

## Implementation Plan

The primary enhancement is to add real-time visual feedback during crop selection by rendering the crop rectangle in the `render()` method when in `CropMode` with an active selection.

### Architecture Design

#### Component Structure
```
CoverEditor
  |- EditorMode (enum)
  |- CropState (enum)
  |- render() -> visual feedback
  |- handle_event() -> input processing
  |- apply_crop_rect() -> crop execution
```

#### Data Flow
1. **Input**: Touch events -> coordinate tracking
2. **Processing**: State machine updates -> coordinate normalization
3. **Rendering**: Real-time rectangle drawing -> visual feedback
4. **Execution**: Crop application -> image modification

### Implementation Steps

#### Step 1: Visual Feedback Integration
**Location**: `crates/core/src/view/cover_editor.rs`
**Target Method**: `render()` (lines 285-338)
**Requirements**:
- Render only when `mode == EditorMode::CropMode`
- Render only when `crop_state == CropState::Selecting`
- Validate start/end positions before rendering

#### Step 2: Rectangle Drawing Implementation
**Implementation Details**:
- Extract coordinates from `crop_state`
- Normalize coordinates for proper rectangle geometry
- Create `Rectangle` using `pt!` macro
- Configure `BorderSpec` with theme-appropriate styling
- Utilize framebuffer `draw_rectangle_outline` method
- Optional: Add semi-transparent overlay for enhanced visibility

#### Step 3: Coordinate System Management
**Considerations**:
- View coordinate system alignment
- Screen space to view space mapping
- Boundary validation and clamping
- Device-specific scaling factors

#### Step 4: Styling and Theming
**Configuration**:
- Define crop selection colors in theme constants
- Ensure visibility across different background types
- Maintain consistency with existing UI elements
- Support for dark/light mode variations

#### Step 5: Testing and Validation
**Verification Protocol**:
1. Build for host target: `cargo build --target x86_64-unknown-linux-gnu`
2. Run emulator: `./run-emulator.sh`
3. Navigate to cover editor interface
4. Test crop selection workflow
5. Validate visual feedback accuracy
6. Confirm crop operation correctness

## Technical Implementation

### Code Structure

**Primary File**: `crates/core/src/view/cover_editor.rs`
**Target Function**: `render()` (lines 285-338)
**Insertion Point**: After children rendering, before image rendering

### Core Implementation

```rust
// Visual feedback for crop selection
if let EditorMode::CropMode = self.mode {
    if let CropState::Selecting { start, end } = &self.crop_state {
        // Normalize coordinates to ensure proper rectangle geometry
        let x0 = start.0.min(end.0);
        let y0 = start.1.min(end.1);
        let x1 = start.0.max(end.0);
        let y1 = start.1.max(end.1);
        
        // Validate selection has meaningful dimensions
        if (x1 - x0) > MIN_CROP_SIZE && (y1 - y0) > MIN_CROP_SIZE {
            let crop_rect = Rectangle::new(pt!(x0, y0), pt!(x1, y1));
            
            // Configure visual styling
            let border = BorderSpec {
                thickness: CROP_BORDER_THICKNESS,
                color: CROP_SELECTION_COLOR,
            };
            
            // Draw rectangle outline
            fb.draw_rectangle_outline(&crop_rect, &border)
                .with_context(|| "Failed to draw crop selection outline")?;
            
            // Optional: Add semi-transparent overlay
            if CROP_SHOW_OVERLAY {
                let overlay_color = CROP_OVERLAY_COLOR;
                fb.draw_blended_rectangle(&crop_rect, overlay_color, CROP_OVERLAY_ALPHA)
                    .with_context(|| "Failed to draw crop selection overlay")?;
            }
        }
    }
}
```

### Configuration Constants

```rust
// Crop selection visual configuration
const MIN_CROP_SIZE: u32 = 10;
const CROP_BORDER_THICKNESS: u8 = 2;
const CROP_SELECTION_COLOR: Color = WHITE;
const CROP_SHOW_OVERLAY: bool = true;
const CROP_OVERLAY_COLOR: Color = WHITE;
const CROP_OVERLAY_ALPHA: f32 = 0.25;
```

### Dependencies

**Required Imports**:
```rust
use anyhow::Context;
use crate::geom::{Rectangle, pt};
use crate::color::Color;
use crate::framebuffer::{Framebuffer, BorderSpec};
```

### Error Handling Strategy

- Use `with_context()` for drawing operation errors
- Graceful degradation if drawing fails
- Log warnings without interrupting user interaction
- Validate all inputs before rendering operations

## Dependencies and Module Integration

### Core Dependencies

#### Framebuffer Module
**File**: `crates/core/src/framebuffer/mod.rs`
**Purpose**: Drawing primitives and rendering operations
**Key Methods**:
- `draw_rectangle_outline(&rect, &border)` - Rectangle outline rendering
- `draw_blended_rectangle(&rect, color, alpha)` - Semi-transparent overlay

#### Geometry Module
**File**: `crates/core/src/geom/helpers.rs`
**Purpose**: Geometric primitives and utilities
**Key Types**:
- `Rectangle` - 2D rectangle representation
- `BorderSpec` - Border styling configuration
- `pt!` macro - Point construction helper

#### Color Module
**File**: `crates/core/src/color.rs`
**Purpose**: Color definitions and manipulation
**Key Constants**:
- `WHITE` - Default selection color
- Color manipulation utilities

### Module Integration Points

#### Input Validation
- Coordinate boundary checking
- Selection size validation
- Device-specific scaling

#### Rendering Pipeline
1. Background rendering
2. Children rendering
3. **Crop selection overlay (new)**
4. Image rendering
5. UI overlay rendering

#### State Management
- Mode transitions
- Coordinate tracking
- Selection persistence

## Quality Assurance

### Testing Strategy

#### Unit Testing
**Location**: `crates/core/src/view/cover_editor_tests.rs`
**Test Cases**:
- Coordinate normalization logic
- Rectangle creation from points
- Border specification validation
- State transition verification

```rust
#[cfg(test)]
mod crop_selection_tests {
    use super::*;
    
    #[test]
    fn test_coordinate_normalization() {
        // Test coordinate normalization for various input combinations
    }
    
    #[test]
    fn test_crop_rectangle_creation() {
        // Test rectangle geometry creation
    }
    
    #[test]
    fn test_minimum_selection_size() {
        // Test minimum size validation
    }
}
```

#### Integration Testing
**Location**: `tests/cover_editor_integration_tests.rs`
**Test Scenarios**:
- End-to-end crop selection workflow
- Multi-touch interaction handling
- Performance under various image sizes
- Memory usage validation

#### Manual Testing Protocol
1. **Regression Testing**:
   - Verify existing functionality (rotate, brightness, contrast)
   - Test with various image formats and sizes
   - Validate touch interaction responsiveness

2. **Edge Case Testing**:
   - Very small selections (< MIN_CROP_SIZE)
   - Full image selections
   - Selections at image boundaries
   - Rapid touch movements

3. **Performance Testing**:
   - Rendering performance impact measurement
   - Memory usage during selection
   - Battery consumption assessment

### Code Quality Standards

#### Style Compliance
- Follow Rust naming conventions (snake_case, PascalCase)
- Maintain consistent indentation and formatting
- Use `cargo fmt` and `cargo clippy` for validation
- Document all public APIs with rustdoc comments

#### Error Handling
- Use `anyhow::Error` for error propagation
- Provide meaningful error context with `with_context()`
- Implement graceful degradation for drawing failures
- Log warnings without interrupting user experience

#### Performance Optimization
- Minimize allocation in hot paths
- Use `#[inline]` for small, frequently called functions
- Pre-allocate buffers where size is predictable
- Validate performance impact on target devices

## Configuration Management

### Centralized Configuration

**File**: `crates/core/src/config/crop_selection.rs`
```rust
pub struct CropSelectionConfig {
    pub min_selection_size: u32,
    pub border_thickness: u8,
    pub selection_color: Color,
    pub show_overlay: bool,
    pub overlay_color: Color,
    pub overlay_alpha: f32,
}

impl Default for CropSelectionConfig {
    fn default() -> Self {
        Self {
            min_selection_size: 10,
            border_thickness: 2,
            selection_color: WHITE,
            show_overlay: true,
            overlay_color: WHITE,
            overlay_alpha: 0.25,
        }
    }
}
```

### Validation Rules

#### Input Validation
- Minimum selection size enforcement
- Coordinate boundary checking
- Device-specific scaling validation

#### Configuration Validation
- Color value validation
- Alpha range validation (0.0 - 1.0)
- Thickness range validation

## Documentation and Architecture

### Module Documentation

#### Cover Editor Module
**Purpose**: Interactive image editing functionality
**Responsibilities**:
- Touch event processing
- Visual feedback rendering
- Image manipulation operations
- State management

#### Crop Selection Subsystem
**Purpose**: Visual crop area selection
**Components**:
- Coordinate tracking
- Rectangle rendering
- Visual styling
- State validation

### API Documentation

```rust
/// Renders visual feedback for crop selection.
/// 
/// This method draws a rectangle outline showing the currently selected
/// crop area when in crop mode. The rectangle is rendered with a
/// configurable border and optional semi-transparent overlay.
/// 
/// # Arguments
/// 
/// * `fb` - The framebuffer for rendering operations
/// 
/// # Errors
/// 
/// Returns an error if drawing operations fail. The error is
/// handled gracefully and does not interrupt user interaction.
/// 
/// # Examples
/// 
/// ```rust
/// let editor = CoverEditor::new();
/// editor.enter_crop_mode();
/// // ... user interaction ...
/// editor.render(&mut framebuffer);
/// ```
fn render_crop_selection(&self, fb: &mut dyn Framebuffer) -> Result<(), Error> {
    // Implementation
}
```

## References

### Core Documentation
- **AGENTS.md**: Plato's coding guidelines and architectural principles
- **CONTRIBUTING.md**: Development workflow and contribution guidelines
- **DEVELOPMENT_SETUP.md**: Environment setup and build instructions

### Technical References
- **Framebuffer API**: Available drawing methods and capabilities
- **Geometry Module**: Rectangle and coordinate handling utilities
- **Color System**: Color definitions and manipulation functions
- **Touch Input System**: Event handling and coordinate mapping

### Related Components
- **Image Processing**: Document rendering and manipulation
- **UI Framework**: View system and event handling
- **Device Abstraction**: Hardware-specific adaptations

## Implementation Status

### Completed Features
- **Visual Feedback Integration**: Real-time crop rectangle rendering during selection
- **Configuration Constants**: Centralized configuration with proper validation
- **Unit Tests**: Comprehensive test suite covering all crop selection functionality
- **Code Quality**: Full compliance with AGENTS.md rules and coding standards

### Implementation Details
The visual feedback has been successfully implemented in `crates/core/src/view/cover_editor.rs`:

- **Render Method Enhancement**: Added crop selection rectangle rendering in the `render()` method
- **Coordinate Normalization**: Proper coordinate handling for rectangle geometry
- **Minimum Size Validation**: Ensures meaningful selections only
- **Border Styling**: Configurable border thickness and color
- **Intersection Handling**: Proper viewport intersection for clipping

### Testing Implementation
Created comprehensive unit tests in `crates/core/src/view/cover_editor_tests.rs`:

- **Coordinate Normalization Tests**: Validates coordinate processing logic
- **Rectangle Creation Tests**: Verifies proper rectangle geometry
- **Size Validation Tests**: Ensures minimum selection size enforcement
- **State Transition Tests**: Validates crop state machine transitions
- **Configuration Tests**: Verifies constant values and border specifications

## Conclusion

This implementation plan has been successfully executed, enhancing the crop selection feature by adding real-time visual feedback, significantly improving the user experience. The implementation follows Plato's architectural principles:

- **Minimal Changes**: Focused enhancement without disrupting existing functionality
- **Modular Design**: Clear separation of concerns and well-defined interfaces
- **Error Handling**: Robust error management with graceful degradation
- **Testing**: Comprehensive testing strategy covering unit, integration, and manual testing
- **Documentation**: Complete API documentation and architectural guidance

The implementation leverages existing infrastructure while adding the missing visual feedback component, making the crop selection feature more intuitive and user-friendly. The code is production-ready and follows all AGENTS.md compliance requirements.