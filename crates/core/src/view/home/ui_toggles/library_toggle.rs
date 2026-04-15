//! Library Toggle Module
//!
//! This module handles library menu visibility and interaction for the Home view.

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::view::menu::{Menu, MenuKind};
use crate::view::{Event, Hub, RenderData, RenderQueue, View};
use crate::view::{EntryId, ViewId};

use super::super::Home;

/// Library toggle configuration
#[derive(Debug, Clone)]
pub struct LibraryToggleConfig {
    pub auto_refresh: bool,
    pub show_statistics: bool,
    pub animation_duration: u32,
}

impl Default for LibraryToggleConfig {
    fn default() -> Self {
        Self {
            auto_refresh: true,
            show_statistics: false,
            animation_duration: 200,
        }
    }
}

/// Library toggle state
#[derive(Debug, Clone)]
pub struct LibraryToggleState {
    pub visible: bool,
    pub active: bool,
    pub config: LibraryToggleConfig,
}

impl Default for LibraryToggleState {
    fn default() -> Self {
        Self {
            visible: false,
            active: false,
            config: LibraryToggleConfig::default(),
        }
    }
}

impl Home {
    /// Toggle library menu visibility
    pub fn toggle_library_menu(
        &mut self,
        enable: Option<bool>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let should_enable = enable.unwrap_or(!self.library_menu.is_some());
        
        if should_enable {
            self.show_library_menu(rq, context);
        } else {
            self.hide_library_menu(rq, context);
        }
    }

    /// Show library menu
    fn show_library_menu(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.library_menu.is_some() {
            return;
        }

        let rect = self.calculate_library_menu_rect(context);
        let menu = self.create_library_menu(rect, context);
        
        self.library_menu = Some(Box::new(menu) as Box<dyn View>);
        self.focus = Some(ViewId::LibraryMenu);
        
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Hide library menu
    fn hide_library_menu(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if self.library_menu.is_none() {
            return;
        }

        self.library_menu = None;
        self.focus = None;
        
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    /// Calculate library menu rectangle
    fn calculate_library_menu_rect(&self, context: &Context) -> Rectangle {
        let screen_width = context.display.dims.0 as i32;
        let screen_height = context.display.dims.1 as i32;
        
        let width = (screen_width as f32 * 0.5) as i32;
        let height = (screen_height as f32 * 0.6) as i32;
        let x = (screen_width - width) / 2;
        let y = (screen_height - height) / 2;
        
        rect![x, y, width, height]
    }

    /// Create library menu
    fn create_library_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let mut menu = Menu::new(rect, context);
        
        // Add library options
        menu.add_entry(crate::view::menu_entry::MenuEntry::new(
            "Import Books".to_string(),
            self.id,
            Some("import_books".to_string()),
        ));
        
        menu.add_entry(crate::view::menu_entry::MenuEntry::new(
            "Library Statistics".to_string(),
            self.id,
            Some("library_statistics".to_string()),
        ));
        
        menu.add_separator();
        
        menu.add_entry(crate::view::menu_entry::MenuEntry::new(
            "Sort by Title".to_string(),
            self.id,
            Some("sort_by_title".to_string()),
        ));
        
        menu.add_entry(crate::view::menu_entry::MenuEntry::new(
            "Sort by Author".to_string(),
            self.id,
            Some("sort_by_author".to_string()),
        ));
        
        menu.add_entry(crate::view::menu_entry::MenuEntry::new(
            "Sort by Date".to_string(),
            self.id,
            Some("sort_by_date".to_string()),
        ));
        
        menu.add_separator();
        
        menu.add_entry(crate::view::menu_entry::MenuEntry::new(
            "Filter by Format".to_string(),
            self.id,
            Some("filter_by_format".to_string()),
        ));
        
        menu.add_entry(crate::view::menu_entry::MenuEntry::new(
            "Filter by Category".to_string(),
            self.id,
            Some("filter_by_category".to_string()),
        ));
        
        menu
    }

    /// Get library state
    fn get_library_state(&self) -> LibraryToggleState {
        LibraryToggleState {
            visible: self.library_menu.is_some(),
            active: self.focus == Some(ViewId::LibraryMenu),
            config: LibraryToggleConfig::default(),
        }
    }

    /// Update library configuration
    pub fn update_library_config(&mut self, config: LibraryToggleConfig) {
        // TODO: Implement library config update
        // This would require recreating the library menu if visible
    }

    /// Handle library menu events
    pub fn handle_library_menu_event(
        &mut self,
        event: &Event,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Close(ViewId::LibraryMenu) => {
                self.hide_library_menu(rq, context);
                true
            }
            Event::Select(name) => {
                self.handle_library_selection(name, hub, rq, context);
                true
            }
            _ => false,
        }
    }

    /// Handle library menu selection
    fn handle_library_selection(
        &mut self,
        name: &str,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        match name {
            "import_books" => {
                // TODO: Send appropriate import books event
                self.hide_library_menu(rq, context);
            }
            "library_statistics" => {
                // TODO: Send appropriate library statistics event
                self.hide_library_menu(rq, context);
            }
            "sort_by_title" => {
                // TODO: Send appropriate sort by title event
                self.hide_library_menu(rq, context);
            }
            "sort_by_author" => {
                // TODO: Send appropriate sort by author event
                self.hide_library_menu(rq, context);
            }
            "sort_by_date" => {
                // TODO: Send appropriate sort by date event
                self.hide_library_menu(rq, context);
            }
            "filter_by_format" => {
                // TODO: Send appropriate filter by format event
                self.hide_library_menu(rq, context);
            }
            "filter_by_category" => {
                // TODO: Send appropriate filter by category event
                self.hide_library_menu(rq, context);
            }
            _ => {}
        }
    }

    /// Check if library should auto-refresh
    pub fn should_auto_refresh_library(&self) -> bool {
        self.get_library_state().config.auto_refresh
    }

    /// Check if library should show statistics
    pub fn should_show_library_statistics(&self) -> bool {
        self.get_library_state().config.show_statistics
    }

    /// Get library animation duration
    pub fn get_library_animation_duration(&self) -> u32 {
        self.get_library_state().config.animation_duration
    }

    /// Update library statistics
    pub fn update_library_statistics(&mut self, rq: &mut RenderQueue) {
        // TODO: Update library statistics display
        if let Some(ref mut library_menu) = self.library_menu {
            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        }
    }
}

/// Utility functions for library toggles
pub mod utils {
    use super::*;

    /// Create default library toggle config
    pub fn create_default_library_config() -> LibraryToggleConfig {
        LibraryToggleConfig::default()
    }

    /// Get library sort options
    pub fn get_library_sort_options() -> Vec<LibrarySortOption> {
        vec![
            LibrarySortOption::Title,
            LibrarySortOption::Author,
            LibrarySortOption::Date,
            LibrarySortOption::Size,
            LibrarySortOption::Format,
        ]
    }

    /// Library sort options
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LibrarySortOption {
        Title,
        Author,
        Date,
        Size,
        Format,
    }

    /// Get library filter options
    pub fn get_library_filter_options() -> Vec<LibraryFilterOption> {
        vec![
            LibraryFilterOption::All,
            LibraryFilterOption::PDF,
            LibraryFilterOption::EPUB,
            LibraryFilterOption::TXT,
            LibraryFilterOption::Other,
        ]
    }

    /// Library filter options
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LibraryFilterOption {
        All,
        PDF,
        EPUB,
        TXT,
        Other,
    }

    /// Calculate library statistics
    pub fn calculate_library_statistics(context: &Context) -> LibraryStatistics {
        let total_books = context.library.len();
        let pdf_count = context.library.iter()
            .filter(|(_, info)| info.file.kind == "pdf")
            .count();
        let epub_count = context.library.iter()
            .filter(|(_, info)| info.file.kind == "epub")
            .count();
        let other_count = total_books - pdf_count - epub_count;
        
        let total_size = context.library.iter()
            .map(|(_, info)| info.file.size)
            .sum();
        
        LibraryStatistics {
            total_books,
            pdf_count,
            epub_count,
            other_count,
            total_size,
        }
    }

    /// Library statistics
    #[derive(Debug, Clone)]
    pub struct LibraryStatistics {
        pub total_books: usize,
        pub pdf_count: usize,
        pub epub_count: usize,
        pub other_count: usize,
        pub total_size: u64,
    }
}

// Library event types
#[derive(Debug, Clone)]
pub enum ImportBooks {}
#[derive(Debug, Clone)]
pub enum LibraryStatistics {}
#[derive(Debug, Clone)]
pub enum SortBy {
    Title,
    Author,
    Date,
}
#[derive(Debug, Clone)]
pub enum FilterByFormat {}
#[derive(Debug, Clone)]
pub enum FilterByCategory {}
