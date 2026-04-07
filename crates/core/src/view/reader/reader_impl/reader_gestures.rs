//! Reader Gestures and Input Module
//!
//! Handles touch gestures, button input, stylus interaction, and event processing.
//!
//! ## Current Status
//! This module is a placeholder. The actual gesture handling is currently implemented
//! in the main `Reader::handle_event()` method in `reader.rs`. This module will
//! contain the extracted gesture handling logic in a future refactoring phase.
//!
//! ## Planned Methods
//! The following methods are candidates for extraction from `Reader::handle_event()`:
//!
//! ### Gesture Handlers
//! - `handle_swipe_gesture()` - Left/right swipes for page navigation
//! - `handle_tap_gesture()` - Single tap for menu toggles and link navigation  
//! - `handle_long_press()` - Long press for text selection and context menus
//! - `handle_pinch_gesture()` - Pinch to zoom (if supported)
//! - `handle_stylus_input()` - Stylus/pen input for annotations and margin cropping
//!
//! ### Button Handlers
//! - `handle_physical_buttons()` - Home, power, and device-specific buttons
//! - `handle_navigation_buttons()` - Forward/back page navigation
//!
//! ### Event Dispatching
//! - `handle_menu_event()` - Menu selection callbacks
//! - `handle_margin_cropper_event()` - Margin cropper interaction
//! - `handle_selection_event()` - Text selection callbacks
//!
//! ## Architecture Notes
//!
//! ### Why It's Still in reader.rs
//! The gesture handling is tightly coupled with:
//! - Document state (current page, position, layout)
//! - View state (menu visibility, focus)
//! - Rendering state (cache invalidation)
//! - Multiple Reader methods (update, layout, etc.)
//!
//! Full extraction would require passing 15+ parameters per method.
//! **Estimated effort**: 6-8 hours to refactor with acceptable parameter passing.
//!
//! ### Potential Improvements
//! 1. **Gesture Abstraction**: Create a `GestureProcessor` trait for different device types
//! 2. **Event Queue**: Buffer events instead of processing immediately
//! 3. **State Machine**: Use explicit state transitions for complex interactions
//! 4. **Sub-Handlers**: Extract into separate functions grouped by gesture type
//!
//! ## Dependencies
//! When implemented, will import:
//! - `crate::input::*` - Input event types
//! - `crate::gesture::*` - Gesture event types
//! - `crate::context::Context` - Device and display context
//! - `crate::view::*` - Hub, RenderQueue, RenderData
//! - `super::reader_core::*` - Reader state types
//! - `super::reader_settings::*` - Settings menu functions
//!
//! ## Size Estimate
//! Extracting all gesture handling would be ~400-500 lines of code.
//! Main challenge: reducing the parameter passing overhead.
