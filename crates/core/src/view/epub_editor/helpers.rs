//! Helper functions for EPUB editor view.

use super::state::EditorState;
use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::halves;
use crate::theme;
use crate::unit::scale_by_dpi;
use crate::view::button::Button;
use crate::view::filler::Filler;
use crate::view::input_field::InputField;
use crate::view::keyboard::Keyboard;
use crate::view::label::Label;
use crate::view::menu::{Menu, MenuKind};
use crate::view::notification::Notification;
use crate::view::search_replace::SearchReplaceView;
use crate::view::{
    Align, EntryId, EntryKind, Event, Hub, RenderData, RenderQueue, View, ViewId, SMALL_BAR_HEIGHT,
    THICKNESS_MEDIUM,
};
use epub_edit::SearchOptions;
use std::fs;

/// Show the chapter list view.
pub fn show_chapter_list(
    editor: &mut super::EpubEditor,
    _hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    editor.state = EditorState::ChapterList;
    editor
        .children
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
    editor
        .children
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
    editor
        .children
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
    _hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    if index >= editor.core.chapters.len() {
        return;
    }

    editor.state = EditorState::EditingChapter { index };
    editor
        .children
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

    let input_field = InputField::new(textarea_rect, ViewId::EditNoteInput).text(&content, context);
    editor.children.push(Box::new(input_field) as Box<dyn View>);

    let _menu_rect = rect![
        editor.rect.min.x,
        editor.rect.max.y - small_height - small_thickness,
        editor.rect.max.x,
        editor.rect.max.y - small_height
    ];

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

/// Show the search/replace dialog.
pub fn show_search_replace(
    editor: &mut super::EpubEditor,
    _hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    editor.children.retain(|c| !c.is::<SearchReplaceView>());

    let dpi = crate::unit::get_device_dpi();
    let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
    let popup_height = 160;
    let popup_rect = rect![
        editor.rect.min.x + 20,
        editor.rect.min.y + small_height + 10,
        editor.rect.max.x - 20,
        editor.rect.min.y + small_height + 10 + popup_height
    ];

    let (search_text, replace_text) = match &editor.search_replace {
        Some(state) => (state.search_text.clone(), state.replace_text.clone()),
        None => (String::with_capacity(32), String::with_capacity(32)),
    };

    let search_replace_view =
        SearchReplaceView::new(popup_rect, &search_text, &replace_text, context);
    rq.add(RenderData::new(
        search_replace_view.id(),
        popup_rect,
        UpdateMode::Gui,
    ));
    editor
        .children
        .push(Box::new(search_replace_view) as Box<dyn View>);
}

/// Perform search in the current chapter.
pub fn do_search(editor: &mut super::EpubEditor, rq: &mut RenderQueue, _context: &mut Context) {
    if let Some(state) = &editor.search_replace {
        if state.search_text.is_empty() {
            return;
        }
        if let EditorState::EditingChapter { index } = editor.state {
            let options = editor
                .children
                .iter()
                .find(|c| c.is::<SearchReplaceView>())
                .and_then(|v| v.downcast_ref::<SearchReplaceView>())
                .map(|sr| {
                    let (use_regex, case_sensitive, whole_word) = sr.get_search_options();
                    SearchOptions {
                        use_regex,
                        case_sensitive,
                        whole_word,
                    }
                })
                .unwrap_or_default();
            let matches = editor
                .core
                .search_in_chapter(index, &state.search_text, options);
            if let Some(view) = editor
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

/// Perform replace in the current chapter.
pub fn do_replace_in_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    if let Some(state) = &editor.search_replace {
        if state.search_text.is_empty() {
            return;
        }
        let search_text = state.search_text.clone();
        let options = editor
            .children
            .iter()
            .find(|c| c.is::<SearchReplaceView>())
            .and_then(|v| v.downcast_ref::<SearchReplaceView>())
            .map(|sr| {
                let (use_regex, case_sensitive, whole_word) = sr.get_search_options();
                SearchOptions {
                    use_regex,
                    case_sensitive,
                    whole_word,
                }
            })
            .unwrap_or_default();
        if let EditorState::EditingChapter { index } = editor.state {
            match editor
                .core
                .replace_in_chapter(index, &search_text, &state.replace_text, options)
            {
                Ok(count) => {
                    if count > 0 {
                        editor.modified = true;
                        update_input_field(editor, rq, context);
                        let notif = Notification::new(
                            format!("Replaced {} occurrence(s)", count),
                            hub,
                            rq,
                            context,
                        );
                        editor.children.push(Box::new(notif) as Box<dyn View>);
                        let matches = editor.core.search_in_chapter(index, &search_text, options);
                        if let Some(view) = editor
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
                        editor.children.push(Box::new(notif) as Box<dyn View>);
                    }
                }
                Err(e) => {
                    let notif =
                        Notification::new(format!("Error replacing: {}", e), hub, rq, context);
                    editor.children.push(Box::new(notif) as Box<dyn View>);
                }
            }
        }
    }
}

/// Update the input field with current chapter content.
pub fn update_input_field(
    editor: &mut super::EpubEditor,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    if let EditorState::EditingChapter { index } = editor.state {
        if index < editor.core.chapters.len() {
            let content = editor.core.chapters[index].content.clone();
            if let Some(view) = editor.children.iter_mut().find(|c| c.is::<InputField>()) {
                if let Some(input) = view.downcast_mut::<InputField>() {
                    input.set_text(&content, true, rq, context);
                }
            }
        }
    }
}

/// Close the search/replace dialog.
pub fn close_search_replace(editor: &mut super::EpubEditor, rq: &mut RenderQueue) {
    editor.children.retain(|c| !c.is::<SearchReplaceView>());
    rq.add(RenderData::new(editor.id, editor.rect, UpdateMode::Gui));
}

/// Show a single-line input field pre-filled with the current chapter title for renaming (BUG-2).
///
/// The submitted text arrives as `Event::Submit(ViewId::RenameDocumentInput, text)`.
pub fn show_rename_input(
    editor: &mut super::EpubEditor,
    _hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    let EditorState::EditingChapter { index } = editor.state else {
        return;
    };
    if index >= editor.core.chapters.len() {
        return;
    }

    // Remove any stale rename inputs.
    editor
        .children
        .retain(|c| !c.is::<InputField>() || c.view_id() != Some(ViewId::RenameDocumentInput));

    let current_title = editor.core.chapters[index].title.clone();
    let dpi = crate::unit::get_device_dpi();
    let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
    let row_height = scale_by_dpi(40.0, dpi) as i32;
    let padding = scale_by_dpi(10.0, dpi) as i32;

    // Label
    let label_rect = rect![
        editor.rect.min.x + padding,
        editor.rect.min.y + small_height + padding,
        editor.rect.max.x - padding,
        editor.rect.min.y + small_height + padding + row_height
    ];
    editor.children.push(Box::new(Label::new(
        label_rect,
        "Rename chapter:".to_string(),
        Align::Left(0),
    )) as Box<dyn View>);

    // Pre-filled input field
    let input_rect = rect![
        editor.rect.min.x + padding,
        editor.rect.min.y + small_height + padding + row_height,
        editor.rect.max.x - padding,
        editor.rect.min.y + small_height + padding + row_height * 2
    ];
    let input = InputField::new(input_rect, ViewId::RenameDocumentInput)
        .border(true)
        .text(&current_title, context);
    editor.children.push(Box::new(input) as Box<dyn View>);

    // Keyboard
    let mut kb_rect = rect![
        editor.rect.min.x,
        editor.rect.max.y - small_height,
        editor.rect.max.x,
        editor.rect.max.y
    ];
    let keyboard = Keyboard::new(&mut kb_rect, true, context);
    editor.children.push(Box::new(keyboard) as Box<dyn View>);

    rq.add(RenderData::new(editor.id, editor.rect, UpdateMode::Gui));
}

/// Scan the library's `Downloads/` directory for importable files and show them as a menu (BUG-3).
///
/// Supports `.xhtml`, `.html`, and `.txt` files.
/// If no files are found, a notification is shown instead.
/// The selected file path arrives as `Event::Select(EntryId::ImportChapterFile(path))`.
pub fn show_import_chapter_menu(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    let library_path = context.settings.libraries[context.settings.selected_library]
        .path
        .clone();
    let downloads_path = library_path.join("Downloads");

    let importable: Vec<std::path::PathBuf> = fs::read_dir(&downloads_path)
        .map(|dir| {
            dir.flatten()
                .map(|e| e.path())
                .filter(|p| {
                    matches!(
                        p.extension().and_then(|s| s.to_str()),
                        Some("xhtml" | "html" | "txt")
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    if importable.is_empty() {
        let notif = Notification::new(
            format!("No importable files in {}", downloads_path.display()),
            hub,
            rq,
            context,
        );
        editor.children.push(Box::new(notif) as Box<dyn View>);
        return;
    }

    let entries: Vec<EntryKind> = importable
        .iter()
        .map(|p| {
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            EntryKind::Command(
                name,
                EntryId::ImportChapterFile(p.to_string_lossy().into_owned()),
            )
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
