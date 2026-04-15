//! Menu Toggle Module
//!
//! This module handles menu visibility and interaction for the Home view,
//! including sort menu, book menu, and other UI menus.

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::unit::scale_by_dpi;
use crate::view::menu::{Menu, MenuKind};
use crate::view::{Event, Hub, RenderData, RenderQueue, View};
use crate::view::{EntryId, ViewId};

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
#[derive(Debug, Clone)]
pub struct MenuToggleState {
    pub visible: bool,
    pub active: bool,
    pub config: MenuToggleConfig,
}

impl Default for MenuToggleState {
    fn default() -> Self {
        Self {
            visible: false,
            active: false,
            config: MenuToggleConfig::default(),
        }
    }
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
        let should_enable = enable.unwrap_or(!self.sort_menu.is_some());
        
        if should_enable {
            self.show_sort_menu(rect, rq, context);
        } else {
            self.hide_sort_menu(rq, context);
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
    fn hide_sort_menu(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.sort_menu.is_none() {
            return;
        }

        self.sort_menu = None;
        self.focus = None;
        
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Create sort menu
    fn create_sort_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let entries = vec![
            EntryKind::Command("Sort by Title".to_string(), EntryId::Sort(crate::metadata::SortMethod::Title)),
            EntryKind::Command("Sort by Author".to_string(), EntryId::Sort(crate::metadata::SortMethod::Author)),
            EntryKind::Command("Sort by Date".to_string(), EntryId::Sort(crate::metadata::SortMethod::Date)),
            EntryKind::Command("Sort by Size".to_string(), EntryId::Sort(crate::metadata::SortMethod::Size)),
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
        let should_enable = enable.unwrap_or(!self.book_menu.is_some());
        
        if should_enable {
            self.show_book_menu(index, rect, rq, context);
        } else {
            self.hide_book_menu(rq, context);
        }
    }

    /// Show book menu
    fn show_book_menu(&mut self, index: usize, rect: Rectangle, rq: &mut RenderQueue, context: &mut Context) {
        if self.book_menu.is_some() {
            return;
        }

        let menu = self.create_book_menu(index, rect, context);
        
        self.book_menu = Some(Box::new(menu) as Box<dyn View>);
        self.focus = Some(ViewId::BookMenu);
        
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide book menu
    fn hide_book_menu(&mut self, rq: &mut RenderQueue, context: &mut Context) {
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
            EntryKind::Command("Rename".to_string(), EntryId::Rename(std::path::PathBuf::new())),
            EntryKind::Command("Delete".to_string(), EntryId::Remove(std::path::PathBuf::new())),
            EntryKind::Separator,
            EntryKind::Command("Add Bookmark".to_string(), EntryId::Load(std::path::PathBuf::new())),
            EntryKind::Command("View Info".to_string(), EntryId::Load(std::path::PathBuf::new())),
        ];
        
        Menu::new(rect, ViewId::BookMenu, MenuKind::Contextual, entries, context)
    }

    /// Get menu state
    fn get_menu_state(&self) -> MenuToggleState {
        MenuToggleState {
            visible: self.sort_menu.is_some() || self.book_menu.is_some(),
            active: self.focus == Some(ViewId::SortMenu) || self.focus == Some(ViewId::BookMenu),
            config: MenuToggleConfig::default(),
        }
    }

    /// Update menu configuration
    pub fn update_menu_config(&mut self, config: MenuToggleConfig) {
        // TODO: Implement menu config update
        // This would require recreating the menus if visible
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
    fn handle_menu_selection(
        &mut self,
        name: &str,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        match name {
            "sort_title" => {
                // TODO: Implement sort by title
                self.hide_sort_menu(rq, context);
            }
            "sort_author" => {
                // TODO: Implement sort by author
                self.hide_sort_menu(rq, context);
            }
            "sort_date" => {
                // TODO: Implement sort by date
                self.hide_sort_menu(rq, context);
            }
            "sort_size" => {
                // TODO: Implement sort by size
                self.hide_sort_menu(rq, context);
            }
            "book_open" => {
                // TODO: Implement book open
                self.hide_book_menu(rq, context);
            }
            "book_rename" => {
                // TODO: Implement book rename
                self.hide_book_menu(rq, context);
            }
            "book_delete" => {
                // TODO: Implement book delete
                self.hide_book_menu(rq, context);
            }
            "book_bookmark" => {
                // TODO: Implement add bookmark
                self.hide_book_menu(rq, context);
            }
            "book_info" => {
                // TODO: Implement view info
                self.hide_book_menu(rq, context);
            }
            _ => {}
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

/// Utility functions for menu toggles
pub mod utils {
    use super::*;

    /// Create default menu toggle config
    pub fn create_default_menu_config() -> MenuToggleConfig {
        MenuToggleConfig::default()
    }

    /// Get available sort options
    pub fn get_sort_options() -> Vec<SortOption> {
        vec![
            SortOption::Title,
            SortOption::Author,
            SortOption::Date,
            SortOption::Size,
            SortOption::Format,
        ]
    }

    /// Sort options
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SortOption {
        Title,
        Author,
        Date,
        Size,
        Format,
    }

    /// Get available book actions
    pub fn get_book_actions() -> Vec<BookAction> {
        vec![
            BookAction::Open,
            BookAction::Rename,
            BookAction::Delete,
            BookAction::Bookmark,
            BookAction::Info,
            BookAction::Export,
        ]
    }

    /// Book actions
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BookAction {
        Open,
        Rename,
        Delete,
        Bookmark,
        Info,
        Export,
    }

    /// Calculate optimal menu size
    pub fn calculate_optimal_menu_size(screen_width: i32, screen_height: i32) -> (i32, i32) {
        let width = (screen_width as f32 * 0.4) as i32;
        let height = (screen_height as f32 * 0.6) as i32;
        (width, height)
    }

    /// Check if menu should be shown at position
    pub fn should_show_menu_at_position(
        position: (i32, i32),
        screen_size: (i32, i32),
        menu_size: (i32, i32),
    ) -> bool {
        let (x, y) = position;
        let (screen_w, screen_h) = screen_size;
        let (menu_w, menu_h) = menu_size;
        
        // Check if menu would fit on screen
        x + menu_w <= screen_w && y + menu_h <= screen_h
    }

    /// Adjust menu position to fit on screen
    pub fn adjust_menu_position(
        mut position: (i32, i32),
        screen_size: (i32, i32),
        menu_size: (i32, i32),
    ) -> (i32, i32) {
        let (x, y) = position;
        let (screen_w, screen_h) = screen_size;
        let (menu_w, menu_h) = menu_size;
        
        // Adjust x position
        let mut adjusted_x = x;
        if x + menu_w > screen_w {
            adjusted_x = screen_w - menu_w;
        }
        if adjusted_x < 0 {
            adjusted_x = 0;
        }
        
        // Adjust y position
        let mut adjusted_y = y;
        if y + menu_h > screen_h {
            adjusted_y = screen_h - menu_h;
        }
        if adjusted_y < 0 {
            adjusted_y = 0;
        }
        
        (adjusted_x, adjusted_y)
    }
}
