//! Home Toggle UI
//!
//! Handles modal/overlay toggle methods for the Home view:
//! - toggle_keyboard()
//! - toggle_address_bar()
//! - toggle_navigation_bar()
//! - toggle_search_bar()
//! - toggle_rename_document()
//! - toggle_go_to_page()
//! - toggle_sort_menu()
//! - toggle_book_menu()
//! - toggle_library_menu()
//! - book_index() (helper)

use crate::log_error;
use crate::view::home::home_utils;

use std::path::Path;

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::geom::{halves, Rectangle};
use crate::library::Library;
use crate::metadata::{SimpleStatus, SortMethod};
use crate::settings::{FirstColumn, LibraryMode, SecondColumn};
use crate::theme;
use crate::unit::scale_by_dpi;
use crate::view::filler::Filler;
use crate::view::home::AddressBar;
use crate::view::home::BookMenuData;
use crate::view::home::BottomBar;
use crate::view::home::NavigationBar;
use crate::view::home::Shelf;
use crate::view::home::TRASH_DIRNAME;
use crate::view::keyboard::Keyboard;
use crate::view::menu::{Menu, MenuKind};
use crate::view::menu_helpers::{toggle_menu_ctx, toggle_menu_item};
use crate::view::named_input::NamedInput;
use crate::view::search_bar::SearchBar;
use crate::view::{AppCmd, Event, Hub, RenderData, RenderQueue, View};
use crate::view::{EntryId, EntryKind, ViewId};
use crate::view::{BIG_BAR_HEIGHT, SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};

use super::Home;

impl Home {
    pub(crate) fn toggle_select_directory(
        &mut self,
        path: &Path,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if self.current_directory.starts_with(path) {
            if let Some(parent) = path.parent() {
                self.select_directory(parent, hub, rq, context);
            }
        } else {
            self.select_directory(path, hub, rq, context);
        }
    }

    pub(crate) fn toggle_keyboard(
        &mut self,
        enable: bool,
        update: bool,
        id: Option<ViewId>,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let dpi = CURRENT_DEVICE.dpi;
        let (small_height, big_height) = (
            scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32,
            scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32,
        );
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);
        let has_search_bar = self.children[self.shelf_index + 2].is::<SearchBar>();

        if let Some(index) = home_utils::find_child_index_by_type::<Keyboard>(&self.children) {
            if enable {
                return;
            }

            let y_min = self.child(self.shelf_index + 1).rect().min.y;
            let mut rect = *self.child(index).rect();
            rect.absorb(self.child(index - 1).rect());

            self.children.drain(index - 1..=index);

            let delta_y = rect.height() as i32;

            if has_search_bar {
                for i in self.shelf_index + 1..=self.shelf_index + 2 {
                    let shifted_rect = *self.child(i).rect() + pt!(0, delta_y);
                    self.child_mut(i).resize(shifted_rect, hub, rq, context);
                }
            }

            context.kb_rect = Rectangle::default();
            hub.send(Event::Focus(None)).ok();
            if update {
                let rect = rect![self.rect.min.x, y_min, self.rect.max.x, y_min + delta_y];
                rq.add(RenderData::expose(rect, UpdateMode::Gui));
            }
        } else {
            if !enable {
                return;
            }

            let Some(index) = home_utils::find_child_index_by_type::<BottomBar>(&self.children)
            else {
                return;
            };
            let index = index - 1;
            let mut kb_rect = rect![
                self.rect.min.x,
                self.rect.max.y - (small_height + 3 * big_height) as i32 + big_thickness,
                self.rect.max.x,
                self.rect.max.y - small_height - small_thickness
            ];

            let number = matches!(id, Some(ViewId::GoToPageInput));
            let keyboard = Keyboard::new(&mut kb_rect, number, context);
            self.children
                .insert(index, Box::new(keyboard) as Box<dyn View>);

            let separator = Filler::new(
                rect![
                    self.rect.min.x,
                    kb_rect.min.y - thickness,
                    self.rect.max.x,
                    kb_rect.min.y
                ],
                crate::color::foreground(theme::is_dark_mode()),
            );
            self.children
                .insert(index, Box::new(separator) as Box<dyn View>);

            let delta_y = kb_rect.height() as i32 + thickness;

            if has_search_bar {
                for i in self.shelf_index + 1..=self.shelf_index + 2 {
                    let shifted_rect = *self.child(i).rect() + pt!(0, -delta_y);
                    self.child_mut(i).resize(shifted_rect, hub, rq, context);
                }
            }
        }

        if update {
            if enable {
                if has_search_bar {
                    for i in self.shelf_index + 1..=self.shelf_index + 4 {
                        let update_mode = if (i - self.shelf_index) == 1 {
                            UpdateMode::Partial
                        } else {
                            UpdateMode::Gui
                        };
                        rq.add(RenderData::new(
                            self.child(i).id(),
                            *self.child(i).rect(),
                            update_mode,
                        ));
                    }
                } else {
                    for i in self.shelf_index + 1..=self.shelf_index + 2 {
                        rq.add(RenderData::new(
                            self.child(i).id(),
                            *self.child(i).rect(),
                            UpdateMode::Gui,
                        ));
                    }
                }
            } else if has_search_bar {
                for i in self.shelf_index + 1..=self.shelf_index + 2 {
                    rq.add(RenderData::new(
                        self.child(i).id(),
                        *self.child(i).rect(),
                        UpdateMode::Gui,
                    ));
                }
            }
        }
    }

    pub(crate) fn toggle_address_bar(
        &mut self,
        enable: Option<bool>,
        update: bool,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let dpi = CURRENT_DEVICE.dpi;
        let (small_height, big_height) = (
            scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32,
            scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32,
        );
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;

        if let Some(index) = home_utils::find_child_index_by_type::<AddressBar>(&self.children) {
            if let Some(true) = enable {
                return;
            }

            if let Some(ViewId::AddressBarInput) = self.focus {
                self.toggle_keyboard(
                    false,
                    false,
                    Some(ViewId::AddressBarInput),
                    hub,
                    rq,
                    context,
                );
            }

            // Remove the address bar and its separator.
            self.children.drain(index..=index + 1);
            self.shelf_index -= 2;
            context.settings.home.address_bar = false;

            // Move the navigation bar up.
            if context.settings.home.navigation_bar {
                if let Some(nav_bar) =
                    self.children[self.shelf_index - 2].downcast_mut::<NavigationBar>()
                {
                    nav_bar.shift(pt!(0, -small_height));
                }
            }

            // Move the separator above the shelf up.
            *self.children[self.shelf_index - 1].rect_mut() += pt!(0, -small_height);

            // Move the shelf's top edge up.
            self.children[self.shelf_index].rect_mut().min.y -= small_height;
        } else {
            if let Some(false) = enable {
                return;
            }

            let sp_rect = *self.child(1).rect() + pt!(0, small_height);

            let separator = Filler::new(sp_rect, crate::color::foreground(theme::is_dark_mode()));
            self.children
                .insert(2, Box::new(separator) as Box<dyn View>);

            let addr_bar = AddressBar::new(
                rect![
                    self.rect.min.x,
                    sp_rect.min.y - small_height + thickness,
                    self.rect.max.x,
                    sp_rect.min.y
                ],
                self.current_directory.to_string_lossy(),
                context,
            );
            self.children.insert(2, Box::new(addr_bar) as Box<dyn View>);

            self.shelf_index += 2;
            context.settings.home.address_bar = true;

            // Move the separator above the shelf down.
            *self.children[self.shelf_index - 1].rect_mut() += pt!(0, small_height);

            // Move the shelf's top edge down.
            self.children[self.shelf_index].rect_mut().min.y += small_height;

            if context.settings.home.navigation_bar {
                let rect = *self.children[self.shelf_index].rect();
                let y_shift = rect.height() as i32 - (big_height - thickness);
                if let Some(nav_bar) =
                    self.children[self.shelf_index - 2].downcast_mut::<NavigationBar>()
                {
                    // Move the navigation bar down.
                    nav_bar.shift(pt!(0, small_height));

                    // Shrink the nav bar.
                    if y_shift < 0 {
                        let y_shift = nav_bar.shrink(y_shift, &mut context.fonts);
                        self.children[self.shelf_index].rect_mut().min.y += y_shift;
                        *self.children[self.shelf_index - 1].rect_mut() += pt!(0, y_shift);
                    }
                }
            }
        }

        if update {
            for i in 2..self.shelf_index {
                rq.add(RenderData::new(
                    self.child(i).id(),
                    *self.child(i).rect(),
                    UpdateMode::Gui,
                ));
            }

            self.update_shelf(true, hub, rq, context);
            self.update_bottom_bar(rq, context);
        }
    }

    pub(crate) fn toggle_navigation_bar(
        &mut self,
        enable: Option<bool>,
        update: bool,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let dpi = CURRENT_DEVICE.dpi;
        let (small_height, big_height) = (
            scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32,
            scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32,
        );
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, _) = halves(thickness);

        if let Some(index) = home_utils::find_child_index_by_type::<NavigationBar>(&self.children) {
            if let Some(true) = enable {
                return;
            }

            let mut rect = *self.child(index).rect();
            rect.absorb(self.child(index + 1).rect());
            let delta_y = rect.height() as i32;

            // Remove the navigation bar and its separator.
            self.children.drain(index..=index + 1);
            self.shelf_index -= 2;
            context.settings.home.navigation_bar = false;

            // Move the shelf's top edge up.
            self.children[self.shelf_index].rect_mut().min.y -= delta_y;
        } else {
            if let Some(false) = enable {
                return;
            }

            let sep_index = if context.settings.home.address_bar {
                3
            } else {
                1
            };
            let sp_rect = *self.child(sep_index).rect() + pt!(0, small_height);

            let separator = Filler::new(sp_rect, crate::color::foreground(theme::is_dark_mode()));
            self.children
                .insert(sep_index + 1, Box::new(separator) as Box<dyn View>);

            let mut nav_bar = NavigationBar::new(
                rect![
                    self.rect.min.x,
                    sp_rect.min.y - small_height + thickness,
                    self.rect.max.x,
                    sp_rect.min.y
                ],
                self.rect.max.y - small_height - big_height - small_thickness,
                context.settings.home.max_levels,
            );
            let (_, dirs) = context.library.list(&self.current_directory, None, true);
            nav_bar.set_path(&self.current_directory, &dirs, rq, context);
            self.children
                .insert(sep_index + 1, Box::new(nav_bar) as Box<dyn View>);

            self.shelf_index += 2;
            context.settings.home.navigation_bar = true;

            home_utils::adjust_shelf_top_edge(&mut self.children, self.shelf_index - 2);
        }

        if update {
            for i in 2..self.shelf_index {
                rq.add(RenderData::new(
                    self.child(i).id(),
                    *self.child(i).rect(),
                    UpdateMode::Gui,
                ));
            }

            self.update_shelf(true, hub, rq, context);
            self.update_bottom_bar(rq, context);
        }
    }

    pub(crate) fn toggle_search_bar(
        &mut self,
        enable: Option<bool>,
        update: bool,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let dpi = CURRENT_DEVICE.dpi;
        let (small_height, big_height) = (
            scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32,
            scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32,
        );
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let delta_y = small_height;
        let search_visible: bool;
        let mut has_keyboard = false;

        if let Some(index) = home_utils::find_child_index_by_type::<SearchBar>(&self.children) {
            if let Some(true) = enable {
                return;
            }

            if let Some(ViewId::HomeSearchInput) = self.focus {
                self.toggle_keyboard(
                    false,
                    false,
                    Some(ViewId::HomeSearchInput),
                    hub,
                    rq,
                    context,
                );
            }

            // Remove the search bar and its separator.
            self.children.drain(index - 1..=index);

            // Move the shelf's bottom edge.
            self.children[self.shelf_index].rect_mut().max.y += delta_y;

            if context.settings.home.navigation_bar {
                if let Some(nav_bar) =
                    self.children[self.shelf_index - 2].downcast_mut::<NavigationBar>()
                {
                    nav_bar.vertical_limit += delta_y;
                }
            }

            self.query = None;
            search_visible = false;
        } else {
            if let Some(false) = enable {
                return;
            }

            let sp_rect = *self.child(self.shelf_index + 1).rect() - pt!(0, delta_y);
            let search_bar = SearchBar::new(
                rect![
                    self.rect.min.x,
                    sp_rect.max.y,
                    self.rect.max.x,
                    sp_rect.max.y + delta_y - thickness
                ],
                ViewId::HomeSearchInput,
                "Title, author, series",
                "",
                context,
            );
            self.children
                .insert(self.shelf_index + 1, Box::new(search_bar) as Box<dyn View>);

            let separator = Filler::new(sp_rect, crate::color::foreground(theme::is_dark_mode()));
            self.children
                .insert(self.shelf_index + 1, Box::new(separator) as Box<dyn View>);

            // Move the shelf's bottom edge.
            self.children[self.shelf_index].rect_mut().max.y -= delta_y;

            if context.settings.home.navigation_bar {
                let rect = *self.children[self.shelf_index].rect();
                let y_shift = rect.height() as i32 - (big_height - thickness);
                if let Some(nav_bar) =
                    self.children[self.shelf_index - 2].downcast_mut::<NavigationBar>()
                {
                    nav_bar.vertical_limit -= delta_y;

                    // Shrink the nav bar.
                    if y_shift < 0 {
                        let y_shift = nav_bar.shrink(y_shift, &mut context.fonts);
                        self.children[self.shelf_index].rect_mut().min.y += y_shift;
                        *self.children[self.shelf_index - 1].rect_mut() += pt!(0, y_shift);
                    }
                }
            }

            if self.query.is_none() {
                if home_utils::find_child_index_by_type::<Keyboard>(&self.children).is_none() {
                    self.toggle_keyboard(
                        true,
                        false,
                        Some(ViewId::HomeSearchInput),
                        hub,
                        rq,
                        context,
                    );
                    has_keyboard = true;
                }

                hub.send(Event::Focus(Some(ViewId::HomeSearchInput))).ok();
            }

            search_visible = true;
        }

        if update {
            if !search_visible {
                self.refresh_visibles(false, true, hub, rq, context);
            }

            self.update_top_bar(search_visible, rq);

            if search_visible {
                rq.add(RenderData::new(
                    self.child(self.shelf_index - 1).id(),
                    *self.child(self.shelf_index - 1).rect(),
                    UpdateMode::Partial,
                ));
                let mut rect = *self.child(self.shelf_index).rect();
                rect.max.y = self.child(self.shelf_index + 1).rect().min.y;
                // Render the part of the shelf that isn't covered.
                self.update_shelf(true, hub, &mut RenderQueue::new(), context);
                rq.add(RenderData::new(
                    self.child(self.shelf_index).id(),
                    rect,
                    UpdateMode::Partial,
                ));
                // Render the views on top of the shelf.
                rect.min.y = rect.max.y;
                let end_index = self.shelf_index + if has_keyboard { 4 } else { 2 };
                rect.max.y = self.child(end_index).rect().max.y;
                rq.add(RenderData::expose(rect, UpdateMode::Partial));
            } else {
                for i in self.shelf_index - 1..=self.shelf_index + 1 {
                    if i == self.shelf_index {
                        home_utils::update_shelf_and_bottom_bar(true, self, hub, rq, context);
                        continue;
                    }
                    rq.add(RenderData::new(
                        self.child(i).id(),
                        *self.child(i).rect(),
                        UpdateMode::Partial,
                    ));
                }
            }

            self.update_bottom_bar(rq, context);
        }
    }

    pub(crate) fn toggle_rename_document(
        &mut self,
        enable: Option<bool>,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Some(index) =
            home_utils::find_child_index_by_view_id(&self.children, ViewId::RenameDocument)
        {
            if let Some(true) = enable {
                return;
            }
            self.target_document = None;
            rq.add(RenderData::expose(
                *self.child(index).rect(),
                UpdateMode::Gui,
            ));
            self.children.remove(index);
            if let Some(ViewId::RenameDocumentInput) = self.focus {
                self.toggle_keyboard(
                    false,
                    true,
                    Some(ViewId::RenameDocumentInput),
                    hub,
                    rq,
                    context,
                );
            }
        } else {
            if let Some(false) = enable {
                return;
            }
            let mut ren_doc = NamedInput::new(
                "Rename document".to_string(),
                ViewId::RenameDocument,
                ViewId::RenameDocumentInput,
                21,
                context,
            );
            if let Some(text) = self
                .target_document
                .as_ref()
                .and_then(|path| path.file_name())
                .and_then(|file_name| file_name.to_str())
            {
                ren_doc.set_text(text, rq, context);
            }
            rq.add(RenderData::new(
                ren_doc.id(),
                *ren_doc.rect(),
                UpdateMode::Gui,
            ));
            hub.send(Event::Focus(Some(ViewId::RenameDocumentInput)))
                .ok();
            self.children.push(Box::new(ren_doc) as Box<dyn View>);
        }
    }

    pub(crate) fn toggle_go_to_page(
        &mut self,
        enable: Option<bool>,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Some(index) =
            home_utils::find_child_index_by_view_id(&self.children, ViewId::GoToPage)
        {
            if let Some(true) = enable {
                return;
            }
            rq.add(RenderData::expose(
                *self.child(index).rect(),
                UpdateMode::Gui,
            ));
            self.children.remove(index);
            if let Some(ViewId::GoToPageInput) = self.focus {
                self.toggle_keyboard(false, true, Some(ViewId::GoToPageInput), hub, rq, context);
            }
        } else {
            if let Some(false) = enable {
                return;
            }
            if self.pages_count < 2 {
                return;
            }
            let go_to_page = NamedInput::new(
                "Go to page".to_string(),
                ViewId::GoToPage,
                ViewId::GoToPageInput,
                4,
                context,
            );
            rq.add(RenderData::new(
                go_to_page.id(),
                *go_to_page.rect(),
                UpdateMode::Gui,
            ));
            hub.send(Event::Focus(Some(ViewId::GoToPageInput))).ok();
            self.children.push(Box::new(go_to_page) as Box<dyn View>);
        }
    }

    pub(crate) fn toggle_sort_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let sort_method = self.sort_method;
        let reverse_order = self.reverse_order;
        toggle_menu_ctx(
            ViewId::SortMenu,
            |ctx| {
                let entries = vec![
                    EntryKind::RadioButton(
                        "Date Opened".to_string(),
                        EntryId::Sort(SortMethod::Opened),
                        sort_method == SortMethod::Opened,
                    ),
                    EntryKind::RadioButton(
                        "Date Added".to_string(),
                        EntryId::Sort(SortMethod::Added),
                        sort_method == SortMethod::Added,
                    ),
                    EntryKind::RadioButton(
                        "Status".to_string(),
                        EntryId::Sort(SortMethod::Status),
                        sort_method == SortMethod::Status,
                    ),
                    EntryKind::RadioButton(
                        "Progress".to_string(),
                        EntryId::Sort(SortMethod::Progress),
                        sort_method == SortMethod::Progress,
                    ),
                    EntryKind::RadioButton(
                        "Author".to_string(),
                        EntryId::Sort(SortMethod::Author),
                        sort_method == SortMethod::Author,
                    ),
                    EntryKind::RadioButton(
                        "Title".to_string(),
                        EntryId::Sort(SortMethod::Title),
                        sort_method == SortMethod::Title,
                    ),
                    EntryKind::RadioButton(
                        "Year".to_string(),
                        EntryId::Sort(SortMethod::Year),
                        sort_method == SortMethod::Year,
                    ),
                    EntryKind::RadioButton(
                        "Series".to_string(),
                        EntryId::Sort(SortMethod::Series),
                        sort_method == SortMethod::Series,
                    ),
                    EntryKind::RadioButton(
                        "File Size".to_string(),
                        EntryId::Sort(SortMethod::Size),
                        sort_method == SortMethod::Size,
                    ),
                    EntryKind::RadioButton(
                        "File Type".to_string(),
                        EntryId::Sort(SortMethod::Kind),
                        sort_method == SortMethod::Kind,
                    ),
                    EntryKind::RadioButton(
                        "File Name".to_string(),
                        EntryId::Sort(SortMethod::FileName),
                        sort_method == SortMethod::FileName,
                    ),
                    EntryKind::RadioButton(
                        "File Path".to_string(),
                        EntryId::Sort(SortMethod::FilePath),
                        sort_method == SortMethod::FilePath,
                    ),
                    EntryKind::Separator,
                    EntryKind::CheckBox(
                        "Reverse Order".to_string(),
                        EntryId::ReverseOrder,
                        reverse_order,
                    ),
                ];
                Menu::new(rect, ViewId::SortMenu, MenuKind::DropDown, entries, ctx)
            },
            self,
            enable,
            rq,
            context,
        );
    }

    fn book_index(&self, index: usize) -> usize {
        let max_lines = self
            .child(self.shelf_index)
            .downcast_ref::<Shelf>()
            .map_or(1, |shelf| shelf.max_lines);
        let index_lower = self.current_page * max_lines;
        (index_lower + index).min(self.visible_books.len())
    }

    pub(crate) fn toggle_book_menu(
        &mut self,
        index: usize,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let book_index = self.book_index(index);
        let info = &self.visible_books[book_index];
        let path = info.file.path.clone();
        let kind = info.file.kind.clone();
        let author = info.author.clone();
        let simple_status = info.simple_status();
        let selected_library = context.settings.selected_library;
        let library_home = context.library.home.clone();
        let libraries = context
            .settings
            .libraries
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != selected_library)
            .map(|(i, lib)| (i, lib.name.clone()))
            .collect::<Vec<(usize, String)>>();

        toggle_menu_item(
            ViewId::BookMenu,
            |ctx, data: &BookMenuData| {
                let mut entries = Vec::new();

                if let Some(parent) = data.path.parent() {
                    entries.push(EntryKind::Command(
                        "Select Parent".to_string(),
                        EntryId::SelectDirectory(data.library_home.join(parent)),
                    ));
                }

                if !data.author.is_empty() {
                    entries.push(EntryKind::Command(
                        "Search Author".to_string(),
                        EntryId::SearchAuthor(data.author.clone()),
                    ));
                }

                if !entries.is_empty() {
                    entries.push(EntryKind::Separator);
                }

                let submenu: &[SimpleStatus] = match data.simple_status {
                    SimpleStatus::New => &[SimpleStatus::Reading, SimpleStatus::Finished],
                    SimpleStatus::Reading => &[SimpleStatus::New, SimpleStatus::Finished],
                    SimpleStatus::Finished => &[SimpleStatus::New, SimpleStatus::Reading],
                };

                let submenu = submenu
                    .iter()
                    .map(|s| {
                        EntryKind::Command(s.to_string(), EntryId::SetStatus(data.path.clone(), *s))
                    })
                    .collect();
                entries.push(EntryKind::SubMenu("Mark As".to_string(), submenu));
                entries.push(EntryKind::Separator);

                if !data.libraries.is_empty() {
                    let copy_to = data
                        .libraries
                        .iter()
                        .map(|(i, name)| {
                            EntryKind::Command(name.clone(), EntryId::CopyTo(data.path.clone(), *i))
                        })
                        .collect::<Vec<EntryKind>>();
                    let move_to = data
                        .libraries
                        .iter()
                        .map(|(i, name)| {
                            EntryKind::Command(name.clone(), EntryId::MoveTo(data.path.clone(), *i))
                        })
                        .collect::<Vec<EntryKind>>();
                    entries.push(EntryKind::SubMenu("Copy To".to_string(), copy_to));
                    entries.push(EntryKind::SubMenu("Move To".to_string(), move_to));
                }

                entries.push(EntryKind::Command(
                    "Rename".to_string(),
                    EntryId::Rename(data.path.clone()),
                ));
                entries.push(EntryKind::Command(
                    "Remove".to_string(),
                    EntryId::Remove(data.path.clone()),
                ));

                if data.kind == "epub" {
                    entries.push(EntryKind::Separator);
                    entries.push(EntryKind::Command(
                        "Cover Editor".to_string(),
                        EntryId::Launch(AppCmd::OpenCoverEditor(data.path.clone())),
                    ));
                } else if data.kind == "pdf" {
                    entries.push(EntryKind::Separator);
                    entries.push(EntryKind::Command(
                        "PDF Tools".to_string(),
                        EntryId::Launch(AppCmd::OpenPdfManipulator(data.path.clone())),
                    ));
                }

                Menu::new(rect, ViewId::BookMenu, MenuKind::Contextual, entries, ctx)
            },
            self,
            BookMenuData {
                path,
                kind,
                author,
                simple_status,
                libraries,
                library_home,
            },
            enable,
            rq,
            context,
        );
    }

    pub(crate) fn toggle_library_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let selected_library = context.settings.selected_library;
        let library_settings = context.settings.libraries[selected_library].clone();
        let show_hidden = context.library.show_hidden;
        let library_home = context.library.home.clone();
        let settings_libraries = context.settings.libraries.clone();
        let lib_index = selected_library;

        toggle_menu_ctx(
            ViewId::LibraryMenu,
            |ctx| {
                let libraries: Vec<EntryKind> = settings_libraries
                    .iter()
                    .enumerate()
                    .map(|(index, lib)| {
                        EntryKind::RadioButton(
                            lib.name.clone(),
                            EntryId::LoadLibrary(index),
                            index == lib_index,
                        )
                    })
                    .collect();

                let database = if library_settings.mode == LibraryMode::Database {
                    vec![
                        EntryKind::Command("Import".to_string(), EntryId::Import),
                        EntryKind::Command("Flush".to_string(), EntryId::Flush),
                    ]
                } else {
                    Vec::new()
                };

                let filesystem = if library_settings.mode == LibraryMode::Filesystem {
                    vec![
                        EntryKind::CheckBox(
                            "Show Hidden".to_string(),
                            EntryId::ToggleShowHidden,
                            show_hidden,
                        ),
                        EntryKind::Separator,
                        EntryKind::Command("Clean Up".to_string(), EntryId::CleanUp),
                        EntryKind::Command("Flush".to_string(), EntryId::Flush),
                        EntryKind::Command("Batch Select".to_string(), EntryId::ToggleBatchMode),
                    ]
                } else {
                    vec![EntryKind::Command(
                        "Batch Select".to_string(),
                        EntryId::ToggleBatchMode,
                    )]
                };

                let mut entries = vec![EntryKind::SubMenu("Library".to_string(), libraries)];

                if !database.is_empty() {
                    entries.push(EntryKind::SubMenu("Database".to_string(), database));
                }

                if !filesystem.is_empty() {
                    entries.push(EntryKind::SubMenu("Filesystem".to_string(), filesystem));
                }

                let hooks: Vec<EntryKind> = library_settings
                    .hooks
                    .iter()
                    .map(|v| {
                        EntryKind::Command(
                            v.path.to_string_lossy().into_owned(),
                            EntryId::ToggleSelectDirectory(library_home.join(&v.path)),
                        )
                    })
                    .collect();

                if !hooks.is_empty() {
                    entries.push(EntryKind::SubMenu("Toggle Select".to_string(), hooks));
                }

                entries.push(EntryKind::Separator);

                let first_column = library_settings.first_column;
                entries.push(EntryKind::SubMenu(
                    "First Column".to_string(),
                    vec![
                        EntryKind::RadioButton(
                            "Title and Author".to_string(),
                            EntryId::FirstColumn(FirstColumn::TitleAndAuthor),
                            first_column == FirstColumn::TitleAndAuthor,
                        ),
                        EntryKind::RadioButton(
                            "File Name".to_string(),
                            EntryId::FirstColumn(FirstColumn::FileName),
                            first_column == FirstColumn::FileName,
                        ),
                    ],
                ));

                let second_column = library_settings.second_column;
                entries.push(EntryKind::SubMenu(
                    "Second Column".to_string(),
                    vec![
                        EntryKind::RadioButton(
                            "Progress".to_string(),
                            EntryId::SecondColumn(SecondColumn::Progress),
                            second_column == SecondColumn::Progress,
                        ),
                        EntryKind::RadioButton(
                            "Year".to_string(),
                            EntryId::SecondColumn(SecondColumn::Year),
                            second_column == SecondColumn::Year,
                        ),
                    ],
                ));

                entries.push(EntryKind::CheckBox(
                    "Thumbnail Previews".to_string(),
                    EntryId::ThumbnailPreviews,
                    library_settings.thumbnail_previews,
                ));

                let trash_path = library_home.join(TRASH_DIRNAME);
                if let Ok(trash) = Library::new(trash_path, LibraryMode::Database)
                    .map_err(|e| log_error!("Can't inspect trash: {:#?}.", e))
                {
                    if trash.is_empty() == Some(false) {
                        entries.push(EntryKind::Separator);
                        entries.push(EntryKind::Command(
                            "Empty Trash".to_string(),
                            EntryId::EmptyTrash,
                        ));
                    }
                }

                Menu::new(rect, ViewId::LibraryMenu, MenuKind::DropDown, entries, ctx)
            },
            self,
            enable,
            rq,
            context,
        );
    }
}
