//! Reader Search Module
//!
//! Handles search functionality and result management.
//!
//! ## Methods to Move Here
//! - `toggle_search_menu()` - Search direction menu ✓
//! - `render_results()` - Highlight search results on page (postponed - type conflicts)
//! - `go_to_results_neighbor()` - Navigate between search results
//! - `go_to_results_page()` - Jump to specific result
//! - `toggle_search_bar()` - Search input UI
//! - `toggle_results_bar()` - Results display bar
//! - `search()` - Execute search query (stub)
//! - `update_results_bar()` - Update results display
//!
//! ## Notes
//! Type duplication issue: RenderChunk exists in both reader.rs and reader_core.rs.
//! This needs architectural cleanup before extracting render-related functions.

use crate::framebuffer::UpdateMode;
use crate::geom::{LinearDir, Rectangle};
use crate::view::menu::{Menu, MenuKind};
use crate::view::{EntryId, EntryKind};

use crate::context::Context;

/// Toggle the search direction menu.
///
/// Creates or removes a contextual menu allowing the user to select
/// search direction (Forward/Backward).
///
/// This helper is called from Reader::toggle_search_menu() which provides
/// the children vector and other reader state.
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
