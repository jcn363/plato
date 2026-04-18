//! Reader Search Handler Module
//!
//! This module handles all search functionality for the Reader view,
//! including text search, result navigation, and search history.

use crate::context::Context;
use crate::document::Location;
use crate::geom::LinearDir;
use crate::geom::Rectangle;
use crate::rustc_hash::FxHashMap;
use crate::view::reader::reader_impl::reader_core::Search;
use crate::view::search_bar::SearchBar;
use crate::view::ViewId;
use crate::view::{Hub, Id, RenderQueue};
use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;

/// Search result information
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub page: usize,
    pub location: Location,
    pub text: String,
    pub context: String,
}

/// Search handler for the Reader view
pub struct ReaderSearchHandler {
    pub id: Id,
    pub current_search: Option<Search>,
    pub search_direction: LinearDir,
    pub search_history: VecDeque<String>,
    pub search_results: Vec<SearchResult>,
    pub current_result_index: usize,
    pub max_history_size: usize,
}

impl ReaderSearchHandler {
    /// Create a new search handler
    pub fn new(id: Id) -> Self {
        Self {
            id,
            current_search: None,
            search_direction: LinearDir::Forward,
            search_history: VecDeque::new(),
            search_results: Vec::new(),
            current_result_index: 0,
            max_history_size: 50,
        }
    }

    /// Start a new search
    pub fn start_search(&mut self, query: String, direction: LinearDir) {
        // Add to history
        if !query.is_empty() {
            self.search_history.push_back(query.clone());
            if self.search_history.len() > self.max_history_size {
                self.search_history.pop_front();
            }
        }

        // Initialize search
        self.current_search = Some(Search {
            _query: query,
            results: Vec::new(),
            index: 0,
            running: AtomicBool::new(false),
            _results_count: 0,
            highlights: FxHashMap::default(),
            direction,
        });
        self.search_direction = direction;
        self.current_result_index = 0;
    }

    /// Clear current search
    pub fn clear_search(&mut self) {
        self.current_search = None;
        self.search_results.clear();
        self.current_result_index = 0;
    }

    /// Add a search result
    pub fn add_result(&mut self, result: SearchResult) {
        self.search_results.push(result);
    }

    /// Get current search results
    pub fn get_results(&self) -> &[SearchResult] {
        &self.search_results
    }

    /// Get current search result
    pub fn get_current_result(&self) -> Option<&SearchResult> {
        self.search_results.get(self.current_result_index)
    }

    /// Navigate to next result
    pub fn next_result(&mut self) -> bool {
        if self.current_result_index < self.search_results.len().saturating_sub(1) {
            self.current_result_index += 1;
            true
        } else {
            false
        }
    }

    /// Navigate to previous result
    pub fn previous_result(&mut self) -> bool {
        if self.current_result_index > 0 {
            self.current_result_index -= 1;
            true
        } else {
            false
        }
    }

    /// Jump to a specific result
    pub fn jump_to_result(&mut self, index: usize) -> bool {
        if index < self.search_results.len() {
            self.current_result_index = index;
            true
        } else {
            false
        }
    }

    /// Get search history
    pub fn get_history(&self) -> Vec<&String> {
        self.search_history.iter().collect()
    }

    /// Clear search history
    pub fn clear_history(&mut self) {
        self.search_history.clear();
    }

    /// Check if search is active
    pub fn is_searching(&self) -> bool {
        self.current_search.is_some()
    }

    /// Get current search query
    pub fn get_current_query(&self) -> Option<&str> {
        self.current_search.as_ref().map(|s| s._query.as_str())
    }

    /// Get search direction
    pub fn get_search_direction(&self) -> LinearDir {
        self.search_direction
    }

    /// Set search direction
    pub fn set_search_direction(&mut self, direction: LinearDir) {
        self.search_direction = direction;
        if let Some(ref mut search) = self.current_search {
            search.direction = direction;
        }
    }

    /// Get result count
    pub fn result_count(&self) -> usize {
        self.search_results.len()
    }

    /// Get current result index (1-based)
    pub fn current_result_number(&self) -> usize {
        self.current_result_index + 1
    }

    /// Check if at first result
    pub fn is_at_first_result(&self) -> bool {
        self.current_result_index == 0
    }

    /// Check if at last result
    pub fn is_at_last_result(&self) -> bool {
        self.current_result_index >= self.search_results.len().saturating_sub(1)
    }

    /// Create search bar with current query
    pub fn create_search_bar(&self, rect: Rectangle, context: &mut Context) -> SearchBar {
        let query = self
            .current_search
            .as_ref()
            .map(|s| s._query.as_str())
            .unwrap_or("")
            .to_string();
        SearchBar::new(rect, ViewId::SearchBarInput, "", &query, context)
    }

    /// Handle search completion
    pub fn handle_search_complete(&mut self, results: Vec<SearchResult>) {
        self.search_results = results;
        self.current_result_index = 0;
    }

    /// Handle search navigation
    pub fn handle_navigation(
        &mut self,
        direction: LinearDir,
        _hub: &Hub,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match direction {
            LinearDir::Forward => self.next_result(),
            LinearDir::Backward => self.previous_result(),
        }
    }

    /// Get search statistics
    pub fn get_search_stats(&self) -> (usize, usize, Option<&str>) {
        (
            self.result_count(),
            self.current_result_number(),
            self.get_current_query(),
        )
    }

    /// Update search results for a new page
    pub fn update_page_results(&mut self, page: usize, page_results: Vec<SearchResult>) {
        // Remove old results for this page and add new ones
        self.search_results.retain(|r| r.page != page);
        self.search_results.extend(page_results);
        // Sort by page number since Location doesn't implement Ord
        self.search_results.sort_by_key(|r| r.page);

        // Adjust current index if needed
        if self.current_result_index >= self.search_results.len() {
            self.current_result_index = self.search_results.len().saturating_sub(1);
        }
    }
}
