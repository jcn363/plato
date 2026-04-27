//! Menu Toggle Module
//!
//! This module handles menu visibility and interaction for the Home view,
//! including sort menu, book menu, and other UI menus.

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::view::menu::{Menu, MenuKind};
use crate::view::{EntryId, EntryKind, Event, Hub, RenderData, RenderQueue, View, ViewId};

use super::super::Home;

/// Menu toggle configuration
#[derive(Debug, Clone)]
pub struct MenuToggleConfig {
    pub auto_hide: bool,
    pub animation_duration: u32,
    pub show_icons: bool,
}

impl Default for MenuToggleConfig {
    fn default() -> Self {
        Self {
            auto_hide: true,
            animation_duration: 200,
            show_icons: true,
        }
    }
}

/// Menu toggle state
#[derive(Debug, Clone, Default)]
pub struct MenuToggleState {
    pub _visible: bool,
    pub _active: bool,
    pub config: MenuToggleConfig,
}

impl Home {
    /// Toggle sort menu visibility
    pub fn toggle_sort_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(self.sort_menu.is_none());

        if should_enable {
            self.show_sort_menu(rect, rq, context);
        } else {
            self.hide_sort_menu(rq, context);
        }
    }

    /// Toggle search menu visibility
    pub fn toggle_search_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(self.search_menu.is_none());

        if should_enable {
            self.show_search_menu(rect, rq, context);
        } else {
            self.hide_search_menu(rq, context);
        }
    }

    /// Show sort menu
    fn show_sort_menu(&mut self, rect: Rectangle, rq: &mut RenderQueue, context: &mut Context) {
        if self.sort_menu.is_some() {
            return;
        }

        let menu = self.create_sort_menu(rect, context);

        self.sort_menu = Some(Box::new(menu) as Box<dyn View>);
        self.focus = Some(ViewId::SortMenu);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide sort menu
    fn hide_sort_menu(&mut self, rq: &mut RenderQueue, _context: &mut Context) {
        if self.sort_menu.is_none() {
            return;
        }

        self.sort_menu = None;
        self.focus = None;

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Show search menu
    fn show_search_menu(&mut self, rect: Rectangle, rq: &mut RenderQueue, context: &mut Context) {
        if self.search_menu.is_some() {
            return;
        }

        use super::super::search_menu::SearchMenu;
        let menu = SearchMenu::new(rect, context);

        self.search_menu = Some(Box::new(menu) as Box<dyn View>);
        self.focus = Some(ViewId::SearchMenu);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide search menu
    fn hide_search_menu(&mut self, rq: &mut RenderQueue, _context: &mut Context) {
        if self.search_menu.is_none() {
            return;
        }

        self.search_menu = None;
        self.focus = None;

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Create sort menu
    fn create_sort_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let reverse_text = if self.reverse_order { "↓" } else { "↑" };

        let entries = vec![
            EntryKind::Command(
                format!("Reverse Order {}", reverse_text),
                EntryId::ToggleReverseOrder,
            ),
            EntryKind::Separator,
            EntryKind::Command(
                "Recently Read".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::Opened),
            ),
            EntryKind::Command(
                "Recently Added".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::Added),
            ),
            EntryKind::Command(
                "A-Z Title".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::Title),
            ),
            EntryKind::Command(
                "A-Z Author".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::Author),
            ),
            EntryKind::Separator,
            EntryKind::Command(
                "Reading Progress".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::Progress),
            ),
            EntryKind::Command(
                "Status".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::Status),
            ),
            EntryKind::Separator,
            EntryKind::Command(
                "Year".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::Year),
            ),
            EntryKind::Command(
                "Series".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::Series),
            ),
            EntryKind::Command(
                "Pages".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::Pages),
            ),
            EntryKind::Command(
                "File Size".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::Size),
            ),
            EntryKind::Command(
                "File Type".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::Kind),
            ),
            EntryKind::Separator,
            EntryKind::Command(
                "File Name".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::FileName),
            ),
            EntryKind::Command(
                "File Path".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::FilePath),
            ),
            EntryKind::Separator,
            EntryKind::Command(
                "Manual Order".to_string(),
                EntryId::Sort(crate::metadata::SortMethod::Manual),
            ),
            EntryKind::Command("Edit Order".to_string(), EntryId::ToggleReorderMode),
        ];

        Menu::new(rect, ViewId::SortMenu, MenuKind::DropDown, entries, context)
    }

    /// Toggle book menu visibility
    pub fn toggle_book_menu(
        &mut self,
        index: usize,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(self.book_menu.is_none());

        if should_enable {
            self.show_book_menu(index, rect, rq, context);
        } else {
            self.hide_book_menu(rq, context);
        }
    }

    /// Show book menu
    fn show_book_menu(
        &mut self,
        index: usize,
        rect: Rectangle,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if self.book_menu.is_some() {
            return;
        }

        let menu = self.create_book_menu(index, rect, context);

        self.book_menu = Some(Box::new(menu) as Box<dyn View>);
        self.focus = Some(ViewId::BookMenu);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide book menu
    fn hide_book_menu(&mut self, rq: &mut RenderQueue, _context: &mut Context) {
        if self.book_menu.is_none() {
            return;
        }

        self.book_menu = None;
        self.focus = None;

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Create book menu
    fn create_book_menu(&self, _index: usize, rect: Rectangle, context: &mut Context) -> Menu {
        let entries = vec![
            EntryKind::Command("Open".to_string(), EntryId::Load(std::path::PathBuf::new())),
            EntryKind::Command(
                "Rename".to_string(),
                EntryId::Rename(std::path::PathBuf::new()),
            ),
            EntryKind::Command(
                "Delete".to_string(),
                EntryId::Remove(std::path::PathBuf::new()),
            ),
            EntryKind::Separator,
            EntryKind::Command("Add to Collection".to_string(), EntryId::AddToCollection),
            EntryKind::Command(
                "Remove from Collection".to_string(),
                EntryId::RemoveFromCollection,
            ),
            EntryKind::Separator,
            EntryKind::Command(
                "Add Bookmark".to_string(),
                EntryId::Load(std::path::PathBuf::new()),
            ),
            EntryKind::Command(
                "View Info".to_string(),
                EntryId::Load(std::path::PathBuf::new()),
            ),
        ];

        Menu::new(
            rect,
            ViewId::BookMenu,
            MenuKind::Contextual,
            entries,
            context,
        )
    }

    /// Get menu state
    fn get_menu_state(&self) -> MenuToggleState {
        MenuToggleState {
            _visible: self.sort_menu.is_some() || self.book_menu.is_some(),
            _active: self.focus == Some(ViewId::SortMenu) || self.focus == Some(ViewId::BookMenu),
            config: MenuToggleConfig::default(),
        }
    }

    /// Update menu configuration
    pub fn update_menu_config(
        &mut self,
        config: MenuToggleConfig,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        // Close and recreate visible menus to apply new config
        if self.sort_menu.is_some() {
            self.hide_sort_menu(rq, context);
            if config.auto_hide == self.should_auto_hide_menu() {
                // Config requires menu refresh
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
            }
        }

        if self.book_menu.is_some() {
            self.hide_book_menu(rq, context);
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }

    /// Handle menu events
    pub fn handle_menu_event(
        &mut self,
        event: &Event,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Close(ViewId::SortMenu) => {
                self.hide_sort_menu(rq, context);
                true
            }
            Event::Close(ViewId::SearchMenu) => {
                self.hide_search_menu(rq, context);
                true
            }
            Event::Close(ViewId::BookMenu) => {
                self.hide_book_menu(rq, context);
                true
            }
            Event::Select(name) => {
                self.handle_menu_selection(name, hub, rq, context);
                true
            }
            _ => false,
        }
    }

    /// Handle menu selection
    ///
    /// Note: Sort and document operations are handled by the main event loop in input.rs.
    /// This handler's role is to close menus and let events propagate for proper handling.
    fn handle_menu_selection(
        &mut self,
        entry_id: &EntryId,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        // Determine which menu is active and close it appropriately
        let is_sort_menu = self.sort_menu.is_some();
        let is_book_menu = self.book_menu.is_some();

        // Handle actions that need immediate processing
        match entry_id {
            EntryId::Sort(sort_method) => {
                // Apply sort method - delegates to set_sort_method via main event loop
                self.set_sort_method(*sort_method, hub, rq, context);
                self.hide_sort_menu(rq, context);
            }
            EntryId::Load(ref path) if !path.as_os_str().is_empty() => {
                // Book open action - event will be handled by main loop
                self.hide_book_menu(rq, context);
                // Send event for main handler to process
                hub.send(Event::Select(entry_id.clone())).ok();
            }
            EntryId::Rename(ref path) if !path.as_os_str().is_empty() => {
                // Book rename - handled by main event loop
                self.hide_book_menu(rq, context);
                hub.send(Event::Select(entry_id.clone())).ok();
            }
            EntryId::Remove(ref path) if !path.as_os_str().is_empty() => {
                // Book delete - handled by main event loop
                self.hide_book_menu(rq, context);
                hub.send(Event::Select(entry_id.clone())).ok();
            }
            _ => {
                // Close active menus
                if is_sort_menu {
                    self.hide_sort_menu(rq, context);
                }
                if is_book_menu {
                    self.hide_book_menu(rq, context);
                }
            }
        }
    }

    /// Check if menu should auto-hide
    pub fn should_auto_hide_menu(&self) -> bool {
        self.get_menu_state().config.auto_hide
    }

    /// Get menu animation duration
    pub fn get_menu_animation_duration(&self) -> u32 {
        self.get_menu_state().config.animation_duration
    }
}
