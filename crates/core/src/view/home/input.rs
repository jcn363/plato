//! Home Event Handling
//!
//! Handles event routing for the Home view:
//! - handle_event()

use std::io::Write;
use std::mem;
use std::path::Path;

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::geom::{CycleDir, DiagDir, Dir, Rectangle};
use crate::gesture::GestureEvent;
use crate::input::{ButtonCode, ButtonStatus, DeviceEvent};
use crate::log_error;
use crate::metadata::{sort, BookQuery};
use crate::view::common::locate;
use crate::view::common::{toggle_battery_menu, toggle_clock_menu, toggle_main_menu};
use crate::view::create_collection_dialog::CreateCollectionDialog;
use crate::view::notification::Notification;
use crate::view::search_bar::SearchBar;
use crate::view::top_bar::TopBar;
use crate::view::{Bus, EntryId, Event, Hub, RenderData, RenderQueue, View, ViewId};
use rand_core::Rng;

use super::Home;
use super::HomeCollectionsExt;

pub trait HomeInputExt {
    fn handle_event_impl(
        &mut self,
        evt: &Event,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool;
}

impl HomeInputExt for Home {
    fn handle_event_impl(
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
                            self.toggle_navigation_bar(Some(true), hub, rq, context);
                        } else if !context.settings.home.address_bar {
                            self.toggle_address_bar(Some(true), hub, rq, context);
                        }
                    }
                    Dir::North
                        if self.children[self.shelf_index].rect().includes(start)
                            && self.children[0].rect().includes(end) =>
                    {
                        if context.settings.home.address_bar {
                            self.toggle_address_bar(Some(false), hub, rq, context);
                        } else if context.settings.home.navigation_bar {
                            self.toggle_navigation_bar(Some(false), hub, rq, context);
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
                    Dir::South => self.toggle_search_bar(None, false, hub, rq, context),
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
                        self.toggle_keyboard(Some(true), hub, rq, context);
                    }
                }
                true
            }
            Event::Show(ViewId::Keyboard) => {
                self.toggle_keyboard(Some(true), hub, rq, context);
                true
            }
            Event::Show(ViewId::AboutDialog) => {
                use crate::view::about_dialog::AboutDialog;
                let dialog = AboutDialog::new(context);
                self.children.push(Box::new(dialog) as Box<dyn View>);
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::Show(ViewId::ShareDialog) => {
                use crate::view::share_dialog::ShareDialog;
                let dialog = ShareDialog::new(context, None);
                self.children.push(Box::new(dialog) as Box<dyn View>);
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::Show(ViewId::SystemInfo) => {
                use crate::view::system_info_dialog::SystemInfoDialog;
                let dialog = SystemInfoDialog::new(context);
                self.children.push(Box::new(dialog) as Box<dyn View>);
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::Show(ViewId::EmailDialog) => {
                use crate::view::email_dialog::EmailDialog;
                let dialog = EmailDialog::new(context, None);
                self.children.push(Box::new(dialog) as Box<dyn View>);
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::Show(ViewId::CloudDialog) => {
                use crate::view::cloud_dialog::CloudDialog;
                let dialog = CloudDialog::new(context, None);
                self.children.push(Box::new(dialog) as Box<dyn View>);
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::Show(ViewId::CreateCollectionDialog) => {
                let dialog = CreateCollectionDialog::new(context);
                self.children.push(Box::new(dialog) as Box<dyn View>);
                hub.send(Event::Focus(Some(ViewId::CreateCollectionDialog)))
                    .ok();
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::Close(ViewId::AboutDialog)
            | Event::Close(ViewId::ShareDialog)
            | Event::Close(ViewId::EmailDialog)
            | Event::Close(ViewId::CloudDialog)
            | Event::Close(ViewId::SystemInfo)
            | Event::Close(ViewId::CreateCollectionDialog) => {
                // Remove dialog from children
                self.children.retain(|child| {
                    let view_id = child.view_id();
                    view_id != Some(ViewId::AboutDialog)
                        && view_id != Some(ViewId::ShareDialog)
                        && view_id != Some(ViewId::EmailDialog)
                        && view_id != Some(ViewId::CloudDialog)
                        && view_id != Some(ViewId::SystemInfo)
                        && view_id != Some(ViewId::CreateCollectionDialog)
                });
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::Toggle(ViewId::GoToPage) => {
                self.toggle_go_to_page(None, hub, rq, context);
                true
            }
            Event::Toggle(ViewId::SearchBar) => {
                self.toggle_search_bar(None, false, hub, rq, context);
                true
            }
            Event::ToggleNear(ViewId::SearchMenu, rect) => {
                self.toggle_search_menu(rect, None, rq, context);
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
            Event::ToggleNear(ViewId::LibraryMenu, _rect) => {
                self.toggle_library_menu(None, hub, rq, context);
                true
            }
            Event::ToggleNear(ViewId::CollectionsMenu, _rect) => {
                self.toggle_collections_menu(None, hub, rq, context);
                true
            }
            Event::Close(ViewId::AddressBar) => {
                self.toggle_address_bar(Some(false), hub, rq, context);
                true
            }
            Event::Close(ViewId::SearchBar) => {
                self.toggle_address_bar(Some(false), hub, rq, context);
                true
            }
            Event::Close(ViewId::SearchMenu) => {
                self.toggle_search_menu(Rectangle::default(), Some(false), rq, context);
                true
            }
            Event::Close(ViewId::SortMenu) => {
                self.toggle_sort_menu(Rectangle::default(), Some(false), rq, context);
                true
            }
            Event::Close(ViewId::LibraryMenu) => {
                self.toggle_library_menu(Some(false), hub, rq, context);
                true
            }
            Event::Close(ViewId::CollectionsMenu) => {
                self.toggle_collections_menu(Some(false), hub, rq, context);
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
            Event::Select(EntryId::Sort(ref sort_method)) => {
                let selected_library = context.settings.selected_library;
                context.settings.libraries[selected_library].sort_method = *sort_method;
                self.set_sort_method(*sort_method, hub, rq, context);
                true
            }
            Event::Select(EntryId::ToggleReorderMode) => {
                self.reorder_mode = !self.reorder_mode;
                if self.reorder_mode {
                    context.settings.libraries[context.settings.selected_library].sort_method =
                        crate::metadata::SortMethod::Manual;
                    self.set_sort_method(crate::metadata::SortMethod::Manual, hub, rq, context);
                }
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::Select(EntryId::SetManualOrder(ref path, ref order)) => {
                self.set_manual_order(path, *order, hub, rq, context);
                true
            }
            Event::Select(EntryId::ToggleReverseOrder) => {
                let next_value = !self.reverse_order;
                self.set_reverse_order(next_value, hub, rq, context);
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
            Event::Select(EntryId::SearchTitle) => {
                // Open input field for title search
                self.toggle_keyboard(Some(true), hub, rq, context);
                true
            }
            Event::Select(EntryId::SearchSeries) => {
                // Open input field for series search
                self.toggle_keyboard(Some(true), hub, rq, context);
                true
            }
            Event::Select(EntryId::SearchPublisher) => {
                // Open input field for publisher search
                self.toggle_keyboard(Some(true), hub, rq, context);
                true
            }
            Event::Select(EntryId::SearchYear) => {
                // Open input field for year search
                self.toggle_keyboard(Some(true), hub, rq, context);
                true
            }
            Event::Select(EntryId::ToggleSearchReading) => {
                // Toggle reading status filter
                true
            }
            Event::Select(EntryId::ToggleSearchNew) => {
                // Toggle new status filter
                true
            }
            Event::Select(EntryId::ToggleSearchFinished) => {
                // Toggle finished status filter
                true
            }
            Event::Select(EntryId::ToggleSemanticSearch) => {
                // Toggle semantic results view
                let rect = *self.rect();
                if self.semantic_results.is_some() {
                    self.semantic_results = None;
                } else {
                    use crate::view::home::search_results::SearchResults;
                    // For now using dummy query, will hook to search bar later
                    if let Some(indexer) = context.library.indexer.as_ref() {
                        let res =
                            SearchResults::new(rect, context, "test query", indexer.indexer());
                        self.semantic_results = Some(Box::new(res));
                    }
                }
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::Select(EntryId::ClearSearchFilters) => {
                // Clear all search filters
                self.query = None;
                self.refresh_visibles(true, true, hub, rq, context);
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
            Event::Select(EntryId::AllBooks) => {
                self.handle_collections_menu_event(
                    &Event::Select(EntryId::AllBooks),
                    hub,
                    rq,
                    context,
                );
                true
            }
            Event::Select(EntryId::Collection(_)) => {
                self.handle_collections_menu_event(evt, hub, rq, context);
                true
            }
            Event::Select(EntryId::CreateCollection) => {
                // Check if this is from the dialog (get collection name from dialog)
                if let Some(dialog) = self
                    .children
                    .iter()
                    .find(|c| c.view_id() == Some(ViewId::CreateCollectionDialog))
                {
                    if let Some(dialog) = dialog.downcast_ref::<CreateCollectionDialog>() {
                        let name = dialog.get_collection_name().to_string();
                        if !name.trim().is_empty() {
                            self.create_collection(name, None, hub, rq, context);
                            hub.send(Event::Close(ViewId::CreateCollectionDialog)).ok();
                        }
                    }
                } else {
                    // From menu - show dialog
                    self.handle_collections_menu_event(evt, hub, rq, context);
                }
                true
            }
            Event::Select(EntryId::AddToCollection) => {
                // Show collections menu to select which collection to add to
                self.toggle_collections_menu(None, hub, rq, context);
                true
            }
            Event::Select(EntryId::RemoveFromCollection) => {
                // Remove from current collection
                if let Some(_collection_id) = &self.current_collection_id {
                    if let Some(_library) = &self.library {
                        // Get the selected book's fingerprint
                        // This would need to be tracked when the book menu is opened
                        // For now, just show a notification
                        hub.send(Event::Notify("Removed from collection".to_string()))
                            .ok();
                    }
                } else {
                    hub.send(Event::Notify("No collection selected".to_string()))
                        .ok();
                }
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
                self.toggle_keyboard(Some(false), hub, rq, context);
                self.select_directory(Path::new(addr), hub, rq, context);
                true
            }
            Event::Submit(ViewId::HomeSearchInput, ref text) => {
                if let Some(indexer) = context.library.indexer.as_ref() {
                    // Trigger semantic search results view
                    use crate::view::home::search_results::SearchResults;
                    let rect = *self.rect();
                    let res = SearchResults::new(rect, context, text, indexer.indexer());
                    self.semantic_results = Some(Box::new(res));
                    self.toggle_keyboard(Some(false), hub, rq, context);
                    rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                } else {
                    // Standard filter-based search
                    self.query = BookQuery::new(text);
                    if self.query.is_some() {
                        self.toggle_keyboard(Some(false), hub, rq, context);
                        for i in self.shelf_index + 1..=self.shelf_index + 2 {
                            rq.add(RenderData::new(
                                self.child(i).id(),
                                *self.child(i).rect(),
                                UpdateMode::Gui,
                            ));
                        }
                        self.refresh_visibles(true, true, hub, rq, context);
                    } else {
                        let notif = Notification::new(
                            "Invalid search query.".to_string(),
                            hub,
                            rq,
                            context,
                        );
                        self.children.push(Box::new(notif) as Box<dyn View>);
                    }
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
                crate::view::home::home_utils::adjust_shelf_top_edge(
                    &mut self.children,
                    self.shelf_index - 2,
                );
                crate::view::home::home_utils::update_shelf_and_bottom_bar(
                    true, self, hub, rq, context,
                );
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
                    self.toggle_keyboard(Some(false), hub, rq, context);
                    if let Some(search_bar) =
                        self.children[self.shelf_index + 2].downcast_mut::<SearchBar>()
                    {
                        search_bar.set_text(&text, rq, context);
                    }
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
                self.go_to_page(location, hub, rq, context);
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
                        writeln!(
                            stdin,
                            "{}",
                            serde_json::json!({"type": "network", "status": "up"})
                        )
                        .ok();
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
                    mem::swap(&mut entry.reader, &mut entry.reader_info);
                }
                if let Some(fetcher) = self.background_fetchers.get_mut(&id) {
                    if let Some(stdin) = fetcher.process.stdin.as_mut() {
                        writeln!(
                            stdin,
                            "{}",
                            serde_json::json!({"type": "search", "results": files})
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
            Event::Select(EntryId::ToggleBookView) => {
                self.toggle_book_view(None, hub, rq, context);
                true
            }
            Event::Select(EntryId::ToggleDirectoryView) => {
                self.toggle_directory_view(None, hub, rq, context);
                true
            }
            _ => false,
        }
    }
}
