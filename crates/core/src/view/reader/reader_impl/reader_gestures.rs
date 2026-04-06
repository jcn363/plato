//! Reader Gestures Module
//!
//! Handles touch/gesture handling and input processing.
//!
//! ## Methods to Move Here  
//! - `handle_event()` - Main event dispatcher (~1400 lines - LARGEST)
//! - Touch/swipe/pinch gesture handling
//! - Button input processing
//! - Stylus input handling
//! - Margin cropper gestures
//! - Menu selection callbacks
//!
//! ## Size
//! This is the largest module (~1,400 lines from handle_event alone).
//! Will need careful extraction to maintain readability.
//!
//! ## Dependencies
//! Relies on most other Reader methods (navigation, rendering, settings, annotations).
//!
//! ## Future Use
//! Types from reader_core will be imported when methods are extracted.
