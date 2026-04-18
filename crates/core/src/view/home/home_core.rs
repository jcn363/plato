//! Home Core Module - Data Model and State Management
//!
//! This module contains the core data structures and state management
//! for the Home view. It defines the Home struct, its fields, and
//! related data types used throughout the home module.
//!
//! ## Module Structure
//!
//! - `Home` struct - Main view state container
//! - `Fetcher` struct - Background fetcher state
//! - `BookMenuData` struct - Book menu context data
//!
//! ## Design Notes
//!
//! This module was extracted from `mod.rs` as part of the Phase 5 refactoring
//! to separate data model concerns from UI layout and event handling per
//! AGENTS.md modular design rules.

use std::path::{Path, PathBuf};
use std::process::Child;

use crate::geom::Rectangle;
use crate::metadata::{BookQuery, Metadata, SimpleStatus, SortMethod};
use crate::settings::{FirstColumn, SecondColumn};
use crate::view::{Id, View, ViewId};
use rustc_hash::{FxHashMap, FxHashSet};

/// Trash directory name constant
pub const TRASH_DIRNAME: &str = ".trash";

/// Home Library View - Main struct containing all state
#[derive(Debug)]
pub struct Home {
    pub id: Id,
    pub rect: Rectangle,
    pub children: Vec<Box<dyn View>>,
    pub current_page: usize,
    pub pages_count: usize,
    pub shelf_index: usize,
    pub focus: Option<ViewId>,
    pub query: Option<BookQuery>,
    pub sort_method: SortMethod,
    pub reverse_order: bool,
    pub visible_books: Metadata,
    pub current_directory: PathBuf,
    pub target_document: Option<PathBuf>,
    pub background_fetchers: FxHashMap<u32, Fetcher>,
    pub batch_mode: bool,
    pub batch_selected: FxHashSet<usize>,
    // UI toggle fields for modularized components
    pub keyboard: Option<Box<dyn View>>,
    pub address_bar: Option<Box<dyn View>>,
    pub navigation_bar: Option<Box<dyn View>>,
    pub search_bar: Option<Box<dyn View>>,
    pub go_to_page: Option<Box<dyn View>>,
    pub sort_menu: Option<Box<dyn View>>,
    pub book_menu: Option<Box<dyn View>>,
    pub library_menu: Option<Box<dyn View>>,
    pub settings_menu: Option<Box<dyn View>>,
    pub shelf: Option<Box<dyn View>>,
    pub book_view: Option<Box<dyn View>>,
    pub directory_view: Option<Box<dyn View>>,
    #[allow(dead_code)] // Reserved for future bottom bar functionality
    pub bottom_bar: Option<Box<dyn View>>,
}

/// Background fetcher state for library scanning
#[derive(Debug)]
pub struct Fetcher {
    pub path: PathBuf,
    pub full_path: PathBuf,
    pub process: Child,
    pub sort_method: Option<SortMethod>,
    pub first_column: Option<FirstColumn>,
    pub second_column: Option<SecondColumn>,
}

/// Book menu context data for menu operations
#[derive(Debug)]
#[allow(dead_code)] // Reserved for future book menu functionality
pub struct BookMenuData {
    pub path: PathBuf,
    pub kind: String,
    pub author: String,
    pub simple_status: SimpleStatus,
    pub libraries: Vec<(usize, String)>,
    pub library_home: PathBuf,
}

impl Home {
    /// Get the current directory path
    pub fn current_directory(&self) -> &Path {
        &self.current_directory
    }

    /// Get the current page number
    pub fn current_page(&self) -> usize {
        self.current_page
    }

    /// Get the total pages count
    pub fn pages_count(&self) -> usize {
        self.pages_count
    }

    /// Get the current sort method
    pub fn sort_method(&self) -> SortMethod {
        self.sort_method
    }

    /// Check if reverse order is enabled
    pub fn reverse_order(&self) -> bool {
        self.reverse_order
    }

    /// Get visible books metadata
    pub fn visible_books(&self) -> &Metadata {
        &self.visible_books
    }

    /// Get mutable reference to visible books
    pub fn visible_books_mut(&mut self) -> &mut Metadata {
        &mut self.visible_books
    }

    /// Check if batch mode is active
    pub fn batch_mode(&self) -> bool {
        self.batch_mode
    }

    /// Get batch selected items
    pub fn batch_selected(&self) -> &FxHashSet<usize> {
        &self.batch_selected
    }

    /// Get mutable reference to batch selected
    pub fn batch_selected_mut(&mut self) -> &mut FxHashSet<usize> {
        &mut self.batch_selected
    }

    /// Get the target document if any
    pub fn target_document(&self) -> Option<&Path> {
        self.target_document.as_deref()
    }

    /// Set the target document
    pub fn set_target_document(&mut self, path: Option<PathBuf>) {
        self.target_document = path;
    }

    /// Get background fetchers
    pub fn background_fetchers(&self) -> &FxHashMap<u32, Fetcher> {
        &self.background_fetchers
    }

    /// Get mutable reference to background fetchers
    pub fn background_fetchers_mut(&mut self) -> &mut FxHashMap<u32, Fetcher> {
        &mut self.background_fetchers
    }

    /// Clear batch selection
    pub fn clear_batch_selection(&mut self) {
        self.batch_selected.clear();
        self.batch_mode = false;
    }

    /// Toggle an item in batch selection
    pub fn toggle_batch_selection(&mut self, index: usize) {
        if self.batch_selected.contains(&index) {
            self.batch_selected.remove(&index);
        } else {
            self.batch_selected.insert(index);
        }
    }

    /// Set batch mode
    pub fn set_batch_mode(&mut self, enabled: bool) {
        self.batch_mode = enabled;
        if !enabled {
            self.batch_selected.clear();
        }
    }

    /// Get the shelf index
    pub fn shelf_index(&self) -> usize {
        self.shelf_index
    }

    /// Set the shelf index
    pub fn set_shelf_index(&mut self, index: usize) {
        self.shelf_index = index;
    }

    /// Get current query
    pub fn query(&self) -> Option<&BookQuery> {
        self.query.as_ref()
    }

    /// Set current query
    pub fn set_query(&mut self, query: Option<BookQuery>) {
        self.query = query;
    }

    /// Get focus view ID
    pub fn focus(&self) -> Option<ViewId> {
        self.focus
    }

    /// Set focus view ID
    pub fn set_focus(&mut self, focus: Option<ViewId>) {
        self.focus = focus;
    }

    /// Update current page
    pub fn set_current_page(&mut self, page: usize) {
        self.current_page = page;
    }

    /// Update pages count
    pub fn set_pages_count(&mut self, count: usize) {
        self.pages_count = count;
    }

    /// Update current directory
    pub fn set_current_directory(&mut self, dir: PathBuf) {
        self.current_directory = dir;
    }

}
