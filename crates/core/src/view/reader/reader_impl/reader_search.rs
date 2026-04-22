//! Reader Search Module
//!
//! This module provides search functionality for the Reader view,
//! including search result navigation and highlighting.

use crate::context::Context;
use crate::document::Location;
use crate::framebuffer::UpdateMode;
use crate::geom::{LinearDir, Rectangle};
use crate::view::menu::{Menu, MenuKind};
use crate::view::menu_helpers::toggle_menu_vec;
use crate::view::notification::Notification;
use crate::view::{EntryId, EntryKind, Id};
use crate::view::{Event, Hub, RenderData, RenderQueue, View, ViewId};
use rustc_hash::FxHashMap;
use std::sync::atomic::AtomicBool;

use super::reader::Reader;
use super::reader_core::{RenderChunk, Search};

/// Create search direction menu
///
/// Creates a menu for selecting search direction (Forward/Backward).
/// This menu is toggled by Reader::toggle_search_menu().
pub(crate) fn create_search_menu(
    search_direction: LinearDir,
    rect: Rectangle,
    context: &mut Context,
) -> Menu {
    use crate::view::ViewId;

    let entries = vec![
        EntryKind::RadioButton(
            "Forward".to_string(),
            EntryId::SearchDirection(LinearDir::Forward),
            search_direction == LinearDir::Forward,
        ),
        EntryKind::RadioButton(
            "Backward".to_string(),
            EntryId::SearchDirection(LinearDir::Backward),
            search_direction == LinearDir::Backward,
        ),
    ];

    Menu::new(
        rect,
        ViewId::SearchMenu,
        MenuKind::Contextual,
        entries,
        context,
    )
}

/// Render search result highlights on visible page chunks
pub(crate) fn render_results(
    search: Option<&Search>,
    chunks: &[RenderChunk],
    view_id: Id,
    rq: &mut RenderQueue,
) {
    if let Some(search) = search {
        for chunk in chunks {
            if let Some(groups) = search.highlights.get(&chunk.location) {
                for rect_ref in groups {
                    let rect = *rect_ref - chunk.frame.min + chunk.position;
                    rq.add(RenderData::new(view_id, rect, UpdateMode::Gui));
                }
            }
        }
    }
}

/// Navigate to the next or previous search result
pub(crate) fn go_to_results_neighbor(
    dir: crate::geom::CycleDir,
    reader: &mut Reader,
    hub: &crate::view::Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    if let Some(ref search) = reader.search {
        let n = search.results.len();
        if n == 0 {
            return;
        }
        let index = match dir {
            crate::geom::CycleDir::Next => (search.index + 1) % n,
            crate::geom::CycleDir::Previous => (search.index + n - 1) % n,
        };
        go_to_results_page(index, reader, hub, rq, context);
    }
}

/// Jump to a specific search result page
pub(crate) fn go_to_results_page(
    index: usize,
    reader: &mut Reader,
    hub: &crate::view::Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    if let Some(ref mut search) = reader.search {
        if index < search.results.len() {
            search.index = index;
            let location = search.results[index].clone();
            if let crate::document::Location::Exact(page) = location {
                reader.go_to_page(page, true, hub, rq, context);
            }
        }
    }
}

/// Toggle search menu visibility
pub(crate) fn toggle_search_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    search_direction: LinearDir,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    let create_menu = |ctx: &mut Context| create_search_menu(search_direction, rect, ctx);

    toggle_menu_vec(
        ViewId::SearchMenu,
        create_menu,
        children,
        enable,
        rq,
        context,
    );
}

impl Reader {
    /// Search for text in document
    pub fn search(&mut self, query: &str, hub: &Hub, rq: &mut RenderQueue, _context: &mut Context) {
        if query.is_empty() {
            self.queue_partial_update(rq);
            return;
        }

        self.search = Some(Search {
            _query: query.to_string(),
            results: Vec::with_capacity(32),
            index: 0,
            running: AtomicBool::new(false),
            _results_count: 0,
            highlights: FxHashMap::default(),
        });

        hub.send(Event::Search(query.to_string())).ok();
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
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

        if self.search.is_none() {
            self.search(query, hub, rq, context);
        }

        self.toggle_search_bar(false, hub, rq, context);
    }

    /// Handle close search bar
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

    /// Handle search result - navigate to search result with highlighting
    pub fn handle_search_result(
        &mut self,
        result_index: usize,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Some(ref search) = self.search {
            if result_index < search.results.len() {
                let location = &search.results[result_index];

                if let Location::Exact(page) = *location {
                    if page != self.current_page {
                        self.go_to_page(page, true, hub, rq, context);
                        return;
                    }
                }

                let msg = format!(
                    "Search result {} of {}",
                    result_index + 1,
                    search.results.len()
                );
                let notif = Notification::new(msg, hub, rq, context);
                self.children.push(Box::new(notif) as Box<dyn View>);
            }
        }

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Handle end of search - finalize search and clear search state
    pub fn handle_end_of_search(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let (result_count, query) = self
            .search
            .as_ref()
            .map_or((0, String::new()), |s| (s.results.len(), s._query.clone()));

        let msg = if result_count > 0 {
            format!("Search complete: {} results for '{}'", result_count, query)
        } else if !query.is_empty() {
            format!("No results found for '{}'", query)
        } else {
            "Search ended".to_string()
        };

        let notif = Notification::new(msg, hub, rq, context);
        self.children.push(Box::new(notif) as Box<dyn View>);

        if let Some(ref mut search) = self.search {
            search
                .running
                .store(false, std::sync::atomic::Ordering::SeqCst);
        }

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Handle highlight selection - process selected text for highlighting
    pub fn handle_highlight_selection(
        &mut self,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Some(ref selection) = self.selection {
            let rect = crate::geom::Rectangle::new(selection.start, selection.end);

            if let Some(ref mut search) = self.search {
                search
                    .highlights
                    .entry(self.current_page)
                    .or_insert_with(Vec::new)
                    .push(rect);
            }

            let msg = "Selection highlighted".to_string();
            let notif = Notification::new(msg, hub, rq, context);
            self.children.push(Box::new(notif) as Box<dyn View>);
        }

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Handle add highlight - create new highlight at current location
    pub fn handle_add_highlight(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        use crate::document::TextLocation;
        use crate::metadata::Annotation;
        use chrono::Local;

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

        let msg = format!("Highlight added on page {}", self.current_page + 1);
        let notif = Notification::new(msg, hub, rq, context);
        self.children.push(Box::new(notif) as Box<dyn View>);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Handle delete highlight - remove highlight from current location
    pub fn handle_delete_highlight(
        &mut self,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let mut removed = false;
        if let Some(ref mut search) = self.search {
            if search.highlights.remove(&self.current_page).is_some() {
                removed = true;
            }
        }

        let msg = if removed {
            format!("Highlight removed from page {}", self.current_page + 1)
        } else {
            format!("No highlights on page {}", self.current_page + 1)
        };
        let notif = Notification::new(msg, hub, rq, context);
        self.children.push(Box::new(notif) as Box<dyn View>);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }
}
