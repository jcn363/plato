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
//!
//! ## Module Organization
//!
//! The Reader functionality is organized as follows:
//! - **reader_core**: Shared types and Reader struct definition
//! - **reader**: Main Reader implementation with all trait methods (to be split)
//! - **reader_rendering**: Page rendering, animation, text extraction, display
//! - **reader_gestures**: Touch/gesture handling, input processing
//! - **reader_annotations**: Annotations, notes, highlighting, bookmarks
//! - **reader_dialogs**: Input dialogs and text entry interactions
//! - **reader_settings**: Settings menus and configuration
//! - **reader_search**: Search functionality and result management
//! - **reader_navigation**: Page navigation, chapter switching, history
//! - **reader_toc**: Table of contents management
//! - **reader_ui**: UI update helpers and basic view operations
//! - **reader_events**: Event handling for device events and keyboard

// Core types and definitions
pub mod reader_core;
pub use reader_core::{
    Contrast, PageAnimKind, PageAnimation, RenderChunk, Resource, Selection, State, ViewPort,
};

// Main implementation (to be split across modules)
pub mod reader;
pub use reader::Reader;

// Feature modules
pub mod reader_annotations;
pub mod reader_dialogs;
pub mod reader_events;
pub mod reader_gestures;
pub mod reader_navigation;
pub mod reader_rendering;
pub mod reader_search;
pub mod reader_settings;
pub mod reader_toc;
pub mod reader_ui;

// Internal helper modules
pub(crate) mod reader_menus;
pub(crate) mod reader_rendering_impl;
pub(crate) mod reader_setters;
