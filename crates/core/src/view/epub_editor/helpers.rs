//! Helper functions for EPUB editor view.

use crate::color;
use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::{halves, Rectangle};
use crate::theme;
use crate::unit::scale_by_dpi;
use crate::view::button::Button;
use crate::view::filler::Filler;
use crate::view::icon::Icon;
use crate::view::input_field::InputField;
use crate::view::keyboard::Keyboard;
use crate::view::label::Label;
use crate::view::menu::{Menu, MenuKind};
use crate::view::notification::Notification;
use crate::view::top_bar::TopBar;
use crate::view::{Align, EntryId, EntryKind, Event, Hub, RenderData, RenderQueue, View, ViewId, ID_FEEDER};
use crate::view::{SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};
use epub_edit::EpubEditorCore;
use super::state::EditorState;

/// Show the chapter list view.
pub fn show_chapter_list(
    editor: &mut super::EpubEditor,
    _hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    editor.state = EditorState::ChapterList;
    editor.children
        .retain(|c| !c.is::<Menu>() && !c.is::<Notification>());

    let entries: Vec<EntryKind> = editor
        .core
        .chapters
        .iter()
        .enumerate()
        .map(|(i, chapter)| {
            let title = if editor.modified_chapters.contains(&i) {
                format!("* {}", chapter.title)
            } else {
                chapter.title.clone()
            };
            EntryKind::Command(title, EntryId::SelectChapter(i))
        })
        .collect();

    let dpi = crate::unit::get_device_dpi();
    let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
    let rect = rect![
        editor.rect.min.x,
        editor.rect.min.y + small_height + 1,
        editor.rect.max.x,
        editor.rect.max.y
    ];

    let menu = Menu::new(
        rect,
        ViewId::BookMenu,
        MenuKind::Contextual,
        entries,
        context,
    );
    rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
    editor.children.push(Box::new(menu) as Box<dyn View>);
}

/// Show the metadata edit view.
pub fn show_metadata_edit_view(
    editor: &mut super::EpubEditor,
    _hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    editor.children
        .retain(|c| !c.is::<Menu>() && !c.is::<Notification>());

    let dpi = crate::unit::get_device_dpi();
    let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
    let row_height = scale_by_dpi(40.0, dpi) as i32;
    let padding = scale_by_dpi(10.0, dpi) as i32;
    let label_width = scale_by_dpi(80.0, dpi) as i32;

    let mut y = editor.rect.min.y + small_height + 10;
    let meta = &editor.core.metadata;

    let fields = vec![
        ("Title", meta.title.clone(), ViewId::EditMetadataTitle),
        ("Author", meta.author.clone(), ViewId::EditMetadataAuthor),
        (
            "Language",
            meta.language.clone(),
            ViewId::EditMetadataLanguage,
        ),
        (
            "Identifier",
            meta.identifier.clone(),
            ViewId::EditMetadataIdentifier,
        ),
        (
            "Publisher",
            meta.publisher.clone().unwrap_or_default(),
            ViewId::EditMetadataPublisher,
        ),
        (
            "Date",
            meta.date.clone().unwrap_or_default(),
            ViewId::EditMetadataDate,
        ),
    ];

    for (label, value, view_id) in fields {
        let label_rect = rect![
            editor.rect.min.x + padding,
            y,
            editor.rect.min.x + padding + label_width,
            y + row_height
        ];
        let label_view = Label::new(label_rect, label.to_string(), Align::Left(0));
        editor.children.push(Box::new(label_view) as Box<dyn View>);

        let input_rect = rect![
            editor.rect.min.x + padding + label_width,
            y,
            editor.rect.max.x - padding,
            y + row_height
        ];
        let input = InputField::new(input_rect, view_id)
            .border(true)
            .text(&value, context);
        editor.children.push(Box::new(input) as Box<dyn View>);

        y += row_height + padding;
    }

    let save_rect = rect![
        editor.rect.min.x + padding,
        y,
        editor.rect.min.x + padding + scale_by_dpi(100.0, dpi) as i32,
        y + row_height
    ];
    let save_btn = Button::new(
        save_rect,
        Event::Select(EntryId::SaveMetadata),
        "Save".to_string(),
    );
    editor.children.push(Box::new(save_btn) as Box<dyn View>);

    rq.add(RenderData::new(editor.id, editor.rect, UpdateMode::Gui));
}

/// Show the save dialog.
pub fn show_save_dialog(
    editor: &mut super::EpubEditor,
    _hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    editor.children
        .retain(|c| !c.is::<Menu>() && !c.is::<Notification>());

    let entries = vec![
        EntryKind::Command("Save".to_string(), EntryId::Save),
        EntryKind::Command("Discard".to_string(), EntryId::Discard),
    ];

    let dpi = crate::unit::get_device_dpi();
    let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
    let rect = rect![
        editor.rect.min.x,
        editor.rect.min.y + small_height + 10,
        editor.rect.max.x,
        editor.rect.min.y + small_height + 120
    ];

    let menu = Menu::new(
        rect,
        ViewId::BookMenu,
        MenuKind::Contextual,
        entries,
        context,
    );
    rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
    editor.children.push(Box::new(menu) as Box<dyn View>);
}

/// Show the edit view for a specific chapter.
pub fn show_edit_view(
    editor: &mut super::EpubEditor,
    index: usize,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    if index >= editor.core.chapters.len() {
        return;
    }

    editor.state = EditorState::EditingChapter { index };
    editor.children
        .retain(|c| !c.is::<Menu>() && !c.is::<Notification>());

    let dpi = crate::unit::get_device_dpi();
    let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
    let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
    let (small_thickness, _) = halves(thickness);
    let row_height = scale_by_dpi(32.0, dpi) as i32;

    let chapter = &editor.core.chapters[index];
    let content = chapter.content.clone();

    let title_label = Label::new(
        rect![
            editor.rect.min.x + 10,
            editor.rect.min.y + small_height + 10,
            editor.rect.max.x - 10,
            editor.rect.min.y + small_height + 40
        ],
        format!("Editing: {}", chapter.title),
        Align::Left(0),
    );
    editor.children.push(Box::new(title_label) as Box<dyn View>);

    let textarea_rect = rect![
        editor.rect.min.x + 10,
        editor.rect.min.y + small_height + 50,
        editor.rect.max.x - 10,
        editor.rect.max.y - small_height - 60
    ];

    let input_field =
        InputField::new(textarea_rect, ViewId::EditNoteInput).text(&content, context);
    editor.children.push(Box::new(input_field) as Box<dyn View>);

    let sep_rect = rect![
        editor.rect.min.x,
        editor.rect.max.y - small_height - small_thickness,
        editor.rect.max.x,
        editor.rect.max.y - small_height
    ];
    let separator = Filler::new(sep_rect, crate::color::foreground(theme::is_dark_mode()));
    editor.children.push(Box::new(separator) as Box<dyn View>);

    let nav_btn_width = (editor.rect.width() as i32) / 2;
    let prev_btn = Button::new(
        rect![
            editor.rect.min.x,
            editor.rect.max.y - small_height - small_thickness - row_height,
            editor.rect.min.x + nav_btn_width,
            editor.rect.max.y - small_height - small_thickness
        ],
        Event::Select(EntryId::PreviousChapter),
        "Previous".to_string(),
    );
    editor.children.push(Box::new(prev_btn) as Box<dyn View>);

    let next_btn = Button::new(
        rect![
            editor.rect.min.x + nav_btn_width,
            editor.rect.max.y - small_height - small_thickness - row_height,
            editor.rect.max.x,
            editor.rect.max.y - small_height - small_thickness
        ],
        Event::Select(EntryId::NextChapter),
        "Next".to_string(),
    );
    editor.children.push(Box::new(next_btn) as Box<dyn View>);

    let kb_rect = rect![
        editor.rect.min.x,
        editor.rect.max.y - small_height,
        editor.rect.max.x,
        editor.rect.max.y
    ];

    let mut kb_rect_mut = kb_rect;
    let keyboard = Keyboard::new(&mut kb_rect_mut, true, context);
    editor.children.push(Box::new(keyboard) as Box<dyn View>);

    rq.add(RenderData::new(editor.id, editor.rect, UpdateMode::Gui));
}
