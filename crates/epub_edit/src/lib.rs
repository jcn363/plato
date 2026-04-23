//! EPUB editing library for Plato e-reader.
//!
//! This library provides functionality for editing EPUB files, including:
//! - Metadata editing (title, author, language, etc.)
//! - Chapter content modification
//! - Search and replace operations
//! - Bookmark management
//! - Table of contents generation
//! - Content validation
//! - Chapter statistics
//! - Image and CSS listing
//!
//! The main entry point is the [`EpubEditorCore`] struct, which loads an EPUB
//! file and provides methods for editing its contents.

#![warn(missing_docs)]

mod chapter;
mod editor;
mod parser;
mod search;
mod types;
mod validation;

pub use editor::EpubEditorCore;
pub use types::{
    Bookmark, CSSInfo, ChapterStatistics, EpubChapter, EpubMetadata, ImageInfo, SearchOptions,
    SpellCheckResult, SpellError, UndoAction, ValidationIssue, ValidationResult,
};
