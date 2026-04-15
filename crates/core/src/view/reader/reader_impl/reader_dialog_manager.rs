//! Reader Dialog Manager Module
//!
//! This module handles all dialog creation and management for the Reader view,
//! including confirmation dialogs, input dialogs, and message dialogs.

use crate::color::Color;
use crate::geom::Rectangle;
use crate::view::{Hub, Id, RenderQueue};
use crate::view::dialog::Dialog;
use crate::view::keyboard::Keyboard;
use crate::view::menu::Menu;
use crate::view::menu_entry::MenuEntry;
use crate::context::Context;

/// Dialog types for the Reader
#[derive(Debug, Clone)]
pub enum ReaderDialogType {
    GoToPage,
    EditNote,
    Search,
    Settings,
    Bookmark,
    Highlight,
    ConfirmDelete,
    Info,
    Error,
}

/// Dialog data for different dialog types
#[derive(Debug, Clone)]
pub struct ReaderDialogData {
    pub title: String,
    pub message: String,
    pub default_text: Option<String>,
    pub placeholder: Option<String>,
    pub buttons: Vec<String>,
    pub dialog_type: ReaderDialogType,
}

/// Dialog manager for the Reader view
pub struct ReaderDialogManager {
    pub id: Id,
    pub current_dialog: Option<ReaderDialogType>,
    pub dialog_stack: Vec<ReaderDialogData>,
    pub max_stack_size: usize,
}

impl ReaderDialogManager {
    /// Create a new dialog manager
    pub fn new(id: Id) -> Self {
        Self {
            id,
            current_dialog: None,
            dialog_stack: Vec::new(),
            max_stack_size: 10,
        }
    }

    /// Create a go-to-page dialog
    pub fn create_go_to_page_dialog(&self, current_page: usize, total_pages: usize, context: &mut Context) -> Dialog {
        let title = format!("Go to Page (1-{})", total_pages);
        let message = format!("Current page: {}", current_page + 1);
        
        // TODO: Create proper dialog with input functionality
        Dialog::new(
            crate::view::ViewId::Dialog,
            None,
            format!("{}\n\n{}", title, message),
            context,
        )
    }

    /// Create an edit note dialog
    pub fn create_edit_note_dialog(&self, note_text: &str, context: &mut Context) -> Dialog {
        let title = "Edit Note".to_string();
        let message = "Enter your note:".to_string();
        
        // TODO: Create proper dialog with input functionality
        Dialog::new(
            crate::view::ViewId::Dialog,
            None,
            format!("{}\n\n{}", title, message),
            context,
        )
    }

    /// Create a search dialog
    pub fn create_search_dialog(&self, current_query: &str, context: &mut Context) -> Dialog {
        let title = "Search".to_string();
        let message = "Enter search text:".to_string();
        
        // TODO: Create proper dialog with input functionality
        Dialog::new(
            crate::view::ViewId::Dialog,
            None,
            format!("{}\n\n{}", title, message),
            context,
        )
    }

    /// Create a bookmark dialog
    pub fn create_bookmark_dialog(&self, page_title: &str, context: &mut Context) -> Dialog {
        let title = "Add Bookmark".to_string();
        let message = format!("Bookmark for: {}", page_title);
        
        // TODO: Create proper dialog with input functionality
        Dialog::new(
            crate::view::ViewId::Dialog,
            None,
            format!("{}\n\n{}", title, message),
            context,
        )
    }

    /// Create a highlight color dialog
    pub fn create_highlight_dialog(&self, context: &mut Context) -> Menu {
        let mut menu = Menu::new(Rectangle::default(), context);
        
        menu.add_entry(MenuEntry::new("Yellow", self.id, Some("highlight_yellow")));
        menu.add_entry(MenuEntry::new("Green", self.id, Some("highlight_green")));
        menu.add_entry(MenuEntry::new("Blue", self.id, Some("highlight_blue")));
        menu.add_entry(MenuEntry::new("Red", self.id, Some("highlight_red")));
        menu.add_entry(MenuEntry::new("Orange", self.id, Some("highlight_orange")));
        menu.add_entry(MenuEntry::new("Purple", self.id, Some("highlight_purple")));
        
        menu
    }

    /// Create a confirmation dialog
    pub fn create_confirmation_dialog(&self, title: String, message: String, context: &mut Context) -> Dialog {
        // TODO: Create proper dialog with confirmation functionality
        Dialog::new(
            crate::view::ViewId::Dialog,
            None,
            format!("{}\n\n{}", title, message),
            context,
        )
    }

    /// Create an info dialog
    pub fn create_info_dialog(&self, title: String, message: String, context: &mut Context) -> Dialog {
        Dialog::new(
            crate::view::ViewId::Dialog,
            None,
            format!("{}\n\n{}", title, message),
            context,
        )
    }

    /// Create an error dialog
    pub fn create_error_dialog(&self, message: String, context: &mut Context) -> Dialog {
        Dialog::new(
            crate::view::ViewId::Dialog,
            None,
            format!("Error\n\n{}", message),
            context,
        )
    }

    /// Show a dialog
    pub fn show_dialog(&mut self, dialog_type: ReaderDialogType, dialog_data: ReaderDialogData) -> bool {
        if self.dialog_stack.len() >= self.max_stack_size {
            return false;
        }
        
        self.current_dialog = Some(dialog_type.clone());
        self.dialog_stack.push(dialog_data);
        true
    }

    /// Hide current dialog
    pub fn hide_dialog(&mut self) -> Option<ReaderDialogData> {
        self.current_dialog.take();
        self.dialog_stack.pop()
    }

    /// Hide all dialogs
    pub fn hide_all_dialogs(&mut self) -> Vec<ReaderDialogData> {
        self.current_dialog.take();
        std::mem::take(&mut self.dialog_stack)
    }

    /// Get current dialog type
    pub fn get_current_dialog_type(&self) -> Option<&ReaderDialogType> {
        self.current_dialog.as_ref()
    }

    /// Check if dialog is active
    pub fn is_dialog_active(&self) -> bool {
        self.current_dialog.is_some()
    }

    /// Get dialog stack depth
    pub fn dialog_stack_depth(&self) -> usize {
        self.dialog_stack.len()
    }

    /// Handle dialog submission
    pub fn handle_dialog_submit(
        &mut self,
        dialog_type: &ReaderDialogType,
        text: &str,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match dialog_type {
            ReaderDialogType::GoToPage => {
                if let Ok(page_num) = text.parse::<usize>() {
                    // Emit go-to-page event
                    return true;
                }
            }
            ReaderDialogType::EditNote => {
                // Emit note-edit event
                return true;
            }
            ReaderDialogType::Search => {
                // Emit search event
                return true;
            }
            ReaderDialogType::Bookmark => {
                // Emit bookmark event
                return true;
            }
            _ => {}
        }
        false
    }

    /// Handle dialog confirmation
    pub fn handle_dialog_confirmation(
        &mut self,
        dialog_type: &ReaderDialogType,
        confirmed: bool,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match dialog_type {
            ReaderDialogType::ConfirmDelete => {
                if confirmed {
                    // Emit delete event
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// Update dialog state
    pub fn update_dialog(&mut self, rect: Rectangle, rq: &mut RenderQueue) {
        if let Some(dialog_type) = &self.current_dialog {
            // Update dialog position and size
            rq.add(crate::view::RenderData::new(
                self.id,
                rect,
                crate::framebuffer::UpdateMode::Partial,
            ));
        }
    }
}
