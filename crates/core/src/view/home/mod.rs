//! Home Library View
//!
//! The Home view displays the library of documents and provides navigation,
//! search, sorting, and document metadata management.
//!
//! ## Module Structure
//!
//! - `home/mod.rs` (596 lines) - Main Home struct and View trait impl
//! - `home/input.rs` (528 lines) - Event handling and routing
//! - `home/ui_toggles.rs` (1018 lines) - UI toggle methods
//! - `home/ops.rs` (217 lines) - Document operations
//! - `home/updates.rs` (191 lines) - UI state updates
//! - `home/fetcher.rs` (226 lines) - Background fetcher management
//! - `home/navigation.rs` (160 lines) - Directory/page navigation
//! - `home/library.rs` (117 lines) - Library operations
//! - `home_utils.rs` (41 lines) - Utility functions
//! - `shelf.rs`, `book.rs`, `directory.rs`, `address_bar.rs`, etc.
//!
//! ## Key Features
//!
//! ### Document Display
//! - Shelf-based grid layout with configurable columns
//! - Cover thumbnails with metadata overlay
//! - Scrollable view with pagination
//!
//! ### Navigation
//! - Directory browser with breadcrumb navigation
//! - Library switching between multiple document sources
//! - Search/filter capability
//!
//! ### Sorting
//! - Multiple sort methods (by title, author, date, etc.)
//! - Configurable sort order (ascending/descending)
//! - Persistent sort preferences
//!
//! ### Document Interaction
//! - Long-press for context menu
//! - Rename, move, delete operations
//! - Metadata editing
//! - Quick access to recent documents
//!
//! ## Known Limitations & TODOs
//!
//! ### Size and Complexity
//! The Home view at 2,690 lines handles many concerns:
//! - View hierarchy management (child views)
//! - Event routing to child views
//! - Library/document model management
//! - File system operations (list, rename, delete)
//! - Search and filter logic
//! - Thumbnail caching and rendering
//!
//! **TODO (Phase 5)**: Consider splitting into:
//! - `home_core.rs` - Data model and state management
//! - `home_library.rs` - Library operations
//! - `home_ui.rs` - UI layout and rendering
//! - `home_input.rs` - Event handling
//!
//! ### Performance Issues
//! - Thumbnail generation is synchronous (blocking UI)
//! - Large libraries (1000+ books) can be slow to scroll
//! - Search filtering is linear (O(n)) across all documents
//!
//! **Note**: These optimizations were evaluated and deferred due to device constraints.
//! Lazy loading and async operations add complexity that may outweigh benefits on limited RAM.
//!
//! ### Type Duplication
//! Fixed: ViewId-based helper function now correctly matches views by ViewId
//! instead of attempting to match by generic Id type.
//! See `home_utils::find_child_index_by_view_id()`.
//!
//! ## Testing
//!
//! Home view is challenging to test because:
//! 1. Heavy File system operations (directory listing, file I/O)
//! 2. Requires actual document library fixtures
//! 3. Complex event routing and state management
//!
//! **Current approach**: Integration tests with fixture directories.
//! Consider: Mocking Library and FileSystem interfaces.
//!
//! ## Future Improvements
//!
//! **Short Term** (10-15 hours):
//! - Async thumbnail generation (off-main thread)
//! - Indexed search (faster filtering)
//! - Better memory management for large libraries
//!
//! **Medium Term** (20-30 hours):
//! - Split into sub-modules for better maintainability
//! - Create ViewModel abstraction for Library operations
//! - Plugin support for custom book sources
//!
//! **Long Term** (40+ hours):
//! - Cloud library integration
//! - Advanced filtering and tagging system
//! - Reading statistics and recommendations

mod address_bar;
mod book;
mod bottom_bar;
mod directories_bar;
mod directory;
mod fetcher;
mod home_core;
mod home_ui;
mod home_utils;
mod input;
mod library;
mod library_label;
mod navigation;
mod navigation_bar;
mod ops;
mod shelf;
mod ui_toggles;
mod updates;

// Re-export types from home_core for public API
pub use self::home_core::{Home, Fetcher, BookMenuData, TRASH_DIRNAME};

pub use self::address_bar::AddressBar;
pub use self::book::Book;
pub use self::bottom_bar::BottomBar;
pub use self::directories_bar::DirectoriesBar;
pub use self::directory::Directory;
pub use self::library_label::LibraryLabel;
pub use self::navigation_bar::NavigationBar;
pub use self::shelf::Shelf;

use self::input::HomeInputExt;

use crate::context::Context;
use crate::font::Fonts;
use crate::framebuffer::Framebuffer;
use crate::geom::Rectangle;
use crate::view::menu::Menu;
use crate::view::{Bus, Event, Hub, RenderData, RenderQueue, View};
use crate::view::{Id, ID_FEEDER};
use crate::framebuffer::UpdateMode;
use anyhow::Error;
use rustc_hash::{FxHashMap, FxHashSet};
use std::path::PathBuf;

// Note: Home, Fetcher, BookMenuData, and TRASH_DIRNAME are defined in home_core.rs
// and re-exported above for public API compatibility

impl Home {
    pub fn new(
        rect: Rectangle,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<Home, Error> {
        let id = ID_FEEDER.next();
        let (_dpi, thickness, small_thickness, big_thickness, small_height, _big_height) =
            Self::calculate_dimensions();
        let (_selected_library, sort_method, reverse_order) =
            Self::get_library_settings(context);

        let current_directory = context.library.home.clone();
        context.library.sort(sort_method, reverse_order);
        let (visible_books, _dirs) = context.library.list(&current_directory, None, false);
        let count = visible_books.len();
        let current_page = 0;

        let mut children = Vec::new();
        let mut y_start = rect.min.y + small_height + big_thickness;
        let mut shelf_index = 2;

        Self::add_top_bar(
            &mut children,
            rect,
            small_height,
            small_thickness,
            big_thickness,
            sort_method,
            context,
        );
        y_start = Self::add_address_bar_if_enabled(
            &mut children,
            context,
            rect,
            y_start,
            thickness,
            small_height,
            small_thickness,
            &current_directory,
            shelf_index,
        );
        y_start = Self::add_navigation_bar_if_enabled(
            &mut children,
            context,
            rect,
            y_start,
            thickness,
            small_height,
            small_thickness,
            &current_directory,
        );
        // Calculate y_max for shelf (leaving room for bottom bar)
        let y_max = rect.max.y - small_height - thickness;
        // Calculate pages count (using default shelf capacity of 10)
        let shelf_capacity = 10;
        let pages_count = (count as f32 / shelf_capacity as f32).ceil() as usize;
        shelf_index = Self::add_shelf_and_bottom_bar(
            &mut children,
            hub,
            context,
            rect,
            y_start,
            y_max,
            current_page,
            pages_count,
        );

        rq.add(RenderData::new(id, rect, UpdateMode::Full));

        Ok(Self::create_home(
            id,
            rect,
            children,
            current_page,
            pages_count,
            shelf_index,
            sort_method,
            reverse_order,
            visible_books,
            current_directory,
        ))
    }

    fn create_home(
        id: Id,
        rect: Rectangle,
        children: Vec<Box<dyn View>>,
        current_page: usize,
        pages_count: usize,
        shelf_index: usize,
        sort_method: crate::metadata::SortMethod,
        reverse_order: bool,
        visible_books: Vec<crate::metadata::Info>,
        current_directory: PathBuf,
    ) -> Home {
        Home {
            id,
            rect,
            children,
            current_page,
            pages_count,
            shelf_index,
            focus: None,
            query: None,
            sort_method,
            reverse_order,
            visible_books,
            current_directory,
            target_document: None,
            background_fetchers: FxHashMap::default(),
            batch_mode: false,
            batch_selected: FxHashSet::default(),
            keyboard: None,
            address_bar: None,
            navigation_bar: None,
            search_bar: None,
            go_to_page: None,
            sort_menu: None,
            book_menu: None,
            library_menu: None,
            settings_menu: None,
            shelf: None,
            book_view: None,
            directory_view: None,
            bottom_bar: None,
        }
    }
}

impl View for Home {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        self.handle_event_impl(evt, hub, bus, rq, context)
    }

    fn render(&self, _fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {}

    fn resize(&mut self, rect: Rectangle, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let (thickness, small_thickness, big_thickness, small_height, big_height) =
            Self::calculate_resize_dimensions();

        self.children.retain(|child| !child.is::<Menu>());

        let mut shelf_min_y = rect.min.y + small_height + big_thickness;
        let index = 2;

        Self::resize_top_bar(
            &mut self.children,
            rect,
            small_height,
            small_thickness,
            big_thickness,
            hub,
            rq,
            context,
        );
        shelf_min_y = Self::resize_address_bar_if_enabled(
            &mut self.children,
            context,
            rect,
            shelf_min_y,
            thickness,
            small_height,
            index,
            hub,
            rq,
        );
        shelf_min_y =
            Self::get_address_bar_end_y(context, rect, shelf_min_y, small_height, thickness);
        let (_idx, new_shelf_min_y) = Self::resize_navigation_bar_if_enabled(
            &mut self.children,
            context,
            rect,
            shelf_min_y,
            thickness,
            small_height,
            small_thickness,
            hub,
            rq,
        );
        shelf_min_y = new_shelf_min_y;
        let bottom_bar_index = Self::resize_bottom_bar(
            &mut self.children,
            rect,
            small_height,
            thickness,
            hub,
            rq,
            context,
        );
        let shelf_max_y = Self::resize_keyboard_and_search_bar(
            &mut self.children,
            rect,
            bottom_bar_index,
            small_height,
            big_height,
            thickness,
            hub,
            rq,
            context,
        );
        Self::resize_shelf(
            &mut self.children,
            rect,
            shelf_min_y,
            shelf_max_y,
            self.shelf_index,
            hub,
            rq,
            context,
        );
        self.update_shelf(true, hub, &mut RenderQueue::new(), context);
        self.update_bottom_bar(&mut RenderQueue::new(), context);
        Self::resize_floating_windows(&mut self.children, rect, bottom_bar_index, hub, rq, context);

        self.rect = rect;
    }

    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn rect_mut(&mut self) -> &mut Rectangle {
        &mut self.rect
    }

    fn children(&self) -> &Vec<Box<dyn View>> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn View>> {
        &mut self.children
    }

    fn id(&self) -> Id {
        self.id
    }
}
