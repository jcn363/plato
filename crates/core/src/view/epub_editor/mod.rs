//! EPUB Editor view for Plato e-reader.
//!
//! This module provides a graphical EPUB editor that allows users to edit
//! chapter content directly on their device. It's designed to fix errors
//! encountered while reading EPUB books.

use crate::color::BLACK;
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::geom::{halves, Rectangle};
use crate::impl_view_boilerplate;
use crate::log_error;
use crate::unit::scale_by_dpi;
use crate::view::common::toggle_main_menu;
use crate::view::filler::Filler;
use crate::view::icon::Icon;
use crate::view::input_field::InputField;
use crate::view::keyboard::Keyboard;
use crate::view::label::Label;
use crate::view::menu::{Menu, MenuKind};
use crate::view::notification::Notification;
use crate::view::search_replace::SearchReplaceView;
use crate::view::top_bar::TopBar;
use crate::view::SMALL_BAR_HEIGHT;
use crate::view::THICKNESS_MEDIUM;
use crate::view::{Align, EntryId, EntryKind, ViewId};
use crate::view::{Bus, Event, Hub, RenderData, RenderQueue, View};
use crate::view::{Id, ID_FEEDER};
use anyhow::Error;
use epub_edit::EpubEditorCore;

/// Current state of the editor UI.
enum EditorState {
    /// Showing the list of chapters to choose from
    ChapterList,
    /// Currently editing a specific chapter
    EditingChapter { index: usize },
}

/// EPUB Editor view providing on-device editing capabilities.
pub struct EpubEditor {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    core: EpubEditorCore,
    state: EditorState,
    modified: bool,
    search_replace: Option<SearchReplaceState>,
}

struct SearchReplaceState {
    search_text: String,
    replace_text: String,
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
        let dpi = CURRENT_DEVICE.dpi;
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
        children.push(Box::new(top_bar) as Box<dyn View>);

        let separator = Filler::new(
            rect![
                rect.min.x,
                rect.min.y + side,
                rect.max.x,
                rect.min.y + side + 1
            ],
            BLACK,
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
            search_replace: None,
        };

        let start_chapter = chapter
            .unwrap_or(0)
            .min(editor.core.chapters.len().saturating_sub(1));

        if chapter.is_some() && !editor.core.chapters.is_empty() {
            editor.show_edit_view(start_chapter, hub, rq, context);
        } else {
            editor.show_chapter_list(hub, rq, context);
        }
        Ok(editor)
    }

    fn show_chapter_list(&mut self, _hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        self.state = EditorState::ChapterList;
        self.children
            .retain(|c| !c.is::<Menu>() && !c.is::<Notification>());

        let entries: Vec<EntryKind> = self
            .core
            .chapters
            .iter()
            .enumerate()
            .map(|(i, chapter)| {
                EntryKind::Command(chapter.title.clone(), EntryId::SelectChapter(i))
            })
            .collect();

        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let rect = rect![
            self.rect.min.x,
            self.rect.min.y + small_height + 1,
            self.rect.max.x,
            self.rect.max.y
        ];

        let menu = Menu::new(
            rect,
            ViewId::BookMenu,
            MenuKind::Contextual,
            entries,
            context,
        );
        rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
        self.children.push(Box::new(menu) as Box<dyn View>);
    }

    fn show_save_dialog(&mut self, _hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        self.children
            .retain(|c| !c.is::<Menu>() && !c.is::<Notification>());

        let entries = vec![
            EntryKind::Command("Save".to_string(), EntryId::Save),
            EntryKind::Command("Discard".to_string(), EntryId::Discard),
        ];

        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let rect = rect![
            self.rect.min.x,
            self.rect.min.y + small_height + 10,
            self.rect.max.x,
            self.rect.min.y + small_height + 120
        ];

        let menu = Menu::new(
            rect,
            ViewId::BookMenu,
            MenuKind::Contextual,
            entries,
            context,
        );
        rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
        self.children.push(Box::new(menu) as Box<dyn View>);
    }

    fn show_edit_view(
        &mut self,
        index: usize,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if index >= self.core.chapters.len() {
            return;
        }

        self.state = EditorState::EditingChapter { index };
        self.children
            .retain(|c| !c.is::<Menu>() && !c.is::<Notification>());

        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, _) = halves(thickness);

        let chapter = &self.core.chapters[index];
        let content: String = chapter.content.chars().take(5000).collect();

        let title_label = Label::new(
            rect![
                self.rect.min.x + 10,
                self.rect.min.y + small_height + 10,
                self.rect.max.x - 10,
                self.rect.min.y + small_height + 40
            ],
            format!("Editing: {}", chapter.title),
            Align::Left(0),
        );
        self.children.push(Box::new(title_label) as Box<dyn View>);

        let textarea_rect = rect![
            self.rect.min.x + 10,
            self.rect.min.y + small_height + 50,
            self.rect.max.x - 10,
            self.rect.max.y - small_height - 60
        ];

        let input_field =
            InputField::new(textarea_rect, ViewId::EditNoteInput).text(&content, context);
        self.children.push(Box::new(input_field) as Box<dyn View>);

        let sep_rect = rect![
            self.rect.min.x,
            self.rect.max.y - small_height - small_thickness,
            self.rect.max.x,
            self.rect.max.y - small_height
        ];
        let separator = Filler::new(sep_rect, BLACK);
        self.children.push(Box::new(separator) as Box<dyn View>);

        let kb_rect = rect![
            self.rect.min.x,
            self.rect.max.y - small_height,
            self.rect.max.x,
            self.rect.max.y
        ];

        let mut kb_rect_mut = kb_rect;
        let keyboard = Keyboard::new(&mut kb_rect_mut, true, context);
        self.children.push(Box::new(keyboard) as Box<dyn View>);

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
        hub.send(Event::Focus(Some(ViewId::EditNoteInput))).ok();
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

    fn show_search_replace(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        self.children.retain(|c| !c.is::<SearchReplaceView>());

        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let popup_height = 160;
        let popup_rect = rect![
            self.rect.min.x + 20,
            self.rect.min.y + small_height + 10,
            self.rect.max.x - 20,
            self.rect.min.y + small_height + 10 + popup_height
        ];

        let (search_text, replace_text) = match &self.search_replace {
            Some(state) => (state.search_text.clone(), state.replace_text.clone()),
            None => (String::new(), String::new()),
        };

        let search_replace_view =
            SearchReplaceView::new(popup_rect, &search_text, &replace_text, context);
        rq.add(RenderData::new(
            search_replace_view.id(),
            popup_rect,
            UpdateMode::Gui,
        ));
        self.children
            .push(Box::new(search_replace_view) as Box<dyn View>);
        hub.send(Event::Focus(Some(ViewId::EpubEditorSearchInput)))
            .ok();
    }

    fn do_search(&mut self, rq: &mut RenderQueue, _context: &mut Context) {
        if let Some(state) = &self.search_replace {
            if state.search_text.is_empty() {
                return;
            }
            if let EditorState::EditingChapter { index } = self.state {
                let matches = self.core.search_in_chapter(index, &state.search_text);
                if let Some(view) = self
                    .children
                    .iter_mut()
                    .find(|c| c.is::<SearchReplaceView>())
                {
                    if let Some(sr_view) = view.downcast_mut::<SearchReplaceView>() {
                        sr_view.update_matches(matches.len(), rq);
                    }
                }
            }
        }
    }

    fn do_replace_in_chapter(&mut self, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        if let Some(state) = &self.search_replace {
            if state.search_text.is_empty() {
                return;
            }
            let search_text = state.search_text.clone();
            if let EditorState::EditingChapter { index } = self.state {
                let _old_content = self.core.chapters[index].content.clone();
                match self
                    .core
                    .replace_in_chapter(index, &search_text, &state.replace_text)
                {
                    Ok(count) => {
                        if count > 0 {
                            self.modified = true;
                            self.update_input_field(rq, context);
                            let notif = Notification::new(
                                format!("Replaced {} occurrence(s)", count),
                                hub,
                                rq,
                                context,
                            );
                            self.children.push(Box::new(notif) as Box<dyn View>);
                            let matches = self.core.search_in_chapter(index, &search_text);
                            if let Some(view) = self
                                .children
                                .iter_mut()
                                .find(|c| c.is::<SearchReplaceView>())
                            {
                                if let Some(sr_view) = view.downcast_mut::<SearchReplaceView>() {
                                    sr_view.update_matches(matches.len(), rq);
                                }
                            }
                        } else {
                            let notif =
                                Notification::new("No matches found".to_string(), hub, rq, context);
                            self.children.push(Box::new(notif) as Box<dyn View>);
                        }
                    }
                    Err(e) => {
                        let notif =
                            Notification::new(format!("Replace error: {}", e), hub, rq, context);
                        self.children.push(Box::new(notif) as Box<dyn View>);
                    }
                }
            }
        }
    }

    fn update_input_field(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if let EditorState::EditingChapter { index } = self.state {
            if index < self.core.chapters.len() {
                let content: String = self.core.chapters[index]
                    .content
                    .chars()
                    .take(5000)
                    .collect();
                if let Some(view) = self.children.iter_mut().find(|c| c.is::<InputField>()) {
                    if let Some(input) = view.downcast_mut::<InputField>() {
                        input.set_text(&content, true, rq, context);
                    }
                }
            }
        }
    }

    fn close_search_replace(&mut self, rq: &mut RenderQueue) {
        self.children.retain(|c| !c.is::<SearchReplaceView>());
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
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
                            self.show_save_dialog(hub, rq, context);
                        } else {
                            self.show_chapter_list(hub, rq, context);
                        }
                        return true;
                    }
                    EditorState::ChapterList => {
                        if self.modified {
                            self.show_save_dialog(hub, rq, context);
                        }
                    }
                }
                false
            }
            Event::Select(EntryId::SelectChapter(i)) => {
                self.show_edit_view(*i, hub, rq, context);
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
                false
            }
            Event::Select(EntryId::Discard) => {
                self.modified = false;
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
                    search_text: String::new(),
                    replace_text: String::new(),
                });
                self.show_search_replace(hub, rq, context);
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
                self.do_search(rq, context);
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
                self.do_replace_in_chapter(hub, rq, context);
                true
            }
            Event::Select(EntryId::CloseSearchReplace) => {
                self.search_replace = None;
                self.close_search_replace(rq);
                true
            }
            Event::Close(ViewId::EpubEditor) => {
                if self.search_replace.is_some() {
                    self.search_replace = None;
                    self.close_search_replace(rq);
                    true
                } else {
                    false
                }
            }
            Event::Submit(ViewId::EpubEditorSearchInput, text) => {
                if let Some(state) = self.search_replace.as_mut() {
                    state.search_text = text.clone();
                }
                self.do_search(rq, context);
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
