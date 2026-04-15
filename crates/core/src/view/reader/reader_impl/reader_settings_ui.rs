//! Reader Settings UI Module
//!
//! This module handles all settings menu creation and management for the Reader view,
//! including font settings, display options, and configuration dialogs.

use crate::color::Color;
use crate::geom::Rectangle;
use crate::metadata::{TextAlign, ZoomMode, ScrollMode};
use crate::view::{Hub, Id, RenderQueue};
use crate::view::menu::Menu;
use crate::view::menu_entry::MenuEntry;
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
        let mut menu = Menu::new(rect, context);
        
        menu.add_entry(MenuEntry::new("Font Settings", self.id, Some("font_settings")));
        menu.add_entry(MenuEntry::new("Display Settings", self.id, Some("display_settings")));
        menu.add_entry(MenuEntry::new("Navigation Settings", self.id, Some("navigation_settings")));
        menu.add_entry(MenuEntry::new("Annotation Settings", self.id, Some("annotation_settings")));
        menu.add_entry(MenuEntry::new("Search Settings", self.id, Some("search_settings")));
        
        menu
    }

    /// Create the font settings menu
    pub fn create_font_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let mut menu = Menu::new(rect, context);
        
        menu.add_entry(MenuEntry::new(
            format!("Font Size: {:.1}", self.font_size),
            self.id,
            Some("font_size"),
        ));
        menu.add_entry(MenuEntry::new(
            format!("Font Family: {}", self.font_family),
            self.id,
            Some("font_family"),
        ));
        menu.add_entry(MenuEntry::new(
            format!("Line Height: {:.1}", self.line_height),
            self.id,
            Some("line_height"),
        ));
        
        menu
    }

    /// Create the display settings menu
    pub fn create_display_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let mut menu = Menu::new(rect, context);
        
        menu.add_entry(MenuEntry::new(
            format!("Text Align: {:?}", self.text_align),
            self.id,
            Some("text_align"),
        ));
        menu.add_entry(MenuEntry::new(
            format!("Zoom Mode: {:?}", self.zoom_mode),
            self.id,
            Some("zoom_mode"),
        ));
        menu.add_entry(MenuEntry::new(
            format!("Scroll Mode: {:?}", self.scroll_mode),
            self.id,
            Some("scroll_mode"),
        ));
        menu.add_entry(MenuEntry::new(
            format!("Margin Width: {}", self.margin_width),
            self.id,
            Some("margin_width"),
        ));
        
        menu
    }

    /// Create the navigation settings menu
    pub fn create_navigation_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let mut menu = Menu::new(rect, context);
        
        menu.add_entry(MenuEntry::new("Page Turning Options", self.id, Some("page_turning")));
        menu.add_entry(MenuEntry::new("Gesture Settings", self.id, Some("gestures")));
        menu.add_entry(MenuEntry::new("Button Mapping", self.id, Some("buttons")));
        menu.add_entry(MenuEntry::new("History Settings", self.id, Some("history")));
        
        menu
    }

    /// Create the annotation settings menu
    pub fn create_annotation_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let mut menu = Menu::new(rect, context);
        
        menu.add_entry(MenuEntry::new("Highlight Color", self.id, Some("highlight_color")));
        menu.add_entry(MenuEntry::new("Note Settings", self.id, Some("note_settings")));
        menu.add_entry(MenuEntry::new("Bookmark Settings", self.id, Some("bookmark_settings")));
        menu.add_entry(MenuEntry::new("Export Options", self.id, Some("export_options")));
        
        menu
    }

    /// Create the search settings menu
    pub fn create_search_menu(&self, rect: Rectangle, context: &mut Context) -> Menu {
        let mut menu = Menu::new(rect, context);
        
        menu.add_entry(MenuEntry::new("Search Options", self.id, Some("search_options")));
        menu.add_entry(MenuEntry::new("Search History", self.id, Some("search_history")));
        menu.add_entry(MenuEntry::new("Search Filters", self.id, Some("search_filters")));
        
        menu
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
