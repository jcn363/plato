//! Library Management Module
//!
//! This module provides book library management functionality including:
//! - Library creation and initialization
//! - Book import and scanning
//! - Metadata management
//! - File operations (rename, move, copy, remove)
//! - Library maintenance and cleanup
//! - Fuzzy search for book titles and authors using Levenshtein distance
//! - Advanced regex search for pattern matching in metadata
//!
//! ## Architecture
//!
//! The library module follows a layered architecture:
//!
//! - **types**: Core data structures (`Library`, `Book`, metadata types)
//! - **import**: External file import functionality
//! - **scan**: Directory scanning and book discovery
//! - **manage**: File operations (rename, move, copy, remove)
//! - **maintenance**: Library cleanup and optimization
//! - **query**: Book searching and filtering including fuzzy and regex search
//!
//! ## Module Hierarchy
//!
//! ```text
//! library/
//! ├── mod.rs       (module exports and documentation)
//! ├── types.rs     (core data structures)
//! ├── import.rs    (file import)
//! ├── scan.rs      (directory scanning)
//! ├── manage.rs    (file operations)
//! ├── maintenance.rs (cleanup)
//! └── query.rs     (search/filter)
//! ```
//!
//! ## Dependencies
//!
//! This module depends on:
//! - `metadata` - For book metadata extraction
//! - `settings` - For library configuration
//! - `helpers` - For file system utilities
//! - `levenshtein` - For fuzzy string matching
//! - `regex` - For regex pattern matching
//!
//! ## Usage
//!
//! ```rust,ignore
//! use plato_core::library::Library;
//! use plato_core::settings::LibraryMode;
//!
//! let library = Library::new("/path/to/library", LibraryMode::Database)?;
//!
//! // Fuzzy search for books
//! let results = library.fuzzy_search("hary poter", Some(0.7));
//!
//! // Regex search for books
//! let results = library.regex_search(r"Harry.*Potter")?;
//! ```

mod import;
mod maintenance;
mod manage;
#[cfg(test)]
mod manage_tests;
mod query;
mod scan;
mod types;

pub use types::Library;
