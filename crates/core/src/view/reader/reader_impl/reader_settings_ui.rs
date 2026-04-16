//! Reader Settings UI Module
//!
//! This module handles all settings menu creation and management for the Reader view,
//! including font settings, display options, and configuration dialogs.

use crate::geom::Rectangle;
use crate::metadata::{TextAlign, ZoomMode, ScrollMode};
use crate::view::{Hub, Id, RenderQueue, ViewId};
use crate::view::menu::{Menu, MenuKind};
use crate::view::entries::{EntryKind, EntryId};
use crate::context::Context;
use crate::settings::DEFAULT_FONT_FAMILY;

/// Settings menu types for the Reader
#[derive(Debug, Clone)]
pub enum ReaderSettingsMenu {
    FontSettings,
    DisplaySettings,
    NavigationSettings,
    AnnotationSettings,
    SearchSettings,
}

/// Settings UI manager for the Reader view
pub struct ReaderSettingsManager {
    pub id: Id,
    pub current_menu: Option<ReaderSettingsMenu>,
    pub font_size: f32,
    pub font_family: String,
    pub text_align: TextAlign,
    pub zoom_mode: ZoomMode,
    pub scroll_mode: ScrollMode,
    pub line_height: f32,
    pub margin_width: i32,
}

impl ReaderSettingsManager {
    /// Create a new settings manager
    pub fn new(id: Id) -> Self {
        Self {
            id,
            current_menu: None,
            font_size: 12.0,
            font_family: DEFAULT_FONT_FAMILY.to_string(),
            text_align: TextAlign::Left,
            zoom_mode: ZoomMode::Fit,
            scroll_mode: ScrollMode::Vertical,
            line_height: 1.2,
            margin_width: 8,
        }
    }

    /// Create the main settings menu
    pub fn create_main_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let entries = vec![
            EntryKind::Command("Font Settings".to_string(), EntryId::SystemInfo),
            EntryKind::Command("Display Settings".to_string(), EntryId::SystemInfo),
            EntryKind::Command("Navigation Settings".to_string(), EntryId::SystemInfo),
            EntryKind::Command("Annotation Settings".to_string(), EntryId::SystemInfo),
            EntryKind::Command("Search Settings".to_string(), EntryId::SystemInfo),
        ];
        
        Menu::new(rect, ViewId::SettingsMenu, MenuKind::DropDown, entries, context)
    }

    /// Create the font settings menu
    pub fn create_font_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let entries = vec![
            EntryKind::Command(format!("Font Size: {:.1}", self.font_size), EntryId::SystemInfo),
            EntryKind::Command(format!("Font Family: {}", self.font_family), EntryId::SystemInfo),
            EntryKind::Command(format!("Line Height: {:.1}", self.line_height), EntryId::SystemInfo),
        ];
        
        Menu::new(rect, ViewId::SettingsMenu, MenuKind::DropDown, entries, context)
    }

    /// Create the display settings menu
    pub fn create_display_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let entries = vec![
            EntryKind::Command(format!("Text Align: {:?}", self.text_align), EntryId::SystemInfo),
            EntryKind::Command(format!("Zoom Mode: {:?}", self.zoom_mode), EntryId::SystemInfo),
            EntryKind::Command(format!("Scroll Mode: {:?}", self.scroll_mode), EntryId::SystemInfo),
            EntryKind::Command(format!("Margin Width: {}", self.margin_width), EntryId::SystemInfo),
        ];
        
        Menu::new(rect, ViewId::SettingsMenu, MenuKind::DropDown, entries, context)
    }

    /// Create the navigation settings menu
    pub fn create_navigation_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let entries = vec![
            EntryKind::Command("Page Turning Options".to_string(), EntryId::SystemInfo),
            EntryKind::Command("Gesture Settings".to_string(), EntryId::SystemInfo),
            EntryKind::Command("Button Mapping".to_string(), EntryId::SystemInfo),
            EntryKind::Command("History Settings".to_string(), EntryId::SystemInfo),
        ];
        
        Menu::new(rect, ViewId::SettingsMenu, MenuKind::DropDown, entries, context)
    }

    /// Create the annotation settings menu
    pub fn create_annotation_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let entries = vec![
            EntryKind::Command("Highlight Color".to_string(), EntryId::SystemInfo),
            EntryKind::Command("Note Settings".to_string(), EntryId::SystemInfo),
            EntryKind::Command("Bookmark Settings".to_string(), EntryId::SystemInfo),
            EntryKind::Command("Export Options".to_string(), EntryId::SystemInfo),
        ];
        
        Menu::new(rect, ViewId::SettingsMenu, MenuKind::DropDown, entries, context)
    }

    /// Create the search settings menu
    pub fn create_search_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let entries = vec![
            EntryKind::Command("Search Options".to_string(), EntryId::SystemInfo),
            EntryKind::Command("Search History".to_string(), EntryId::SystemInfo),
            EntryKind::Command("Search Filters".to_string(), EntryId::SystemInfo),
        ];
        
        Menu::new(rect, ViewId::SettingsMenu, MenuKind::DropDown, entries, context)
    }

    /// Handle a settings menu selection
    pub fn handle_menu_selection(
        &mut self,
        selection: &str,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Option<ReaderSettingsMenu> {
        match selection {
            "font_settings" => Some(ReaderSettingsMenu::FontSettings),
            "display_settings" => Some(ReaderSettingsMenu::DisplaySettings),
            "navigation_settings" => Some(ReaderSettingsMenu::NavigationSettings),
            "annotation_settings" => Some(ReaderSettingsMenu::AnnotationSettings),
            "search_settings" => Some(ReaderSettingsMenu::SearchSettings),
            _ => None,
        }
    }

    /// Update font size
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size.clamp(8.0, 72.0);
    }

    /// Update font family
    pub fn set_font_family(&mut self, family: String) {
        self.font_family = family;
    }

    /// Update text alignment
    pub fn set_text_align(&mut self, align: TextAlign) {
        self.text_align = align;
    }

    /// Update zoom mode
    pub fn set_zoom_mode(&mut self, mode: ZoomMode) {
        self.zoom_mode = mode;
    }

    /// Update scroll mode
    pub fn set_scroll_mode(&mut self, mode: ScrollMode) {
        self.scroll_mode = mode;
    }

    /// Update line height
    pub fn set_line_height(&mut self, height: f32) {
        self.line_height = height.clamp(0.8, 3.0);
    }

    /// Update margin width
    pub fn set_margin_width(&mut self, width: i32) {
        self.margin_width = width.clamp(0, 100);
    }

    /// Get current settings as a tuple
    pub fn get_settings(&self) -> (f32, String, TextAlign, ZoomMode, ScrollMode, f32, i32) {
        (
            self.font_size,
            self.font_family.clone(),
            self.text_align,
            self.zoom_mode,
            self.scroll_mode,
            self.line_height,
            self.margin_width,
        )
    }
}
