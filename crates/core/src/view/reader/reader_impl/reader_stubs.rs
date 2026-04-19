//! Reader Stub Methods
//!
//! This module contains stub and simplified implementations for Reader methods.
//! These methods provide basic functionality or placeholders for:
//! - View update operations
//! - Document pixmap loading and caching
//! - Text extraction and search
//! - Page navigation and history
//! - Annotation management
//! - Keyboard handling
//!
//! Many of these methods are delegating stubs that trigger UI updates
//! while the actual implementation logic resides in other specialized modules.

use crate::context::Context;
use crate::document::Location;
use crate::framebuffer::UpdateMode;
use crate::geom::{CycleDir, Point};
use crate::input::{ButtonStatus, DeviceEvent};
use crate::view::key::KeyKind;
use crate::view::{Event, Hub, RenderData, RenderQueue, View};

// Re-export types used in methods
use crate::document::TextLocation;
use crate::metadata::Annotation;
use chrono::Local;

use super::reader::Reader;
use super::reader_core::Search;

impl Reader {
    /// Helper: Queue a partial update for the reader view
    #[inline]
    fn queue_partial_update(&self, rq: &mut RenderQueue) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    /// Stub: Update the view
    pub fn update(
        &mut self,
        _update: Option<UpdateMode>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.queue_partial_update(rq);
    }

    /// Stub: Update the toolbar
    pub fn update_tool_bar(&mut self, rq: &mut RenderQueue, _context: &Context) {
        self.queue_partial_update(rq);
    }

    /// Stub: Update the bottom bar
    pub fn update_bottom_bar(&mut self, rq: &mut RenderQueue) {
        self.queue_partial_update(rq);
    }

    /// Save document reading state
    pub fn handle_save(&mut self, hub: &Hub, rq: &mut RenderQueue, _context: &mut Context) {
        // Update metadata with current reading position
        if let Some(ref mut r) = self.info.reader {
            r.current_page = self.current_page;
            r.pages_count = self.pages_count;
            r.finished = self.finished;

            // Save to library if available
            if !self.ephemeral {
                hub.send(Event::Save).ok();
            }
        }

        // Trigger UI update to reflect saved state
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    /// Stub: Handle focus change
    pub fn handle_focus(
        &mut self,
        _v: bool,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.queue_partial_update(rq);
    }

    /// Stub: Update annotations
    pub fn update_annotations(&mut self, _hub: &Hub, rq: &mut RenderQueue, _context: &mut Context) {
        self.queue_partial_update(rq);
    }

    /// Stub: Update non-inverted regions
    pub fn update_noninverted_regions(&mut self, rq: &mut RenderQueue) {
        self.queue_partial_update(rq);
    }

    /// Navigate to next or previous chapter using TOC
    pub fn go_to_chapter(
        &mut self,
        dir: CycleDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        // Check if TOC is available and build it using integrated toc_manager
        if let Some(ref simple_toc) = self.info.simple_toc {
            if simple_toc.is_empty() {
                self.queue_partial_update(rq);
                return;
            }

            self.toc_manager.build_toc(&self.info);

            // Find current chapter
            let current_chapter = self.toc_manager.get_chapter_for_page(self.current_page);

            // Get target chapter based on direction
            let target_chapter = match dir {
                CycleDir::Next => current_chapter.and_then(|c| {
                    if c + 1 < self.toc_manager.toc_entries.len() {
                        Some(c + 1)
                    } else {
                        None
                    }
                }),
                CycleDir::Previous => {
                    current_chapter.and_then(|c| if c > 0 { Some(c - 1) } else { None })
                }
            };

            // Navigate to target chapter's page
            if let Some(chapter_idx) = target_chapter {
                if let Some(page) = self.toc_manager.navigate_to_chapter(chapter_idx, self.current_page)
                {
                    self.go_to_page(page, true, hub, rq, context);
                    return;
                }
            }
        }

        self.queue_partial_update(rq);
    }

    /// Navigate to next or previous bookmark
    pub fn go_to_bookmark(
        &mut self,
        dir: CycleDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        // Check if bookmarks exist
        let bookmarks = self
            .info
            .reader
            .as_ref()
            .map(|r| &r.bookmarks)
            .filter(|b| !b.is_empty());

        if let Some(bookmarks) = bookmarks {
            // Find bookmark relative to current position
            let target_page = match dir {
                CycleDir::Next => bookmarks.iter().find(|&&b| b > self.current_page).copied(),
                CycleDir::Previous => bookmarks
                    .iter()
                    .rev()
                    .find(|&&b| b < self.current_page)
                    .copied(),
            };

            if let Some(page) = target_page {
                self.go_to_page(page, true, hub, rq, context);
                return;
            }
        }

        // No bookmark found, just update
        self.queue_partial_update(rq);
    }

    /// Navigate to next or previous annotation
    pub fn go_to_annotation(
        &mut self,
        dir: CycleDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        // Get annotations from document info
        let annotations = self
            .info
            .reader
            .as_ref()
            .map(|r| &r.annotations)
            .filter(|a| !a.is_empty());

        if let Some(annotations) = annotations {
            // Find annotation relative to current position
            // Extract page from TextLocation
            let target_annotation = match dir {
                CycleDir::Next => annotations.iter().find(|a| {
                    let page = a.selection[0].location();
                    page > self.current_page
                }),
                CycleDir::Previous => annotations.iter().rev().find(|a| {
                    let page = a.selection[0].location();
                    page < self.current_page
                }),
            };

            if let Some(annotation) = target_annotation {
                let page = annotation.selection[0].location();
                self.go_to_page(page, true, hub, rq, context);
                return;
            }
        }

        // No annotation found, just update
        self.queue_partial_update(rq);
    }

    /// Navigate to the last page of document
    pub fn go_to_last_page(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let last_page = self.pages_count.saturating_sub(1);
        if last_page != self.current_page {
            self.go_to_page(last_page, true, hub, rq, context);
        } else {
            self.queue_partial_update(rq);
        }
    }

    /// Stub: Directional scroll
    pub fn directional_scroll(
        &mut self,
        delta: Point,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.view_port.page_offset.y = (self.view_port.page_offset.y + delta.y as i32).max(0);
        self.view_port.page_offset.x = (self.view_port.page_offset.x + delta.x as i32).max(0);
        self.queue_partial_update(rq);
    }

    /// Stub: Vertical scroll
    pub fn vertical_scroll(
        &mut self,
        distance: i32,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.view_port.page_offset.y = (self.view_port.page_offset.y + distance).max(0);
        self.queue_partial_update(rq);
    }

    /// Toggle top toolbar visibility
    pub fn toggle_bars(
        &mut self,
        show: Option<bool>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.bars_visible = show.unwrap_or(!self.bars_visible);
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Stub: Toggle keyboard
    pub fn toggle_keyboard(
        &mut self,
        enable: bool,
        _update: Option<UpdateMode>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        if enable {
            self.focus = Some(crate::view::ViewId::Keyboard);
        } else {
            self.focus = None;
        }
        self.queue_partial_update(rq);
    }

    /// Stub: Toggle search bar
    pub fn toggle_search_bar(
        &mut self,
        enable: bool,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        if enable {
            self.focus = Some(crate::view::ViewId::SearchBar);
        } else {
            self.focus = None;
        }
        self.queue_partial_update(rq);
    }

    /// Stub: Toggle margin cropper
    pub fn toggle_margin_cropper(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.margin_cropper_visible = !self.margin_cropper_visible;
        self.queue_partial_update(rq);
    }

    /// Search for text in document
    pub fn search(&mut self, query: &str, hub: &Hub, rq: &mut RenderQueue, _context: &mut Context) {
        if query.is_empty() {
            self.queue_partial_update(rq);
            return;
        }

        // Use search_handler to manage search state
        self.search_handler.start_search(query.to_string(), self.search_direction);

        // Store search query for rendering highlights
        use rustc_hash::FxHashMap;
        use std::sync::atomic::AtomicBool;

        self.search = Some(Search {
            _query: query.to_string(),
            results: Vec::new(),
            index: 0,
            running: AtomicBool::new(false),
            _results_count: 0,
            highlights: FxHashMap::default(),
        });

        // Trigger actual search through hub event
        hub.send(Event::Search(query.to_string())).ok();

        // Update UI to show search is active
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Stub: Load pixmap
    pub fn load_pixmap(
        &mut self,
        page_index: usize,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        let mut doc = self._doc.lock().expect("doc lock poisoned");
        let (pixmap, _) = doc.pixmap(crate::document::Location::Exact(page_index), 1.0, 3).expect("failed to load pixmap");
        drop(doc);
        let resource = crate::view::reader::reader_impl::reader_core::Resource {
            pixmap,
            frame: self.rect,
            scale: 1.0,
        };
        self.cache.insert(page_index, resource);
        self.queue_partial_update(rq);
    }

    /// Navigate to specific page number
    pub fn handle_go_to_page_submit(
        &mut self,
        page: usize,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        // Validate page number
        let target_page = page.min(self.pages_count.saturating_sub(1));

        if target_page != self.current_page {
            self.go_to_page(target_page, true, hub, rq, context);
        } else {
            self.queue_partial_update(rq);
        }
    }

    /// Stub: Handle edit note submit
    pub fn handle_edit_note_submit(
        &mut self,
        note: &str,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &Context,
    ) {
        if let Some(ref target_annotation) = self._target_annotation {
            if let Some(ref mut reader_info) = self.info.reader {
                if let Some(ann) = reader_info.annotations.iter_mut()
                    .find(|a| a.selection == *target_annotation) {
                    ann.note = note.to_string();
                    ann.modified = Local::now().naive_local();
                }
            }
        }
        self.queue_partial_update(rq);
    }

    /// Handle search query submission
    pub fn handle_search_submit(
        &mut self,
        query: &str,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if query.is_empty() {
            self.queue_partial_update(rq);
            return;
        }

        // Initialize search if not already active
        if self.search.is_none() {
            self.search(query, hub, rq, context);
        }

        // Close search bar after submission
        self.toggle_search_bar(false, hub, rq, context);
    }

    /// Stub: Handle go to location
    pub fn handle_go_to_location(
        &mut self,
        location: &Location,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Location::Exact(page) = *location {
            if page != self.current_page && page < self.pages_count {
                self.go_to_page(page, true, hub, rq, context);
            } else {
                self.queue_partial_update(rq);
            }
        } else {
            self.queue_partial_update(rq);
        }
    }

    /// Stub: Handle close search bar
    pub fn handle_close_search_bar(
        &mut self,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        self.toggle_search_bar(false, hub, rq, context);
        self.focus = None;
        self.queue_partial_update(rq);
    }

    /// Stub: Handle close edit note
    pub fn handle_close_edit_note(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &Context,
    ) {
        self._target_annotation = None;
        self.focus = None;
        self.queue_partial_update(rq);
    }

    /// Show table of contents menu
    pub fn handle_show_table_of_contents(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        // Check if TOC is available
        if self.info.simple_toc.is_some() {
            // TOC menu would be shown here
            // Full implementation would create and display TOC menu
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        } else {
            // No TOC available, just update
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
        }
    }

    /// Stub: Handle adjust selection
    pub fn handle_adjust_selection(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &Context,
    ) {
        self.selection = None;
        self.queue_partial_update(rq);
    }

    /// Stub: Handle menu event
    pub fn handle_menu_event(
        &mut self,
        _evt: &Event,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &Context,
    ) -> bool {
        self.queue_partial_update(rq);
        true
    }

    /// Stub: Handle device event
    pub fn handle_device_event(
        &mut self,
        device_event: DeviceEvent,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &Context,
    ) {
        match device_event {
            DeviceEvent::Button { code, status, .. } => {
                if status == ButtonStatus::Pressed {
                    self.held_buttons.insert(code);
                } else {
                    self.held_buttons.remove(&code);
                }
            }
            _ => {}
        }
        self.queue_partial_update(rq);
    }

    /// Stub: Handle keyboard
    pub fn handle_keyboard(
        &mut self,
        _key: KeyKind,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.queue_partial_update(rq);
    }

    /// Handle show annotations - display annotation sidebar
    pub fn handle_show_annotations(
        &mut self,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        use crate::view::notification::Notification;

        // Check if annotations exist
        let annotation_count = self
            .info
            .reader
            .as_ref()
            .map(|r| r.annotations.len())
            .unwrap_or(0);

        if annotation_count > 0 {
            // Show notification with annotation count
            let msg = format!("{} annotations in document", annotation_count);
            let notif = Notification::new(msg, hub, rq, context);
            self.children.push(Box::new(notif) as Box<dyn View>);

            // Trigger UI update to show annotation sidebar
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        } else {
            // No annotations, show info
            let msg = "No annotations in this document".to_string();
            let notif = Notification::new(msg, hub, rq, context);
            self.children.push(Box::new(notif) as Box<dyn View>);
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
        }
    }

    /// Handle show bookmarks - display bookmark list
    pub fn handle_show_bookmarks(
        &mut self,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        use crate::view::notification::Notification;

        // Get bookmark count
        let bookmark_count = self
            .info
            .reader
            .as_ref()
            .map(|r| r.bookmarks.len())
            .unwrap_or(0);

        if bookmark_count > 0 {
            // Show notification with bookmark count
            let msg = format!("{} bookmarks in document", bookmark_count);
            let notif = Notification::new(msg, hub, rq, context);
            self.children.push(Box::new(notif) as Box<dyn View>);

            // Trigger UI update to show bookmark sidebar
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        } else {
            // No bookmarks, show info
            let msg = "No bookmarks in this document".to_string();
            let notif = Notification::new(msg, hub, rq, context);
            self.children.push(Box::new(notif) as Box<dyn View>);
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
        }
    }

    /// Handle search result - navigate to search result with highlighting
    pub fn handle_search_result(
        &mut self,
        result_index: usize,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        use crate::view::notification::Notification;

        // Check if search is active and results exist
        if let Some(ref search) = self.search {
            if result_index < search.results.len() {
                let location = &search.results[result_index];

                // Navigate to the result page
                if let crate::document::Location::Exact(page) = *location {
                    if page != self.current_page {
                        self.go_to_page(page, true, hub, rq, context);
                        return;
                    }
                }

                // Show notification of result position
                let msg = format!(
                    "Search result {} of {}",
                    result_index + 1,
                    search.results.len()
                );
                let notif = Notification::new(msg, hub, rq, context);
                self.children.push(Box::new(notif) as Box<dyn View>);
            }
        }

        // Update UI to show search result
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Handle end of search - finalize search and clear search state
    pub fn handle_end_of_search(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        use crate::view::notification::Notification;

        // Get final search statistics
        let (result_count, query) = self
            .search
            .as_ref()
            .map_or((0, String::new()), |s| (s.results.len(), s._query.clone()));

        // Show completion message
        let msg = if result_count > 0 {
            format!("Search complete: {} results for '{}'", result_count, query)
        } else if !query.is_empty() {
            format!("No results found for '{}'", query)
        } else {
            "Search ended".to_string()
        };

        let notif = Notification::new(msg, hub, rq, context);
        self.children.push(Box::new(notif) as Box<dyn View>);

        // Clear search state but keep highlights visible
        if let Some(ref mut search) = self.search {
            search
                .running
                .store(false, std::sync::atomic::Ordering::SeqCst);
        }

        // Update UI to clear search bar
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Handle highlight selection - process selected text for highlighting
    pub fn handle_highlight_selection(
        &mut self,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        use crate::view::notification::Notification;

        // Get current selection if any
        if let Some(ref selection) = self.selection {
            // Calculate selection rectangle
            let rect = crate::geom::Rectangle::new(selection.start, selection.end);

            // Store highlight for current page
            if let Some(ref mut search) = self.search {
                search
                    .highlights
                    .entry(self.current_page)
                    .or_insert_with(Vec::new)
                    .push(rect);
            }

            // Show confirmation
            let msg = "Selection highlighted".to_string();
            let notif = Notification::new(msg, hub, rq, context);
            self.children.push(Box::new(notif) as Box<dyn View>);
        }

        // Update UI to show highlights
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Handle add highlight - create new highlight at current location
    pub fn handle_add_highlight(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        use crate::view::notification::Notification;

        // Update document info with new annotation
        if let Some(ref mut reader_info) = self.info.reader {
            let annotation = Annotation {
                note: String::new(),
                text: String::new(),
                selection: [
                    TextLocation::Static(self.current_page, 0),
                    TextLocation::Static(self.current_page, 100),
                ],
                modified: Local::now().naive_local(),
            };
            reader_info.annotations.push(annotation);
        }

        // Show confirmation
        let msg = format!("Highlight added on page {}", self.current_page + 1);
        let notif = Notification::new(msg, hub, rq, context);
        self.children.push(Box::new(notif) as Box<dyn View>);

        // Update UI
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Handle delete highlight - remove highlight from current location
    pub fn handle_delete_highlight(
        &mut self,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        use crate::view::notification::Notification;

        // Remove highlight from current page if exists
        let mut removed = false;
        if let Some(ref mut search) = self.search {
            if search.highlights.remove(&self.current_page).is_some() {
                removed = true;
            }
        }

        // Show confirmation
        let msg = if removed {
            format!("Highlight removed from page {}", self.current_page + 1)
        } else {
            format!("No highlights on page {}", self.current_page + 1)
        };
        let notif = Notification::new(msg, hub, rq, context);
        self.children.push(Box::new(notif) as Box<dyn View>);

        // Update UI
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Stub: Handle shown
    pub fn handle_shown(&mut self, _hub: &Hub, rq: &mut RenderQueue, _context: &mut Context) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Full));
    }

    /// Stub: Handle open
    pub fn handle_open(
        &mut self,
        _file: &Box<crate::metadata::Info>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Full));
    }

    /// Stub: Handle back
    pub fn handle_back(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        if let Some(prev_page) = self.history.pop_back() {
            self.go_to_page(prev_page, true, hub, rq, context);
        } else {
            self.queue_partial_update(rq);
        }
    }

    /// Go to results page
    pub fn go_to_results_page(
        &mut self,
        index: usize,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        if let Some(ref mut search) = self.search {
            if index < search.results.len() {
                search.index = index;
            }
        }
        self.queue_partial_update(rq);
    }

    /// Go to results neighbor
    pub fn go_to_results_neighbor(
        &mut self,
        dir: CycleDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Some(ref search) = self.search {
            let new_index = match dir {
                CycleDir::Next => search.index.saturating_add(1),
                CycleDir::Previous => search.index.saturating_sub(1),
            };
            if new_index < search.results.len() {
                self.go_to_results_page(new_index, hub, rq, context);
            }
        }
        self.queue_partial_update(rq);
    }
}
