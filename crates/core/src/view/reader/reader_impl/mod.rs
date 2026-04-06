//! Reader Implementation
//!
//! This module provides the core document reading view for Plato. It handles:
//! - Document loading and rendering (PDF, EPUB, HTML)
//! - Text selection and annotation
//! - Table of contents navigation
//! - Reading state persistence
//! - Page navigation and zoom gestures
//!
//! The main [`Reader`] struct is the primary entry point for reading documents.

mod reader;

pub use reader::{Contrast, Reader, Selection, State};
