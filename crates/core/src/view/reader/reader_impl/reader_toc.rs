//! Reader Table of Contents Module
//!
//! This module handles table of contents functionality for the Reader view,
//! including TOC navigation, page lookup, and chapter management.

use crate::context::Context;
use crate::document::{SimpleTocEntry, TocEntry, TocLocation};
use crate::framebuffer::UpdateMode;
use crate::metadata::Info;
use crate::view::{Hub, RenderData, RenderQueue, View};
use crate::view::notification::Notification;
use rustc_hash::FxHashMap;

use super::reader::Reader;

/// Table of contents manager for the Reader view
pub struct ReaderTocManager {
    pub toc_entries: Vec<TocEntry>,
    _simple_toc: Vec<SimpleTocEntry>,
    _page_map: FxHashMap<String, usize>,
    _current_chapter: Option<usize>,
}

impl ReaderTocManager {
    /// Create a new TOC manager
    pub fn new() -> Self {
        Self {
            toc_entries: Vec::new(),
            _simple_toc: Vec::new(),
            _page_map: FxHashMap::default(),
            _current_chapter: None,
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

    /// Find page by name
    pub fn find_page_by_name(&self, info: &Info, name: &str) -> Option<usize> {
        // Check page map first
        if let Some(&page) = self._page_map.get(name) {
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

    /// Navigate to chapter
    pub fn navigate_to_chapter(
        &mut self,
        chapter_index: usize,
        _current_page: usize,
    ) -> Option<usize> {
        if chapter_index < self.toc_entries.len() {
            if let Some(page) = self.toc_entries[chapter_index].page {
                return Some(page);
            }
        }
        None
    }
}

impl Default for ReaderTocManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader {
    /// Show table of contents menu
    pub fn handle_show_table_of_contents(
        &mut self,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let toc_available = self
            .info
            .simple_toc
            .as_ref()
            .map(|t| !t.is_empty())
            .unwrap_or(false);

        if toc_available {
            self.toc_manager.build_toc(&self.info);
            let chapter_count = self.toc_manager.toc_entries.len();

            if chapter_count > 0 {
                let msg = format!("Table of Contents: {} chapters", chapter_count);
                let notif = Notification::new(msg, hub, rq, context);
                self.children.push(Box::new(notif) as Box<dyn View>);
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                return;
            }
        }

        let msg = "No table of contents available".to_string();
        let notif = Notification::new(msg, hub, rq, context);
        self.children.push(Box::new(notif) as Box<dyn View>);
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }
}
