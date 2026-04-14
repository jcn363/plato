//! Home Library View
//!
//! The Home view displays the library of documents and provides navigation,
//! search, sorting, and document metadata management.
//!
//! ## Module Structure
//!
//! - `home/mod.rs` (2,690 lines) - Main Home struct and implementation
//! - `home_utils.rs` (39 lines) - Utility functions for child view management
//! - `shelf.rs` - Document display shelf with thumbnail grid
//! - `book.rs` - Individual book/document entry
//! - `directory.rs` - Directory view for file browsing
//! - `address_bar.rs` - Path/address bar
//! - `navigation_bar.rs` - Library navigation controls
//! - `bottom_bar.rs` - Status and metadata bar
//! - `library_label.rs` - Library selection label
//! - `directories_bar.rs` - Directory list separator
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
mod library;
mod library_label;
mod navigation;
mod navigation_bar;
mod ops;
mod shelf;
mod ui_toggles;
mod updates;

use self::address_bar::AddressBar;
use self::bottom_bar::BottomBar;
use self::navigation_bar::NavigationBar;
use self::shelf::Shelf;
use super::top_bar::TopBar;

use crate::context::{Context, DeviceFlags};
use crate::device::CURRENT_DEVICE;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::{halves, CycleDir, DiagDir, Dir, Rectangle};
use crate::gesture::GestureEvent;
use crate::input::{ButtonCode, ButtonStatus, DeviceEvent};
use crate::library::Library;
use crate::log_error;
use crate::metadata::{sort, BookQuery, Metadata, SimpleStatus, SortMethod};
use crate::settings::{FirstColumn, Hook, LibraryMode, SecondColumn};
use crate::theme;
use crate::unit::scale_by_dpi;
use crate::view::common::{locate, rlocate};
use crate::view::common::{toggle_battery_menu, toggle_clock_menu, toggle_main_menu};
use crate::view::filler::Filler;
use crate::view::keyboard::Keyboard;
use crate::view::menu::{Menu, MenuKind};
use crate::view::menu_helpers::{toggle_menu_ctx, toggle_menu_item};
use crate::view::named_input::NamedInput;
use crate::view::notification::Notification;
use crate::view::search_bar::SearchBar;
use crate::view::{AppCmd, Bus, Event, Hub, RenderData, RenderQueue, View};
use crate::view::{EntryId, EntryKind, Id, ViewId, ID_FEEDER};
use crate::view::{BIG_BAR_HEIGHT, SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};
use anyhow::{format_err, Error};
use rand_core::Rng;
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::{json, Value};
use std::io::Write;
use std::mem;
use std::path::{Path, PathBuf};
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
        _bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Gesture(GestureEvent::Swipe {
                dir, start, end, ..
            }) => {
                match dir {
                    Dir::South
                        if self.children[0].rect().includes(start)
                            && self.children[self.shelf_index].rect().includes(end) =>
                    {
                        if !context.settings.home.navigation_bar {
                            self.toggle_navigation_bar(Some(true), true, hub, rq, context);
                        } else if !context.settings.home.address_bar {
                            self.toggle_address_bar(Some(true), true, hub, rq, context);
                        }
                    }
                    Dir::North
                        if self.children[self.shelf_index].rect().includes(start)
                            && self.children[0].rect().includes(end) =>
                    {
                        if context.settings.home.address_bar {
                            self.toggle_address_bar(Some(false), true, hub, rq, context);
                        } else if context.settings.home.navigation_bar {
                            self.toggle_navigation_bar(Some(false), true, hub, rq, context);
                        }
                    }
                    _ => (),
                }
                true
            }
            Event::Gesture(GestureEvent::Rotate { quarter_turns, .. }) if quarter_turns != 0 => {
                let (_, dir) = CURRENT_DEVICE.mirroring_scheme();
                let n = (4 + (context.display.rotation - dir * quarter_turns)) % 4;
                hub.send(Event::Select(EntryId::Rotate(n))).ok();
                true
            }
            Event::Gesture(GestureEvent::Arrow { dir, .. }) => {
                match dir {
                    Dir::West => self.go_to_page(0, hub, rq, context),
                    Dir::East => {
                        let pages_count = self.pages_count;
                        self.go_to_page(pages_count.saturating_sub(1), hub, rq, context);
                    }
                    Dir::North => {
                        let path = context.library.home.clone();
                        self.select_directory(&path, hub, rq, context);
                    }
                    Dir::South => self.toggle_search_bar(None, true, hub, rq, context),
                };
                true
            }
            Event::Gesture(GestureEvent::Corner { dir, .. }) => {
                match dir {
                    DiagDir::NorthWest | DiagDir::SouthWest => {
                        self.go_to_status_change(CycleDir::Previous, hub, rq, context)
                    }
                    DiagDir::NorthEast | DiagDir::SouthEast => {
                        self.go_to_status_change(CycleDir::Next, hub, rq, context)
                    }
                };
                true
            }
            Event::Focus(v) => {
                if self.focus != v {
                    self.focus = v;
                    if v.is_some() {
                        self.toggle_keyboard(true, true, v, hub, rq, context);
                    }
                }
                true
            }
            Event::Show(ViewId::Keyboard) => {
                self.toggle_keyboard(true, true, None, hub, rq, context);
                true
            }
            Event::Toggle(ViewId::GoToPage) => {
                self.toggle_go_to_page(None, hub, rq, context);
                true
            }
            Event::Toggle(ViewId::SearchBar) => {
                self.toggle_search_bar(None, true, hub, rq, context);
                true
            }
            Event::ToggleNear(ViewId::TitleMenu, rect) => {
                self.toggle_sort_menu(rect, None, rq, context);
                true
            }
            Event::ToggleBookMenu(rect, index) => {
                self.toggle_book_menu(index, rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::MainMenu, rect) => {
                toggle_main_menu(self, rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::BatteryMenu, rect) => {
                toggle_battery_menu(self, rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::ClockMenu, rect) => {
                toggle_clock_menu(self, rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::LibraryMenu, rect) => {
                self.toggle_library_menu(rect, None, rq, context);
                true
            }
            Event::Close(ViewId::AddressBar) => {
                self.toggle_address_bar(Some(false), true, hub, rq, context);
                true
            }
            Event::Close(ViewId::SearchBar) => {
                self.toggle_search_bar(Some(false), true, hub, rq, context);
                true
            }
            Event::Close(ViewId::SortMenu) => {
                self.toggle_sort_menu(Rectangle::default(), Some(false), rq, context);
                true
            }
            Event::Close(ViewId::LibraryMenu) => {
                self.toggle_library_menu(Rectangle::default(), Some(false), rq, context);
                true
            }
            Event::Close(ViewId::MainMenu) => {
                toggle_main_menu(self, Rectangle::default(), Some(false), rq, context);
                true
            }
            Event::Close(ViewId::GoToPage) => {
                self.toggle_go_to_page(Some(false), hub, rq, context);
                true
            }
            Event::Close(ViewId::RenameDocument) => {
                self.toggle_rename_document(Some(false), hub, rq, context);
                true
            }
            Event::Select(EntryId::Sort(sort_method)) => {
                let selected_library = context.settings.selected_library;
                context.settings.libraries[selected_library].sort_method = sort_method;
                self.set_sort_method(sort_method, hub, rq, context);
                true
            }
            Event::Select(EntryId::ReverseOrder) => {
                let next_value = !self.reverse_order;
                self.set_reverse_order(next_value, hub, rq, context);
                true
            }
            Event::Select(EntryId::LoadLibrary(index)) => {
                self.load_library(index, hub, rq, context);
                true
            }
            Event::Select(EntryId::Import) => {
                self.import(hub, rq, context);
                true
            }
            Event::Select(EntryId::CleanUp) => {
                self.clean_up(hub, rq, context);
                true
            }
            Event::Select(EntryId::Flush) => {
                self.flush(context);
                true
            }
            Event::ToggleBatchMode => {
                self.batch_mode = !self.batch_mode;
                if !self.batch_mode {
                    self.batch_selected.clear();
                }
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::BatchSelect(index) => {
                if self.batch_selected.contains(&index) {
                    self.batch_selected.remove(&index);
                } else {
                    self.batch_selected.insert(index);
                }
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::BatchDelete => {
                if !self.batch_selected.is_empty() {
                    let indices: Vec<usize> = self.batch_selected.iter().cloned().collect();
                    for &idx in &indices {
                        if idx < self.visible_books.len() {
                            let path = context
                                .library
                                .home
                                .join(&self.visible_books[idx].file.path);
                            if let Err(e) = std::fs::remove_file(&path) {
                                log_error!("Failed to delete {}: {}", path.display(), e);
                            }
                        }
                    }
                    self.batch_selected.clear();
                    self.batch_mode = false;
                    self.import(hub, rq, context);
                }
                true
            }
            Event::BatchMove(ref dest) => {
                if !self.batch_selected.is_empty() {
                    let indices: Vec<usize> = self.batch_selected.iter().cloned().collect();
                    for &idx in &indices {
                        if idx < self.visible_books.len() {
                            let src = context
                                .library
                                .home
                                .join(&self.visible_books[idx].file.path);
                            let dst = dest.join(&self.visible_books[idx].file.path);
                            if let Err(e) = std::fs::rename(&src, &dst) {
                                log_error!("Failed to move {}: {}", src.display(), e);
                            }
                        }
                    }
                    self.batch_selected.clear();
                    self.batch_mode = false;
                    self.import(hub, rq, context);
                }
                true
            }
            Event::FetcherAddDocument(_, ref info) => {
                self.add_document(*info.clone(), hub, rq, context);
                true
            }
            Event::Select(EntryId::SetStatus(ref path, status)) => {
                self.set_status(path, status, hub, rq, context);
                true
            }
            Event::Select(EntryId::FirstColumn(first_column)) => {
                let selected_library = context.settings.selected_library;
                context.settings.libraries[selected_library].first_column = first_column;
                self.update_first_column(hub, rq, context);
                true
            }
            Event::Select(EntryId::SecondColumn(second_column)) => {
                let selected_library = context.settings.selected_library;
                context.settings.libraries[selected_library].second_column = second_column;
                self.update_second_column(hub, rq, context);
                true
            }
            Event::Select(EntryId::ThumbnailPreviews) => {
                let selected_library = context.settings.selected_library;
                context.settings.libraries[selected_library].thumbnail_previews =
                    !context.settings.libraries[selected_library].thumbnail_previews;
                self.update_thumbnail_previews(hub, rq, context);
                true
            }
            Event::Submit(ViewId::AddressBarInput, ref addr) => {
                self.toggle_keyboard(false, true, None, hub, rq, context);
                self.select_directory(Path::new(addr), hub, rq, context);
                true
            }
            Event::Submit(ViewId::HomeSearchInput, ref text) => {
                self.query = BookQuery::new(text);
                if self.query.is_some() {
                    self.toggle_keyboard(false, false, None, hub, rq, context);
                    // Render the search bar and its separator.
                    for i in self.shelf_index + 1..=self.shelf_index + 2 {
                        rq.add(RenderData::new(
                            self.child(i).id(),
                            *self.child(i).rect(),
                            UpdateMode::Gui,
                        ));
                    }
                    self.refresh_visibles(true, true, hub, rq, context);
                } else {
                    let notif =
                        Notification::new("Invalid search query.".to_string(), hub, rq, context);
                    self.children.push(Box::new(notif) as Box<dyn View>);
                }
                true
            }
            Event::Submit(ViewId::GoToPageInput, ref text) => {
                if text == "(" {
                    self.go_to_page(0, hub, rq, context);
                } else if text == ")" {
                    self.go_to_page(self.pages_count.saturating_sub(1), hub, rq, context);
                } else if text == "_" {
                    let index = (context.rng.next_u64() % self.pages_count as u64) as usize;
                    self.go_to_page(index, hub, rq, context);
                } else if let Ok(index) = text.parse::<usize>() {
                    self.go_to_page(index.saturating_sub(1), hub, rq, context);
                }
                true
            }
            Event::Submit(ViewId::RenameDocumentInput, ref file_name) => {
                if let Some(ref path) = self.target_document.take() {
                    self.rename(path, file_name, hub, rq, context)
                        .map_err(|e| log_error!("Can't rename document: {:#}.", e))
                        .ok();
                }
                true
            }
            Event::NavigationBarResized(_) => {
                home_utils::adjust_shelf_top_edge(&mut self.children, self.shelf_index - 2);
                home_utils::update_shelf_and_bottom_bar(true, self, hub, rq, context);
                for i in self.shelf_index - 2..=self.shelf_index - 1 {
                    rq.add(RenderData::new(
                        self.child(i).id(),
                        *self.child(i).rect(),
                        UpdateMode::Gui,
                    ));
                }
                true
            }
            Event::Select(EntryId::EmptyTrash) => {
                self.empty_trash(hub, rq, context);
                true
            }
            Event::Select(EntryId::Rename(ref path)) => {
                self.target_document = Some(path.clone());
                self.toggle_rename_document(Some(true), hub, rq, context);
                true
            }
            Event::Select(EntryId::Remove(ref path))
            | Event::FetcherRemoveDocument(_, ref path) => {
                self.remove(path, hub, rq, context)
                    .map_err(|e| log_error!("Can't remove document: {:#}.", e))
                    .ok();
                true
            }
            Event::Select(EntryId::CopyTo(ref path, index)) => {
                self.copy_to(path, index, context)
                    .map_err(|e| log_error!("Can't copy document: {:#}.", e))
                    .ok();
                true
            }
            Event::Select(EntryId::MoveTo(ref path, index)) => {
                self.move_to(path, index, hub, rq, context)
                    .map_err(|e| log_error!("Can't move document: {:#}.", e))
                    .ok();
                true
            }
            Event::Select(EntryId::ToggleShowHidden) => {
                context.library.show_hidden = !context.library.show_hidden;
                self.refresh_visibles(true, false, hub, rq, context);
                true
            }
            Event::SelectDirectory(ref path)
            | Event::Select(EntryId::SelectDirectory(ref path)) => {
                self.select_directory(path, hub, rq, context);
                true
            }
            Event::Select(EntryId::ToggleSelectDirectory(ref path)) => {
                self.toggle_select_directory(path, hub, rq, context);
                true
            }
            Event::Select(EntryId::SearchAuthor(ref author)) => {
                let text = format!("'a {}", author);
                let query = BookQuery::new(&text);
                if query.is_some() {
                    self.query = query;
                    self.toggle_search_bar(Some(true), false, hub, rq, context);
                    self.toggle_keyboard(false, false, None, hub, rq, context);
                    if let Some(search_bar) =
                        self.children[self.shelf_index + 2].downcast_mut::<SearchBar>()
                    {
                        search_bar.set_text(&text, rq, context);
                    }
                    // Render the search bar and its separator.
                    for i in self.shelf_index + 1..=self.shelf_index + 2 {
                        rq.add(RenderData::new(
                            self.child(i).id(),
                            *self.child(i).rect(),
                            UpdateMode::Gui,
                        ));
                    }
                    self.refresh_visibles(true, true, hub, rq, context);
                }
                true
            }
            Event::GoTo(location) => {
                self.go_to_page(location as usize, hub, rq, context);
                true
            }
            Event::Chapter(dir) => {
                let pages_count = self.pages_count;
                match dir {
                    CycleDir::Previous => self.go_to_page(0, hub, rq, context),
                    CycleDir::Next => {
                        self.go_to_page(pages_count.saturating_sub(1), hub, rq, context)
                    }
                }
                true
            }
            Event::Page(dir) => {
                self.go_to_neighbor(dir, hub, rq, context);
                true
            }
            Event::Device(DeviceEvent::Button {
                code: ButtonCode::Backward,
                status: ButtonStatus::Pressed,
                ..
            }) => {
                self.go_to_neighbor(CycleDir::Previous, hub, rq, context);
                true
            }
            Event::Device(DeviceEvent::Button {
                code: ButtonCode::Forward,
                status: ButtonStatus::Pressed,
                ..
            }) => {
                self.go_to_neighbor(CycleDir::Next, hub, rq, context);
                true
            }
            Event::Device(DeviceEvent::NetUp) => {
                for fetcher in self.background_fetchers.values_mut() {
                    if let Some(stdin) = fetcher.process.stdin.as_mut() {
                        writeln!(stdin, "{}", json!({"type": "network", "status": "up"})).ok();
                    }
                }
                true
            }
            Event::FetcherSearch {
                id,
                ref path,
                ref query,
                ref sort_by,
            } => {
                let path = path.as_ref().unwrap_or(&context.library.home);
                let query = query.as_ref().and_then(|text| BookQuery::new(text));
                let (mut files, _) = context.library.list(path, query.as_ref(), false);
                if let Some((sort_method, reverse_order)) = *sort_by {
                    sort(&mut files, sort_method, reverse_order);
                }
                for entry in &mut files {
                    // Let the *reader* field pass through.
                    mem::swap(&mut entry.reader, &mut entry.reader_info);
                }
                if let Some(fetcher) = self.background_fetchers.get_mut(&id) {
                    if let Some(stdin) = fetcher.process.stdin.as_mut() {
                        writeln!(
                            stdin,
                            "{}",
                            json!({"type": "search",
                                                     "results": files})
                        )
                        .ok();
                    }
                }
                true
            }
            Event::CheckFetcher(id) => {
                if let Some(fetcher) = self.background_fetchers.get_mut(&id) {
                    if let Ok(exit_status) = fetcher.process.wait() {
                        if !exit_status.success() {
                            let msg = format!(
                                "{}: abnormal process termination.",
                                fetcher.path.display()
                            );
                            let notif = Notification::new(msg, hub, rq, context);
                            self.children.push(Box::new(notif) as Box<dyn View>);
                        }
                    }
                }
                true
            }
            Event::ToggleFrontlight => {
                if let Some(index) = locate::<TopBar>(self) {
                    if let Some(top_bar) = self.child_mut(index).downcast_mut::<TopBar>() {
                        top_bar.update_frontlight_icon(rq, context);
                    }
                }
                true
            }
            Event::Reseed => {
                self.reseed(hub, rq, context);
                true
            }
            _ => false,
        }
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
