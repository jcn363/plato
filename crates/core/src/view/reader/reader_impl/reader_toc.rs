//! Reader Table of Contents Module
//!
//! This module handles table of contents functionality for the Reader view,
//! including TOC navigation, page lookup, and chapter management.

use crate::document::{SimpleTocEntry, TocEntry, TocLocation};
use crate::metadata::Info;
use rustc_hash::FxHashMap;

/// Table of contents manager for the Reader view
pub struct ReaderTocManager {
    pub toc_entries: Vec<TocEntry>,
    pub simple_toc: Vec<SimpleTocEntry>,
    pub page_map: FxHashMap<String, usize>,
    pub current_chapter: Option<usize>,
}

impl ReaderTocManager {
    /// Create a new TOC manager
    pub fn new() -> Self {
        Self {
            toc_entries: Vec::new(),
            simple_toc: Vec::new(),
            page_map: FxHashMap::default(),
            current_chapter: None,
        }
    }

    /// Build table of contents from document metadata
    pub fn build_toc(&mut self, info: &Info) -> Option<Vec<TocEntry>> {
        if let Some(ref simple_toc) = info.simple_toc {
            let mut index = 0;
            // Pre-collect all page names to avoid borrow issues
            let toc = self.build_toc_aux_internal(simple_toc, &mut index, info);
            self.toc_entries = toc.clone();
            Some(toc)
        } else {
            None
        }
    }

    /// Internal TOC builder that doesn't use closure
    fn build_toc_aux_internal(
        &self,
        simple_toc: &[SimpleTocEntry],
        index: &mut usize,
        info: &Info,
    ) -> Vec<TocEntry> {
        let mut toc = Vec::new();

        for entry in simple_toc {
            let (title, location, children) = match entry {
                SimpleTocEntry::Leaf(t, loc) => (t.as_str(), loc, &[][..]),
                SimpleTocEntry::Container(t, loc, c) => (t.as_str(), loc, c.as_slice()),
            };

            let toc_entry = TocEntry {
                index: *index,
                location: location.clone().into(),
                title: title.to_string(),
                children: if !children.is_empty() {
                    self.build_toc_aux_internal(children, index, info)
                } else {
                    Vec::new()
                },
                page: self.find_page_by_name(info, title),
                level: Self::calculate_level(children),
            };

            toc.push(toc_entry);
            *index += 1;
        }

        toc
    }

    /// Build table of contents from simple TOC entries
    pub fn build_toc_aux<F>(
        &mut self,
        simple_toc: &[SimpleTocEntry],
        index: &mut usize,
        find_page: &F,
    ) -> Vec<TocEntry>
    where
        F: Fn(&str) -> Option<usize>,
    {
        let mut toc = Vec::new();

        for entry in simple_toc {
            let (title, location, children) = match entry {
                SimpleTocEntry::Leaf(t, loc) => (t.as_str(), loc, &[][..]),
                SimpleTocEntry::Container(t, loc, c) => (t.as_str(), loc, c.as_slice()),
            };

            let toc_entry = TocEntry {
                index: *index,
                location: location.clone().into(),
                title: title.to_string(),
                children: if !children.is_empty() {
                    self.build_toc_aux(children, index, &find_page)
                } else {
                    Vec::new()
                },
                page: None,
                level: 0,
            };

            toc.push(toc_entry);
            *index += 1;
        }

        toc
    }

    /// Find page by name
    pub fn find_page_by_name(&self, info: &Info, name: &str) -> Option<usize> {
        // Check page map first
        if let Some(&page) = self.page_map.get(name) {
            return Some(page);
        }

        // Search in simple TOC
        if let Some(ref simple_toc) = info.simple_toc {
            for entry in simple_toc {
                let (title, location, children) = match entry {
                    SimpleTocEntry::Leaf(t, loc) => (t.as_str(), loc, &[][..]),
                    SimpleTocEntry::Container(t, loc, c) => (t.as_str(), loc, c.as_slice()),
                };
                if title == name {
                    return Self::extract_page(location);
                }
                // Search recursively in children
                if let Some(page) = self.find_page_in_children(children, name) {
                    return Some(page);
                }
            }
        }

        None
    }

    /// Find page in children entries
    fn find_page_in_children(&self, children: &[SimpleTocEntry], name: &str) -> Option<usize> {
        for entry in children {
            let (title, location, grandchildren) = match entry {
                SimpleTocEntry::Leaf(t, loc) => (t.as_str(), loc, &[][..]),
                SimpleTocEntry::Container(t, loc, c) => (t.as_str(), loc, c.as_slice()),
            };
            if title == name {
                return Self::extract_page(location);
            }
            if let Some(page) = self.find_page_in_children(grandchildren, name) {
                return Some(page);
            }
        }
        None
    }

    /// Extract page number from TocLocation
    fn extract_page(location: &TocLocation) -> Option<usize> {
        match location {
            TocLocation::Exact(page) => Some(*page),
            TocLocation::Uri(_) => None,
        }
    }

    /// Calculate nesting level from children
    fn calculate_level(children: &[SimpleTocEntry]) -> usize {
        if children.is_empty() {
            0
        } else {
            1 + children
                .iter()
                .map(|c| match c {
                    SimpleTocEntry::Leaf(_, _) => 0,
                    SimpleTocEntry::Container(_, _, grandchildren) => {
                        Self::calculate_level(grandchildren)
                    }
                })
                .max()
                .unwrap_or(0)
        }
    }

    /// Get current table of contents
    pub fn get_toc(&self) -> &[TocEntry] {
        &self.toc_entries
    }

    /// Get simple table of contents
    pub fn get_simple_toc(&self) -> &[SimpleTocEntry] {
        &self.simple_toc
    }

    /// Get current chapter
    pub fn get_current_chapter(&self) -> Option<usize> {
        self.current_chapter
    }

    /// Set current chapter
    pub fn set_current_chapter(&mut self, chapter: Option<usize>) {
        self.current_chapter = chapter;
    }

    /// Navigate to chapter
    pub fn navigate_to_chapter(
        &mut self,
        chapter_index: usize,
        _current_page: usize,
    ) -> Option<usize> {
        if chapter_index < self.toc_entries.len() {
            if let Some(page) = self.toc_entries[chapter_index].page {
                self.current_chapter = Some(chapter_index);
                return Some(page);
            }
        }
        None
    }

    /// Get chapter for current page
    pub fn get_chapter_for_page(&self, page: usize) -> Option<usize> {
        for (i, entry) in self.toc_entries.iter().enumerate() {
            if let Some(entry_page) = entry.page {
                if entry_page <= page {
                    // Check if this is the last chapter or if next chapter starts after current page
                    let is_last_chapter = i == self.toc_entries.len() - 1;
                    let next_chapter_starts_after = if !is_last_chapter {
                        self.toc_entries[i + 1].page.map_or(false, |p| p > page)
                    } else {
                        true
                    };

                    if is_last_chapter || next_chapter_starts_after {
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    /// Get next chapter
    pub fn get_next_chapter(&self, current_page: usize) -> Option<usize> {
        if let Some(current_chapter) = self.get_chapter_for_page(current_page) {
            if current_chapter + 1 < self.toc_entries.len() {
                return Some(current_chapter + 1);
            }
        }
        None
    }

    /// Get previous chapter
    pub fn get_previous_chapter(&self, current_page: usize) -> Option<usize> {
        if let Some(current_chapter) = self.get_chapter_for_page(current_page) {
            if current_chapter > 0 {
                return Some(current_chapter - 1);
            }
        }
        None
    }

    /// Update page map
    pub fn update_page_map(&mut self, page_map: FxHashMap<String, usize>) {
        self.page_map = page_map;
    }

    /// Clear TOC data
    pub fn clear(&mut self) {
        self.toc_entries.clear();
        self.simple_toc.clear();
        self.page_map.clear();
        self.current_chapter = None;
    }

    /// Check if TOC is available
    pub fn is_toc_available(&self) -> bool {
        !self.toc_entries.is_empty()
    }

    /// Get TOC statistics
    pub fn get_toc_stats(&self) -> TocStats {
        TocStats {
            total_chapters: self.toc_entries.len(),
            max_depth: self.calculate_max_depth(&self.toc_entries),
            has_page_numbers: self.toc_entries.iter().any(|e| e.page.is_some()),
        }
    }

    /// Calculate maximum depth of TOC
    fn calculate_max_depth(&self, entries: &[TocEntry]) -> u32 {
        if entries.is_empty() {
            return 0;
        }

        let mut max_depth = 0;
        for entry in entries {
            let depth = entry.level as u32;
            let child_depth = self.calculate_max_depth(&entry.children);
            max_depth = max_depth.max(depth).max(child_depth);
        }

        max_depth
    }
}

/// TOC statistics
#[derive(Debug, Clone)]
pub struct TocStats {
    pub total_chapters: usize,
    pub max_depth: u32,
    pub has_page_numbers: bool,
}

/// Utility functions for TOC management
pub mod utils {
    use super::*;

    /// Create default TOC manager
    pub fn create_default_toc_manager() -> ReaderTocManager {
        ReaderTocManager::new()
    }

    /// Flatten TOC entries into a linear list
    pub fn flatten_toc(entries: &[TocEntry]) -> Vec<TocEntry> {
        let mut flattened = Vec::new();

        for entry in entries {
            flattened.push(TocEntry {
                index: entry.index,
                location: entry.location.clone(),
                title: entry.title.clone(),
                children: Vec::new(),
                page: entry.page,
                level: entry.level,
            });

            // Recursively add children
            flattened.extend(flatten_toc(&entry.children));
        }

        flattened
    }

    /// Search TOC entries by title
    pub fn search_toc_by_title<'a>(entries: &'a [TocEntry], query: &str) -> Vec<&'a TocEntry> {
        let mut matches = Vec::new();

        for entry in entries {
            if entry.title.to_lowercase().contains(&query.to_lowercase()) {
                matches.push(entry);
            }

            // Search in children
            matches.extend(search_toc_by_title(&entry.children, query));
        }

        matches
    }

    /// Get chapter title for page
    pub fn get_chapter_title_for_page(entries: &[TocEntry], page: usize) -> Option<String> {
        for entry in entries {
            if let Some(entry_page) = entry.page {
                if entry_page <= page {
                    // Check if this is the last chapter or if next chapter starts after current page
                    let is_last_chapter = entry.children.is_empty()
                        && entries.iter().all(|e| e.page.map_or(true, |p| p <= page));

                    if is_last_chapter
                        || entry
                            .children
                            .iter()
                            .all(|c| c.page.map_or(true, |p| p > page))
                    {
                        return Some(entry.title.clone());
                    }
                }
            }

            // Check children
            if let Some(title) = get_chapter_title_for_page(&entry.children, page) {
                return Some(title);
            }
        }

        None
    }

    /// Validate TOC structure
    pub fn validate_toc_structure(entries: &[TocEntry]) -> Vec<String> {
        let mut errors = Vec::new();

        for (i, entry) in entries.iter().enumerate() {
            if entry.title.is_empty() {
                errors.push(format!("Chapter {} has empty title", i + 1));
            }

            if entry.level == 0 {
                errors.push(format!("Chapter {} has invalid level 0", i + 1));
            }

            // Validate children
            let child_errors = validate_toc_structure(&entry.children);
            for error in child_errors {
                errors.push(format!("Chapter {}.{}", i + 1, error));
            }
        }

        errors
    }
}
