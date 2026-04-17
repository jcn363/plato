//! Library Management Module
//!
//! This module provides book library management functionality including:
//! - Library creation and initialization
//! - Book import and scanning
//! - Metadata management
//! - File operations (rename, move, copy, remove)
//! - Library maintenance and cleanup
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
//! - **query**: Book searching and filtering
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
//!
//! ## Usage
//!
//! ```rust,ignore
//! use plato_core::library::Library;
//! use plato_core::settings::LibraryMode;
//!
//! let library = Library::new("/path/to/library", LibraryMode::Database)?;
//! ```

mod import;
mod maintenance;
mod manage;
mod query;
mod scan;
mod types;

pub use types::Library;
