//! Reader Search Module
//!
//! This module provides search functionality for the Reader view,
//! including search result navigation and highlighting.
//!
//! ## Methods to Move Here
//! - `toggle_search_menu()` - Search direction menu
//! - `render_results()` - Highlight search results on page
//! - `go_to_results_neighbor()` - Navigate between search results (stub)
//! - `go_to_results_page()` - Jump to specific result (stub)

use crate::geom::{LinearDir, Rectangle};
use crate::view::menu::{Menu, MenuKind};
use crate::view::menu_helpers::toggle_menu_vec;
use crate::view::{EntryId, EntryKind, Id, RenderData, RenderQueue, ViewId};

use crate::context::Context;
use crate::framebuffer::UpdateMode;

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
