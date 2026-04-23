//! EPUB Editor view for Plato e-reader.
//!
//! This module provides a graphical EPUB editor that allows users to edit
//! chapter content directly on their device. It's designed to fix errors
//! encountered while reading EPUB books.

mod chapter;
mod event_handlers;
mod helpers;
mod metadata;
mod navigation;
mod search_replace;
mod state;

use event_handlers::handle_event;
use helpers::{show_chapter_list, show_edit_view};
pub use state::{EditorState, SearchReplaceState};

use crate::color;
use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::impl_view_boilerplate;
use crate::log_error;
use crate::theme;
use crate::unit::scale_by_dpi;
use crate::view::filler::Filler;
use crate::view::icon::Icon;
use crate::view::top_bar::TopBar;
use crate::view::SMALL_BAR_HEIGHT;
use crate::view::{Bus, EntryId, Event, Hub, Id, RenderData, RenderQueue, View, ViewId, ID_FEEDER};
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
        handle_event(self, event, hub, bus, rq, context)
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
