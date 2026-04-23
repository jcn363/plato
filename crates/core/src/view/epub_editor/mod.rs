//! EPUB Editor view for Plato e-reader.
//!
//! This module provides a graphical EPUB editor that allows users to edit
//! chapter content directly on their device. It's designed to fix errors
//! encountered while reading EPUB books.

mod state;
mod helpers;

pub use state::{EditorState, SearchReplaceState};
use helpers::{show_chapter_list, show_metadata_edit_view, show_save_dialog, show_edit_view, show_search_replace, do_search, do_replace_in_chapter, update_input_field, close_search_replace};

use std::path::Path;

use crate::color;
use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::impl_view_boilerplate;
use crate::log_error;
use crate::theme;
use crate::unit::scale_by_dpi;
use crate::view::common::toggle_main_menu;
use crate::view::filler::Filler;
use crate::view::icon::Icon;
use crate::view::input_field::InputField;
use crate::view::notification::Notification;
use crate::view::search_replace::SearchReplaceView;
use crate::view::top_bar::TopBar;
use crate::view::SMALL_BAR_HEIGHT;
use crate::view::{
    Bus, EntryId, Event, Hub, Id, RenderData, RenderQueue, View, ViewId,
    ID_FEEDER,
};
use anyhow::Error;
use epub_edit::EpubEditorCore;

/// EPUB Editor view providing on-device editing capabilities.
pub struct EpubEditor {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    core: EpubEditorCore,
    state: EditorState,
    modified: bool,
    modified_chapters: Vec<usize>,
    search_replace: Option<SearchReplaceState>,
}

impl EpubEditor {
    pub fn new(
        rect: Rectangle,
        epub_path: String,
        chapter: Option<usize>,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<EpubEditor, Error> {
        let id = ID_FEEDER.next();
        let dpi = crate::unit::get_device_dpi();
        let side = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;

        let core = EpubEditorCore::new(&epub_path)?;

        let mut children = Vec::new();

        let top_bar = TopBar::new(
            rect![rect.min.x, rect.min.y, rect.max.x, rect.min.y + side],
            Event::Back,
            "EPUB Editor".to_string(),
            context,
        );
        let mut top_bar = top_bar;
        // Add Undo and Redo buttons to TopBar
        let undo_rect = rect![
            rect.max.x - 3 * side,
            rect.min.y,
            rect.max.x - 2 * side,
            rect.min.y + side
        ];
        let redo_rect = rect![
            rect.max.x - 2 * side,
            rect.min.y,
            rect.max.x - side,
            rect.min.y + side
        ];
        top_bar.children_mut().push(Box::new(Icon::new(
            "undo",
            undo_rect,
            Event::Select(EntryId::Undo),
        )));
        top_bar.children_mut().push(Box::new(Icon::new(
            "redo",
            redo_rect,
            Event::Select(EntryId::Redo),
        )));
        let search_rect = rect![rect.max.x - side, rect.min.y, rect.max.x, rect.min.y + side];
        top_bar.children_mut().push(Box::new(Icon::new(
            "search",
            search_rect,
            Event::Select(EntryId::SearchReplace),
        )));
        let metadata_rect = rect![
            rect.max.x - 2 * side,
            rect.min.y,
            rect.max.x - side,
            rect.min.y + side
        ];
        top_bar.children_mut().push(Box::new(Icon::new(
            "info",
            metadata_rect,
            Event::Select(EntryId::EditMetadata),
        )));
        children.push(Box::new(top_bar) as Box<dyn View>);

        let separator = Filler::new(
            rect![
                rect.min.x,
                rect.min.y + side,
                rect.max.x,
                rect.min.y + side + 1
            ],
            color::foreground(theme::is_dark_mode()),
        );
        children.push(Box::new(separator) as Box<dyn View>);

        rq.add(RenderData::new(id, rect, UpdateMode::Full));

        let mut editor = EpubEditor {
            id,
            rect,
            children,
            core,
            state: EditorState::ChapterList,
            modified: false,
            modified_chapters: Vec::new(),
            search_replace: None,
        };

        let start_chapter = chapter
            .unwrap_or(0)
            .min(editor.core.chapters.len().saturating_sub(1));

        if chapter.is_some() && !editor.core.chapters.is_empty() {
            show_edit_view(&mut editor, start_chapter, hub, rq, context);
        } else {
            show_chapter_list(&mut editor, hub, rq, context);
        }
        Ok(editor)
    }

    fn update_chapter_content(
        &mut self,
        index: usize,
        new_content: String,
        _rq: &mut RenderQueue,
    ) -> bool {
        if let Err(e) = self.core.update_chapter(index, new_content) {
            log_error!("Failed to update chapter: {}", e);
            return false;
        }
        self.modified = true;
        if !self.modified_chapters.contains(&index) {
            self.modified_chapters.push(index);
        }
        true
    }

    fn undo(&mut self, rq: &mut RenderQueue) -> bool {
        match self.core.undo() {
            Ok(true) => {
                self.modified = true;
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            _ => false,
        }
    }

    fn redo(&mut self, rq: &mut RenderQueue) -> bool {
        match self.core.redo() {
            Ok(true) => {
                self.modified = true;
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            _ => false,
        }
    }
}

impl View for EpubEditor {
    fn view_id(&self) -> Option<ViewId> {
        Some(ViewId::EpubEditor)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match event {
            Event::Back => {
                match self.state {
                    EditorState::EditingChapter { .. } => {
                        if self.modified {
                            show_save_dialog(self, hub, rq, context);
                        } else {
                            show_chapter_list(self, hub, rq, context);
                        }
                        return true;
                    }
                    EditorState::ChapterList => {
                        if self.modified {
                            show_save_dialog(self, hub, rq, context);
                        }
                    }
                }
                false
            }
            Event::Select(EntryId::SelectChapter(i)) => {
                show_edit_view(self, *i, hub, rq, context);
                true
            }
            Event::Select(EntryId::EditMetadata) => {
                show_metadata_edit_view(self, hub, rq, context);
                true
            }
            Event::Select(EntryId::PreviousChapter) => {
                if let EditorState::EditingChapter { index } = self.state {
                    if index > 0 {
                        show_edit_view(self, index - 1, hub, rq, context);
                    }
                }
                true
            }
            Event::Select(EntryId::NextChapter) => {
                if let EditorState::EditingChapter { index } = self.state {
                    if index + 1 < self.core.chapters.len() {
                        show_edit_view(self, index + 1, hub, rq, context);
                    }
                }
                true
            }
            Event::Select(EntryId::ToggleRegex) => {
                if let Some(sr) = self
                    .children
                    .iter_mut()
                    .find(|c| c.is::<SearchReplaceView>())
                {
                    if let Some(view) = sr.downcast_mut::<SearchReplaceView>() {
                        view.toggle_regex();
                        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                    }
                }
                true
            }
            Event::Select(EntryId::ToggleCaseSensitive) => {
                if let Some(sr) = self
                    .children
                    .iter_mut()
                    .find(|c| c.is::<SearchReplaceView>())
                {
                    if let Some(view) = sr.downcast_mut::<SearchReplaceView>() {
                        view.toggle_case_sensitive();
                        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                    }
                }
                true
            }
            Event::Select(EntryId::ToggleWholeWord) => {
                if let Some(sr) = self
                    .children
                    .iter_mut()
                    .find(|c| c.is::<SearchReplaceView>())
                {
                    if let Some(view) = sr.downcast_mut::<SearchReplaceView>() {
                        view.toggle_whole_word();
                        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                    }
                }
                true
            }
            Event::Select(EntryId::SaveMetadata) => {
                let mut new_meta = self.core.metadata.clone();
                if let Some(view) = self.children.iter().find(|c| c.is::<InputField>()) {
                    if let Some(input) = view.downcast_ref::<InputField>() {
                        if input.view_id() == Some(ViewId::EditMetadataTitle) {
                            new_meta.title = input.get_text().to_string();
                        }
                    }
                }
                if let Some(view) = self.children.iter().find(|c| {
                    c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataAuthor)
                }) {
                    if let Some(input) = view.downcast_ref::<InputField>() {
                        new_meta.author = input.get_text().to_string();
                    }
                }
                if let Some(view) = self.children.iter().find(|c| {
                    c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataLanguage)
                }) {
                    if let Some(input) = view.downcast_ref::<InputField>() {
                        new_meta.language = input.get_text().to_string();
                    }
                }
                if let Some(view) = self.children.iter().find(|c| {
                    c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataIdentifier)
                }) {
                    if let Some(input) = view.downcast_ref::<InputField>() {
                        new_meta.identifier = input.get_text().to_string();
                    }
                }
                if let Some(view) = self.children.iter().find(|c| {
                    c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataPublisher)
                }) {
                    if let Some(input) = view.downcast_ref::<InputField>() {
                        new_meta.publisher = Some(input.get_text().to_string());
                    }
                }
                if let Some(view) = self
                    .children
                    .iter()
                    .find(|c| c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataDate))
                {
                    if let Some(input) = view.downcast_ref::<InputField>() {
                        new_meta.date = Some(input.get_text().to_string());
                    }
                }
                self.core.set_metadata(new_meta);
                self.modified = true;
                let notif = Notification::new("Metadata updated".to_string(), hub, rq, context);
                self.children.push(Box::new(notif) as Box<dyn View>);
                show_chapter_list(self, hub, rq, context);
                true
            }
            Event::Select(EntryId::Save) => {
                if let Err(e) = self.core.save() {
                    let notif = Notification::new(format!("Error saving: {}", e), hub, rq, context);
                    self.children.push(Box::new(notif) as Box<dyn View>);
                } else {
                    let notif = Notification::new("Changes saved!".to_string(), hub, rq, context);
                    self.children.push(Box::new(notif) as Box<dyn View>);
                }
                self.modified = false;
                self.modified_chapters.clear();
                false
            }
            Event::Select(EntryId::Discard) => {
                self.modified = false;
                self.modified_chapters.clear();
                false
            }
            Event::Submit(ViewId::EditNoteInput, text) => {
                if let EditorState::EditingChapter { index } = self.state {
                    if self.update_chapter_content(index, text.clone(), rq) {
                        let notif = Notification::new(
                            format!("Chapter {} saved!", self.core.chapters[index].title),
                            hub,
                            rq,
                            context,
                        );
                        self.children.push(Box::new(notif) as Box<dyn View>);
                    }
                }
                true
            }
            Event::ToggleNear(ViewId::MainMenu, rect) => {
                toggle_main_menu(self, *rect, None, rq, context);
                true
            }
            Event::Select(EntryId::Undo) => {
                if self.undo(rq) {
                    bus.push_back(Event::Render("Undone".to_string()));
                }
                true
            }
            Event::Select(EntryId::Redo) => {
                if self.redo(rq) {
                    bus.push_back(Event::Render("Redone".to_string()));
                }
                true
            }
            Event::Select(EntryId::Preview) => {
                if let EditorState::EditingChapter { index } = self.state {
                    bus.push_back(Event::Render(format!(
                        "Preview: {}",
                        self.core.chapters[index].title
                    )));
                }
                true
            }
            Event::Select(EntryId::SearchReplace) => {
                self.search_replace = Some(SearchReplaceState {
                    search_text: String::with_capacity(32),
                    replace_text: String::with_capacity(32),
                });
                show_search_replace(self, hub, rq, context);
                true
            }
            Event::SearchReplace => {
                if let Some(state) = self.search_replace.as_mut() {
                    if let Some(view) = self.children.iter().find(|c| c.is::<SearchReplaceView>()) {
                        if let Some(sr_view) = view.downcast_ref::<SearchReplaceView>() {
                            state.search_text = sr_view.get_search_text().to_string();
                            state.replace_text = sr_view.get_replace_text().to_string();
                        }
                    }
                }
                do_search(self, rq, context);
                true
            }
            Event::Select(EntryId::ReplaceInChapter) => {
                if let Some(state) = self.search_replace.as_mut() {
                    if let Some(view) = self.children.iter().find(|c| c.is::<SearchReplaceView>()) {
                        if let Some(sr_view) = view.downcast_ref::<SearchReplaceView>() {
                            state.search_text = sr_view.get_search_text().to_string();
                            state.replace_text = sr_view.get_replace_text().to_string();
                        }
                    }
                }
                do_replace_in_chapter(self, hub, rq, context);
                true
            }
            Event::Select(EntryId::ReplaceInDocument) => {
                if let Some(state) = self.search_replace.as_mut() {
                    if let Some(view) = self.children.iter().find(|c| c.is::<SearchReplaceView>()) {
                        if let Some(sr_view) = view.downcast_ref::<SearchReplaceView>() {
                            state.search_text = sr_view.get_search_text().to_string();
                            state.replace_text = sr_view.get_replace_text().to_string();
                        }
                    }
                }
                if let Some(state) = &self.search_replace {
                    if state.search_text.is_empty() {
                        let notif =
                            Notification::new("Search text is empty".to_string(), hub, rq, context);
                        self.children.push(Box::new(notif) as Box<dyn View>);
                        return true;
                    }
                    let options = self
                        .children
                        .iter()
                        .find(|c| c.is::<SearchReplaceView>())
                        .and_then(|v| v.downcast_ref::<SearchReplaceView>())
                        .map(|sr| {
                            let (use_regex, case_sensitive, whole_word) = sr.get_search_options();
                            epub_edit::SearchOptions {
                                use_regex,
                                case_sensitive,
                                whole_word,
                            }
                        })
                        .unwrap_or_default();
                    match self.core.replace_all_in_document(
                        &state.search_text,
                        &state.replace_text,
                        options,
                    ) {
                        Ok(count) => {
                            if count > 0 {
                                self.modified = true;
                                let notif = Notification::new(
                                    format!("Replaced {} occurrence(s) in document", count),
                                    hub,
                                    rq,
                                    context,
                                );
                                self.children.push(Box::new(notif) as Box<dyn View>);
                                if let EditorState::EditingChapter { index: _ } = self.state {
                                    update_input_field(self, rq, context);
                                }
                            } else {
                                let notif = Notification::new(
                                    "No matches found in document".to_string(),
                                    hub,
                                    rq,
                                    context,
                                );
                                self.children.push(Box::new(notif) as Box<dyn View>);
                            }
                        }
                        Err(e) => {
                            let notif = Notification::new(
                                format!("Replace error: {}", e),
                                hub,
                                rq,
                                context,
                            );
                            self.children.push(Box::new(notif) as Box<dyn View>);
                        }
                    }
                }
                true
            }
            Event::Select(EntryId::CloseSearchReplace) => {
                self.search_replace = None;
                close_search_replace(self, rq);
                true
            }
            Event::Select(EntryId::ValidateContent) => {
                let result = self.core.validate_content();
                if result.issues.is_empty() {
                    let notif = Notification::new(
                        format!(
                            "Validation passed: {} chapters checked",
                            result.total_chapters
                        ),
                        hub,
                        rq,
                        context,
                    );
                    self.children.push(Box::new(notif) as Box<dyn View>);
                } else {
                    let notif = Notification::new(
                        format!(
                            "Found {} issues in {} chapters",
                            result.issues.len(),
                            result.chapters_with_issues
                        ),
                        hub,
                        rq,
                        context,
                    );
                    self.children.push(Box::new(notif) as Box<dyn View>);
                }
                true
            }
            Event::Select(EntryId::RenameChapter) => {
                if let EditorState::EditingChapter { index: _ } = self.state {
                    let notif = Notification::new(
                        "Chapter rename feature - UI input needed".to_string(),
                        hub,
                        rq,
                        context,
                    );
                    self.children.push(Box::new(notif) as Box<dyn View>);
                }
                true
            }
            Event::Select(EntryId::DeleteChapter) => {
                if let EditorState::EditingChapter { index } = self.state {
                    match self.core.delete_chapter(index) {
                        Ok(_) => {
                            self.modified = true;
                            let notif =
                                Notification::new("Chapter deleted".to_string(), hub, rq, context);
                            self.children.push(Box::new(notif) as Box<dyn View>);
                        }
                        Err(e) => {
                            let notif = Notification::new(
                                format!("Error deleting chapter: {}", e),
                                hub,
                                rq,
                                context,
                            );
                            self.children.push(Box::new(notif) as Box<dyn View>);
                        }
                    }
                }
                true
            }
            Event::Select(EntryId::MoveChapterUp) => {
                if let EditorState::EditingChapter { index } = self.state {
                    if index > 0 {
                        match self.core.reorder_chapters(index, index - 1) {
                            Ok(_) => {
                                self.modified = true;
                                let notif = Notification::new(
                                    "Chapter moved up".to_string(),
                                    hub,
                                    rq,
                                    context,
                                );
                                self.children.push(Box::new(notif) as Box<dyn View>);
                            }
                            Err(e) => {
                                let notif = Notification::new(
                                    format!("Error moving chapter: {}", e),
                                    hub,
                                    rq,
                                    context,
                                );
                                self.children.push(Box::new(notif) as Box<dyn View>);
                            }
                        }
                    }
                }
                true
            }
            Event::Select(EntryId::MoveChapterDown) => {
                if let EditorState::EditingChapter { index } = self.state {
                    if index < self.core.chapters.len() - 1 {
                        match self.core.reorder_chapters(index, index + 1) {
                            Ok(_) => {
                                self.modified = true;
                                let notif = Notification::new(
                                    "Chapter moved down".to_string(),
                                    hub,
                                    rq,
                                    context,
                                );
                                self.children.push(Box::new(notif) as Box<dyn View>);
                            }
                            Err(e) => {
                                let notif = Notification::new(
                                    format!("Error moving chapter: {}", e),
                                    hub,
                                    rq,
                                    context,
                                );
                                self.children.push(Box::new(notif) as Box<dyn View>);
                            }
                        }
                    }
                }
                true
            }
            Event::Select(EntryId::SpellCheck) => {
                let result = self.core.spell_check();
                if result.errors.is_empty() {
                    let notif = Notification::new(
                        format!(
                            "Spell check passed: {} words checked in {} chapters",
                            result.total_words, result.chapters_checked
                        ),
                        hub,
                        rq,
                        context,
                    );
                    self.children.push(Box::new(notif) as Box<dyn View>);
                } else {
                    let notif = Notification::new(
                        format!(
                            "Found {} potential spelling errors in {} chapters",
                            result.errors.len(),
                            result
                                .errors
                                .iter()
                                .map(|e| e.chapter_index)
                                .collect::<std::collections::HashSet<_>>()
                                .len()
                        ),
                        hub,
                        rq,
                        context,
                    );
                    self.children.push(Box::new(notif) as Box<dyn View>);
                }
                true
            }
            Event::Select(EntryId::ExportChapter) => {
                if let EditorState::EditingChapter { index } = self.state {
                    let export_path = format!("/tmp/chapter_{}.txt", index);
                    let path = Path::new(&export_path);
                    match self.core.export_chapter(index, path) {
                        Ok(_) => {
                            let notif = Notification::new(
                                format!("Chapter exported to {}", export_path),
                                hub,
                                rq,
                                context,
                            );
                            self.children.push(Box::new(notif) as Box<dyn View>);
                        }
                        Err(e) => {
                            let notif = Notification::new(
                                format!("Error exporting chapter: {}", e),
                                hub,
                                rq,
                                context,
                            );
                            self.children.push(Box::new(notif) as Box<dyn View>);
                        }
                    }
                }
                true
            }
            Event::Select(EntryId::ImportChapter) => {
                if let EditorState::EditingChapter { index: _ } = self.state {
                    let notif = Notification::new(
                        "Chapter import - file path selection needed".to_string(),
                        hub,
                        rq,
                        context,
                    );
                    self.children.push(Box::new(notif) as Box<dyn View>);
                }
                true
            }
            Event::Select(EntryId::ChapterStatistics) => {
                if let EditorState::EditingChapter { index } = self.state {
                    if let Some(stats) = self.core.get_chapter_statistics(index) {
                        let notif = Notification::new(
                            format!(
                                "Chapter {}: {} words, {} characters, {} paragraphs",
                                stats.chapter_title,
                                stats.word_count,
                                stats.character_count,
                                stats.paragraph_count
                            ),
                            hub,
                            rq,
                            context,
                        );
                        self.children.push(Box::new(notif) as Box<dyn View>);
                    }
                }
                true
            }
            Event::Select(EntryId::GenerateTOC) => {
                match self.core.update_table_of_contents() {
                    Ok(_) => {
                        self.modified = true;
                        let notif = Notification::new(
                            format!(
                                "Table of contents generated for {} chapters",
                                self.core.chapters.len()
                            ),
                            hub,
                            rq,
                            context,
                        );
                        self.children.push(Box::new(notif) as Box<dyn View>);
                    }
                    Err(e) => {
                        let notif = Notification::new(
                            format!("Error generating table of contents: {}", e),
                            hub,
                            rq,
                            context,
                        );
                        self.children.push(Box::new(notif) as Box<dyn View>);
                    }
                }
                true
            }
            Event::Select(EntryId::ListImages) => {
                let images = self.core.list_images();
                let notif = Notification::new(
                    format!(
                        "Found {} images across {} chapters",
                        images.len(),
                        images
                            .iter()
                            .map(|i| i.chapter_index)
                            .collect::<std::collections::HashSet<_>>()
                            .len()
                    ),
                    hub,
                    rq,
                    context,
                );
                self.children.push(Box::new(notif) as Box<dyn View>);
                true
            }
            Event::Select(EntryId::ClearHistory) => {
                self.core.clear_history();
                let notif =
                    Notification::new("Undo/redo history cleared".to_string(), hub, rq, context);
                self.children.push(Box::new(notif) as Box<dyn View>);
                true
            }
            Event::Select(EntryId::ListCSS) => {
                let css_files = self.core.list_css();
                let notif = Notification::new(
                    format!(
                        "Found {} CSS files across {} chapters",
                        css_files.len(),
                        css_files
                            .iter()
                            .map(|c| c.chapter_index)
                            .collect::<std::collections::HashSet<_>>()
                            .len()
                    ),
                    hub,
                    rq,
                    context,
                );
                self.children.push(Box::new(notif) as Box<dyn View>);
                true
            }
            Event::Select(EntryId::AddBookmark) => {
                if let EditorState::EditingChapter { index } = self.state {
                    self.core.add_bookmark(index, 0, None);
                    let notif = Notification::new(
                        format!(
                            "Bookmark added for chapter: {}",
                            self.core.chapters[index].title
                        ),
                        hub,
                        rq,
                        context,
                    );
                    self.children.push(Box::new(notif) as Box<dyn View>);
                }
                true
            }
            Event::Select(EntryId::ReplaceAllInAllDocuments) => {
                if let Some(state) = &self.search_replace {
                    if state.search_text.is_empty() {
                        let notif =
                            Notification::new("Search text is empty".to_string(), hub, rq, context);
                        self.children.push(Box::new(notif) as Box<dyn View>);
                        return true;
                    }
                    let search_text = state.search_text.clone();
                    let options = self
                        .children
                        .iter()
                        .find(|c| c.is::<SearchReplaceView>())
                        .and_then(|v| v.downcast_ref::<SearchReplaceView>())
                        .map(|sr| {
                            let (use_regex, case_sensitive, whole_word) = sr.get_search_options();
                            epub_edit::SearchOptions {
                                use_regex,
                                case_sensitive,
                                whole_word,
                            }
                        })
                        .unwrap_or_default();
                    match self.core.replace_all_in_all_chapters(
                        &search_text,
                        &state.replace_text,
                        options,
                    ) {
                        Ok(count) => {
                            self.modified = true;
                            let notif = Notification::new(
                                format!("Replaced {} occurrences across all chapters", count),
                                hub,
                                rq,
                                context,
                            );
                            self.children.push(Box::new(notif) as Box<dyn View>);
                        }
                        Err(e) => {
                            let notif = Notification::new(
                                format!("Error replacing in all chapters: {}", e),
                                hub,
                                rq,
                                context,
                            );
                            self.children.push(Box::new(notif) as Box<dyn View>);
                        }
                    }
                }
                true
            }
            Event::Close(ViewId::EpubEditor) => {
                if self.search_replace.is_some() {
                    self.search_replace = None;
                    close_search_replace(self, rq);
                    true
                } else {
                    false
                }
            }
            Event::Submit(ViewId::EpubEditorSearchInput, text) => {
                if let Some(state) = self.search_replace.as_mut() {
                    state.search_text = text.clone();
                }
                do_search(self, rq, context);
                true
            }
            Event::Submit(ViewId::EpubEditorReplaceInput, text) => {
                if let Some(state) = self.search_replace.as_mut() {
                    state.replace_text = text.clone();
                }
                true
            }
            _ => {
                for child in self.children_mut().iter_mut() {
                    if child.handle_event(event, hub, bus, rq, context) {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn render(
        &self,
        fb: &mut dyn crate::framebuffer::Framebuffer,
        rect: Rectangle,
        fonts: &mut crate::font::Fonts,
    ) {
        for child in self.children().iter() {
            child.render(fb, rect, fonts);
        }
    }

    fn might_rotate(&self) -> bool {
        false
    }

    impl_view_boilerplate!();
}
