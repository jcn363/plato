//! Reader Menu Toggle Methods
//!
//! This module provides thin wrapper methods for toggling various reader menus
//! and dialogs. These methods delegate to specialized modules:
//! - `reader_dialogs`: Note editing, page naming, go-to-page dialogs
//! - `reader_settings`: Settings menus (font, display, navigation)
//! - `reader_search`: Search functionality
//! - `reader_annotations`: Annotation-related menus
//!
//! Each toggle method handles showing/hiding the specified UI component
//! and manages focus state appropriately.
use crate::context::Context;
use crate::geom::Rectangle;
use crate::view::{Hub, RenderQueue, ViewId};
use crate::document::Annotation;
use super::reader::Reader;

impl Reader {
    pub fn toggle_edit_note(&mut self, text: Option<&str>, enable: Option<bool>, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        super::reader_dialogs::toggle_edit_note(&mut self.children, text, enable, hub, rq, context);
    }

    pub fn toggle_name_page(&mut self, enable: Option<bool>, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        super::reader_dialogs::toggle_name_page(&mut self.children, enable, hub, rq, context);
        if let Some(false) = enable {
            if self.focus.map(|focus_id| focus_id == ViewId::NamePageInput).unwrap_or(false) {
                self.toggle_keyboard(false, None, hub, rq, context);
            }
        }
    }

    pub fn toggle_go_to_page(&mut self, enable: Option<bool>, id: ViewId, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        super::reader_dialogs::toggle_go_to_page(&mut self.children, enable, id, hub, rq, context);
        if let Some(false) = enable {
            let input_id = if id == ViewId::GoToPage { ViewId::GoToPageInput } else { ViewId::GoToResultsPageInput };
            if self.focus.map(|focus_id| focus_id == input_id).unwrap_or(false) {
                self.toggle_keyboard(false, None, hub, rq, context);
            }
        }
    }

    pub fn toggle_annotation_menu(&mut self, annot: &Annotation, rect: Rectangle, enable: Option<bool>, rq: &mut RenderQueue, context: &mut Context) {
        super::reader_settings::toggle_annotation_menu(&mut self.children, annot, rect, enable, rq, context);
    }

    pub fn toggle_selection_menu(&mut self, rect: Rectangle, enable: Option<bool>, rq: &mut RenderQueue, context: &mut Context) {
        let file_kind = self.info.file.kind.as_str();
        let file_path = context.library.home.join(&self.info.file.path);
        let file_path_str = file_path.to_string_lossy().to_string();
        let has_page_names = self.info.reader.as_ref().map_or(false, |r| !r.page_names.is_empty());
        super::reader_settings::toggle_selection_menu(&mut self.children, self.current_page, file_kind, if file_kind == "epub" { Some(file_path_str) } else { None }, has_page_names, rect, enable, rq, context);
    }

    pub fn toggle_title_menu(&mut self, rect: Rectangle, enable: Option<bool>, rq: &mut RenderQueue, context: &mut Context) {
        let file_kind = self.info.file.kind.as_str();
        let file_path = context.library.home.join(&self.info.file.path);
        let file_path_str = file_path.to_string_lossy().to_string();
        let has_annotations = self.info.reader.as_ref().map_or(false, |r| !r.annotations.is_empty());
        let has_bookmarks = self.info.reader.as_ref().map_or(false, |r| !r.bookmarks.is_empty());
        super::reader_settings::toggle_title_menu(&mut self.children, rect, self.reflowable, file_kind, if file_kind == "epub" { Some(file_path_str) } else { None }, has_annotations, has_bookmarks, self.view_port.zoom_mode, self.view_port.scroll_mode, enable, rq, context);
    }

    pub fn toggle_font_family_menu(&mut self, rect: Rectangle, enable: Option<bool>, _hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let current_family = self.info.reader.as_ref().and_then(|r| r.font_family.clone()).unwrap_or_else(|| context.settings.reader.font_family.clone());
        super::reader_settings::toggle_font_family_menu(&mut self.children, current_family, rect, enable, rq, context);
    }

    pub fn toggle_font_size_menu(&mut self, rect: Rectangle, enable: Option<bool>, rq: &mut RenderQueue, context: &mut Context) {
        let current_size = self.info.reader.as_ref().and_then(|r| r.font_size).unwrap_or(context.settings.reader.font_size);
        super::reader_settings::toggle_font_size_menu(&mut self.children, current_size, rect, enable, rq, context);
    }

    pub fn toggle_text_align_menu(&mut self, rect: Rectangle, enable: Option<bool>, rq: &mut RenderQueue, context: &mut Context) {
        let current_align = self.info.reader.as_ref().and_then(|r| r.text_align).unwrap_or(context.settings.reader.text_align);
        super::reader_settings::toggle_text_align_menu(&mut self.children, current_align, rect, enable, rq, context);
    }

    pub fn toggle_line_height_menu(&mut self, rect: Rectangle, enable: Option<bool>, rq: &mut RenderQueue, context: &mut Context) {
        let current_height = self.info.reader.as_ref().and_then(|r| r.line_height).unwrap_or(context.settings.reader.line_height);
        super::reader_settings::toggle_line_height_menu(&mut self.children, current_height, rect, enable, rq, context);
    }

    pub fn toggle_contrast_exponent_menu(&mut self, rect: Rectangle, enable: Option<bool>, rq: &mut RenderQueue, context: &mut Context) {
        super::reader_settings::toggle_contrast_exponent_menu(&mut self.children, self.contrast.exponent, rect, enable, rq, context);
    }

    pub fn toggle_contrast_gray_menu(&mut self, rect: Rectangle, enable: Option<bool>, rq: &mut RenderQueue, context: &mut Context) {
        super::reader_settings::toggle_contrast_gray_menu(&mut self.children, self.contrast.gray, rect, enable, rq, context);
    }

    pub fn toggle_margin_width_menu(&mut self, rect: Rectangle, enable: Option<bool>, rq: &mut RenderQueue, context: &mut Context) {
        let margin_width = self.info.reader.as_ref().and_then(|r| if self.reflowable { r.margin_width } else { r.screen_margin_width }).unwrap_or_else(|| if self.reflowable { context.settings.reader.margin_width } else { 0 });
        super::reader_settings::toggle_margin_width_menu(&mut self.children, margin_width, rect, enable, rq, context);
    }

    pub fn toggle_page_menu(&mut self, rect: Rectangle, enable: Option<bool>, rq: &mut RenderQueue, context: &mut Context) {
        super::reader_settings::toggle_page_menu(&mut self.children, self.current_page, &self.info, rect, enable, rq, context);
    }

    pub fn toggle_margin_cropper_menu(&mut self, rect: Rectangle, enable: Option<bool>, rq: &mut RenderQueue, context: &mut Context) {
        super::reader_settings::toggle_margin_cropper_menu(&mut self.children, self.current_page, &self.info, rect, enable, rq, context);
    }

    pub fn toggle_search_menu(&mut self, rect: Rectangle, enable: Option<bool>, rq: &mut RenderQueue, context: &mut Context) {
        super::reader_search::toggle_search_menu(&mut self.children, self.search_direction, rect, enable, rq, context);
    }
}
