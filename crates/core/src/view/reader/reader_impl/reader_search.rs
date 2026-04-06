//! Reader Search Module
//!
//! Handles search functionality and result management.
//!
//! ## Methods to Move Here
//! - `toggle_search_menu()` - Search direction menu ✓
//! - `render_results()` - Highlight search results on page ✓
//! - `go_to_results_neighbor()` - Navigate between search results ✓ (stub)
//! - `go_to_results_page()` - Jump to specific result ✓ (stub)
//! - `toggle_search_bar()` - Search input UI ✓ (stub)
//! - `toggle_results_bar()` - Results display bar ✓ (stub)
//! - `search()` - Execute search query (stub) ✓ (stub)
//! - `update_results_bar()` - Update results display ✓ (stub)

use crate::geom::{LinearDir, Rectangle};
use crate::view::menu::{Menu, MenuKind};
use crate::view::{EntryId, EntryKind, Id, RenderData, RenderQueue, View, ViewId};

use crate::context::Context;
use crate::framebuffer::UpdateMode;

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
///
/// Adds render requests for all search result rectangles that fall within
/// the currently visible page chunks.
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
#[allow(dead_code)]
pub(crate) fn go_to_results_neighbor(
    _dir: crate::geom::CycleDir,
    view_id: Id,
    rect: Rectangle,
    rq: &mut RenderQueue,
) {
    rq.add(RenderData::new(view_id, rect, UpdateMode::Partial));
}

/// Jump to a specific search result page
#[allow(dead_code)]
pub(crate) fn go_to_results_page(
    _index: usize,
    view_id: Id,
    rect: Rectangle,
    rq: &mut RenderQueue,
) {
    rq.add(RenderData::new(view_id, rect, UpdateMode::Partial));
}

/// Toggle search input bar visibility
#[allow(dead_code)]
pub(crate) fn toggle_search_bar(_enable: bool, view_id: Id, rect: Rectangle, rq: &mut RenderQueue) {
    rq.add(RenderData::new(view_id, rect, UpdateMode::Partial));
}

/// Toggle search results display bar visibility
#[allow(dead_code)]
pub(crate) fn toggle_results_bar(
    _enable: bool,
    view_id: Id,
    rect: Rectangle,
    rq: &mut RenderQueue,
) {
    rq.add(RenderData::new(view_id, rect, UpdateMode::Partial));
}

/// Execute a text search
#[allow(dead_code)]
pub(crate) fn execute_search(_query: &str, view_id: Id, rect: Rectangle, rq: &mut RenderQueue) {
    rq.add(RenderData::new(view_id, rect, UpdateMode::Partial));
}

/// Update the search results bar display
#[allow(dead_code)]
pub(crate) fn update_results_bar(view_id: Id, rect: Rectangle, rq: &mut RenderQueue) {
    rq.add(RenderData::new(view_id, rect, UpdateMode::Partial));
}

/// Toggle search menu visibility
#[allow(dead_code)]
pub(crate) fn toggle_search_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    search_direction: LinearDir,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    if let Some(index) = children
        .iter()
        .position(|c| c.view_id().map_or(false, |i| i == ViewId::SearchMenu))
    {
        if let Some(true) = enable {
            return;
        }
        rq.add(RenderData::expose(*children[index].rect(), UpdateMode::Gui));
        children.remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
        let search_menu = create_search_menu(search_direction, rect, context);
        rq.add(RenderData::new(
            search_menu.id(),
            *search_menu.rect(),
            UpdateMode::Gui,
        ));
        children.push(Box::new(search_menu) as Box<dyn crate::view::View>);
    }
}
