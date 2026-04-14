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

pub use self::address_bar::AddressBar;
pub use self::book::Book;
pub use self::bottom_bar::BottomBar;
pub use self::directories_bar::DirectoriesBar;
pub use self::directory::Directory;
pub use self::library_label::LibraryLabel;
pub use self::navigation_bar::NavigationBar;
pub use self::shelf::Shelf;

use self::input::HomeInputExt;
use super::top_bar::TopBar;

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::{halves, Rectangle};
use crate::library::Library;
use crate::metadata::{sort, BookQuery, Metadata, SimpleStatus, SortMethod};
use crate::settings::{FirstColumn, Hook, LibraryMode, SecondColumn};
use crate::theme;
use crate::unit::scale_by_dpi;
use crate::view::common::{locate, rlocate, toggle_main_menu};
use crate::view::filler::Filler;
use crate::view::keyboard::Keyboard;
use crate::view::menu::Menu;
use crate::view::menu_helpers::toggle_menu_ctx;
use crate::view::named_input::NamedInput;
use crate::view::notification::Notification;
use crate::view::search_bar::SearchBar;
use crate::view::{Bus, Event, Hub, RenderData, RenderQueue, View};
use crate::view::{EntryId, Id, ViewId, ID_FEEDER};
use crate::view::{BIG_BAR_HEIGHT, SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};
use anyhow::{format_err, Error};
use rand_core::Rng;
use rustc_hash::{FxHashMap, FxHashSet};
use std::path::PathBuf;
use std::process::Child;

pub const TRASH_DIRNAME: &str = ".trash";

#[derive(Debug)]
pub struct Home {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    current_page: usize,
    pages_count: usize,
    shelf_index: usize,
    focus: Option<ViewId>,
    query: Option<BookQuery>,
    sort_method: SortMethod,
    reverse_order: bool,
    visible_books: Metadata,
    current_directory: PathBuf,
    target_document: Option<PathBuf>,
    background_fetchers: FxHashMap<u32, Fetcher>,
    batch_mode: bool,
    batch_selected: FxHashSet<usize>,
}

#[derive(Debug)]
struct Fetcher {
    path: PathBuf,
    full_path: PathBuf,
    process: Child,
    sort_method: Option<SortMethod>,
    first_column: Option<FirstColumn>,
    second_column: Option<SecondColumn>,
}

#[derive(Debug)]
pub struct BookMenuData {
    path: PathBuf,
    kind: String,
    author: String,
    simple_status: SimpleStatus,
    libraries: Vec<(usize, String)>,
    library_home: PathBuf,
}

impl Home {
    pub fn new(
        rect: Rectangle,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<Home, Error> {
        let id = ID_FEEDER.next();
        let dpi = CURRENT_DEVICE.dpi;
        let mut children = Vec::new();

        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);
        let (small_height, big_height) = (
            scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32,
            scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32,
        );

        let selected_library = context.settings.selected_library;
        let library_settings = &context.settings.libraries[selected_library];

        let current_directory = context.library.home.clone();
        let sort_method = library_settings.sort_method;
        let reverse_order = sort_method.reverse_order();

        context.library.sort(sort_method, reverse_order);

        let (visible_books, dirs) = context.library.list(&current_directory, None, false);
        let count = visible_books.len();
        let current_page = 0;
        let mut shelf_index = 2;

        let top_bar = TopBar::new(
            rect![
                rect.min.x,
                rect.min.y,
                rect.max.x,
                rect.min.y + small_height - small_thickness
            ],
            Event::Toggle(ViewId::SearchBar),
            sort_method.title(),
            context,
        );
        children.push(Box::new(top_bar) as Box<dyn View>);

        let separator = Filler::new(
            rect![
                rect.min.x,
                rect.min.y + small_height - small_thickness,
                rect.max.x,
                rect.min.y + small_height + big_thickness
            ],
            crate::color::foreground(theme::is_dark_mode()),
        );
        children.push(Box::new(separator) as Box<dyn View>);

        let mut y_start = rect.min.y + small_height + big_thickness;

        if context.settings.home.address_bar {
            let addr_bar = AddressBar::new(
                rect![
                    rect.min.x,
                    y_start,
                    rect.max.x,
                    y_start + small_height - thickness
                ],
                current_directory.to_string_lossy(),
                context,
            );
            children.push(Box::new(addr_bar) as Box<dyn View>);
            y_start += small_height - thickness;

            let separator = Filler::new(
                rect![rect.min.x, y_start, rect.max.x, y_start + thickness],
                crate::color::foreground(theme::is_dark_mode()),
            );
            children.push(Box::new(separator) as Box<dyn View>);
            y_start += thickness;
            shelf_index += 2;
        }

        if context.settings.home.navigation_bar {
            let mut nav_bar = NavigationBar::new(
                rect![
                    rect.min.x,
                    y_start,
                    rect.max.x,
                    y_start + small_height - thickness
                ],
                rect.max.y - small_height - big_height - small_thickness,
                context.settings.home.max_levels,
            );

            nav_bar.set_path(&current_directory, &dirs, &mut RenderQueue::new(), context);
            y_start = nav_bar.rect().max.y;

            children.push(Box::new(nav_bar) as Box<dyn View>);

            let separator = Filler::new(
                rect![rect.min.x, y_start, rect.max.x, y_start + thickness],
                crate::color::foreground(theme::is_dark_mode()),
            );
            children.push(Box::new(separator) as Box<dyn View>);
            y_start += thickness;
            shelf_index += 2;
        }

        let selected_library = context.settings.selected_library;
        let library_settings = &context.settings.libraries[selected_library];

        let mut shelf = Shelf::new(
            rect![
                rect.min.x,
                y_start,
                rect.max.x,
                rect.max.y - small_height - small_thickness
            ],
            library_settings.first_column,
            library_settings.second_column,
            library_settings.thumbnail_previews,
        );

        let max_lines = shelf.max_lines;
        let pages_count = (visible_books.len() as f32 / max_lines as f32).ceil() as usize;
        let index_lower = current_page * max_lines;
        let index_upper = (index_lower + max_lines).min(visible_books.len());

        shelf.update(
            &visible_books[index_lower..index_upper],
            hub,
            &mut RenderQueue::new(),
            context,
        );

        children.push(Box::new(shelf) as Box<dyn View>);

        let separator = Filler::new(
            rect![
                rect.min.x,
                rect.max.y - small_height - small_thickness,
                rect.max.x,
                rect.max.y - small_height + big_thickness
            ],
            crate::color::foreground(theme::is_dark_mode()),
        );
        children.push(Box::new(separator) as Box<dyn View>);

        let bottom_bar = BottomBar::new(
            rect![
                rect.min.x,
                rect.max.y - small_height + big_thickness,
                rect.max.x,
                rect.max.y
            ],
            current_page,
            pages_count,
            &library_settings.name,
            count,
            false,
        );
        children.push(Box::new(bottom_bar) as Box<dyn View>);

        rq.add(RenderData::new(id, rect, UpdateMode::Full));

        Ok(Home {
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
        })
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
        let dpi = CURRENT_DEVICE.dpi;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);
        let (small_height, big_height) = (
            scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32,
            scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32,
        );

        self.children.retain(|child| !child.is::<Menu>());

        // Top bar.
        let top_bar_rect = rect![
            rect.min.x,
            rect.min.y,
            rect.max.x,
            rect.min.y + small_height - small_thickness
        ];
        self.children[0].resize(top_bar_rect, hub, rq, context);

        let separator_rect = rect![
            rect.min.x,
            rect.min.y + small_height - small_thickness,
            rect.max.x,
            rect.min.y + small_height + big_thickness
        ];
        self.children[1].resize(separator_rect, hub, rq, context);

        let mut shelf_min_y = rect.min.y + small_height + big_thickness;
        let mut index = 2;

        // Address bar.
        if context.settings.home.address_bar {
            self.children[index].resize(
                rect![
                    rect.min.x,
                    shelf_min_y,
                    rect.max.x,
                    shelf_min_y + small_height - thickness
                ],
                hub,
                rq,
                context,
            );
            shelf_min_y += small_height - thickness;
            index += 1;

            self.children[index].resize(
                rect![rect.min.x, shelf_min_y, rect.max.x, shelf_min_y + thickness],
                hub,
                rq,
                context,
            );
            shelf_min_y += thickness;
            index += 1;
        }

        // Navigation bar.
        if context.settings.home.navigation_bar {
            let count = if self.children[self.shelf_index + 2].is::<SearchBar>() {
                2
            } else {
                1
            };
            let (_, dirs) = context.library.list(&self.current_directory, None, true);
            if let Some(nav_bar) = self.children[index]
                .as_mut()
                .downcast_mut::<NavigationBar>()
            {
                nav_bar.clear();
                nav_bar.resize(
                    rect![
                        rect.min.x,
                        shelf_min_y,
                        rect.max.x,
                        shelf_min_y + small_height - thickness
                    ],
                    hub,
                    rq,
                    context,
                );
                nav_bar.vertical_limit =
                    rect.max.y - count * small_height - big_height - small_thickness;
                nav_bar.set_path(
                    &self.current_directory,
                    &dirs,
                    &mut RenderQueue::new(),
                    context,
                );
                shelf_min_y += nav_bar.rect().height() as i32;
                index += 1;

                self.children[index].resize(
                    rect![rect.min.x, shelf_min_y, rect.max.x, shelf_min_y + thickness],
                    hub,
                    rq,
                    context,
                );
                shelf_min_y += thickness;
            }
        }

        // Bottom bar.
        let Some(bottom_bar_index) = rlocate::<BottomBar>(self) else {
            return;
        };
        index = bottom_bar_index;

        let separator_rect = rect![
            rect.min.x,
            rect.max.y - small_height - small_thickness,
            rect.max.x,
            rect.max.y - small_height + big_thickness
        ];
        self.children[index - 1].resize(separator_rect, hub, rq, context);

        let bottom_bar_rect = rect![
            rect.min.x,
            rect.max.y - small_height + big_thickness,
            rect.max.x,
            rect.max.y
        ];
        self.children[index].resize(bottom_bar_rect, hub, rq, context);

        let mut shelf_max_y = rect.max.y - small_height - small_thickness;

        if index - self.shelf_index > 2 {
            index -= 2;
            // Keyboard.
            if self.children[index].is::<Keyboard>() {
                let kb_rect = rect![
                    rect.min.x,
                    rect.max.y - (small_height + 3 * big_height) as i32 + big_thickness,
                    rect.max.x,
                    rect.max.y - small_height - small_thickness
                ];
                self.children[index].resize(kb_rect, hub, rq, context);
                let s_max_y = self.children[index].rect().min.y;
                self.children[index - 1].resize(
                    rect![rect.min.x, s_max_y - thickness, rect.max.x, s_max_y],
                    hub,
                    rq,
                    context,
                );
                index -= 2;
            }
            // Search bar.
            if self.children[index].is::<SearchBar>() {
                let sp_rect = *self.children[index + 1].rect() - pt!(0, small_height);
                self.children[index].resize(
                    rect![
                        rect.min.x,
                        sp_rect.max.y,
                        rect.max.x,
                        sp_rect.max.y + small_height - thickness
                    ],
                    hub,
                    rq,
                    context,
                );
                self.children[index - 1].resize(sp_rect, hub, rq, context);
                shelf_max_y -= small_height;
            }
        }

        // Shelf.
        let shelf_rect = rect![rect.min.x, shelf_min_y, rect.max.x, shelf_max_y];
        self.children[self.shelf_index].resize(shelf_rect, hub, rq, context);

        self.update_shelf(true, hub, &mut RenderQueue::new(), context);
        self.update_bottom_bar(&mut RenderQueue::new(), context);

        // Floating windows.
        for i in bottom_bar_index + 1..self.children.len() {
            self.children[i].resize(rect, hub, rq, context);
        }

        self.rect = rect;
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Full));
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
