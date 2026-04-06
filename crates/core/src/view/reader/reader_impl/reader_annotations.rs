//! Reader Annotations Module
//!
//! Handles annotation management, notes, highlighting, and bookmarks.
//!
//! ## Methods to Move Here
//! - `toggle_edit_note()` - Edit annotation note (~45 lines)
//! - `toggle_annotation_menu()` - Annotation context menu (~70 lines)
//! - `find_annotation_ref()` - Lookup annotation
//! - `find_annotation_mut()` - Lookup and mutate annotation
//! - `toggle_bookmark()` - Bookmark management (~15 lines)
//! - `update_annotations()` - Update annotation UI
//! - `go_to_annotation()` - Navigate to annotation
//!
//! ## Size
//! Moderate size (~600 lines total), relatively well-contained.
//!
//! ## Dependencies
//! Depends on Reader's info and selection state.

pub(crate) use super::reader_core::Selection;
