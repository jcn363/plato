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
//! - **reader_rendering**: Page rendering, animation, text extraction, display (WIP)
//! - **reader_gestures**: Touch/gesture handling, input processing (WIP)
//! - **reader_annotations**: Annotations, notes, highlighting, bookmarks (WIP)
//! - **reader_dialogs**: Input dialogs and text entry interactions (WIP)
//! - **reader_settings**: Settings menus and configuration (WIP)
//! - **reader_search**: Search functionality and result management (WIP)

// Core types and definitions
pub mod reader_core;
pub use reader_core::{Contrast, Selection, State};

// Main implementation (to be split across modules)
mod reader;
pub use reader::Reader;

// Feature modules (WIP)
mod reader_annotations;
mod reader_dialogs;
mod reader_gestures;
mod reader_rendering;
mod reader_search;
mod reader_settings;
