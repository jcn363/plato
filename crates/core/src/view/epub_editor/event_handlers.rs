//! Event handlers for EPUB Editor

use rustc_hash::FxHashSet;
use std::path::Path;

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::view::common::toggle_main_menu;
use crate::view::input_field::InputField;
use crate::view::notification::Notification;
use crate::view::search_replace::SearchReplaceView;
use crate::view::{Bus, EntryId, Event, Hub, RenderData, RenderQueue, View, ViewId};

use super::helpers::{
    close_search_replace, do_replace_in_chapter, do_search, show_chapter_list, show_edit_view,
    show_metadata_edit_view, show_save_dialog, show_search_replace, update_input_field,
};
use super::state::EditorState;

/// Handle events for the EPUB Editor
pub fn handle_event(
    editor: &mut super::EpubEditor,
    event: &Event,
    hub: &Hub,
    bus: &mut Bus,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    match event {
        Event::Back => handle_back(editor, hub, rq, context),
        Event::Select(EntryId::SelectChapter(i)) => {
            show_edit_view(editor, *i, hub, rq, context);
            true
        }
        Event::Select(EntryId::EditMetadata) => {
            show_metadata_edit_view(editor, hub, rq, context);
            true
        }
        Event::Select(EntryId::PreviousChapter) => handle_previous_chapter(editor, hub, rq, context),
        Event::Select(EntryId::NextChapter) => handle_next_chapter(editor, hub, rq, context),
        Event::Select(EntryId::ToggleRegex) => handle_toggle_regex(editor, rq),
        Event::Select(EntryId::ToggleCaseSensitive) => handle_toggle_case_sensitive(editor, rq),
        Event::Select(EntryId::ToggleWholeWord) => handle_toggle_whole_word(editor, rq),
        Event::Select(EntryId::SaveMetadata) => handle_save_metadata(editor, hub, rq, context),
        Event::Select(EntryId::Save) => handle_save(editor, hub, rq, context),
        Event::Select(EntryId::Discard) => handle_discard(editor),
        Event::Submit(ViewId::EditNoteInput, text) => handle_submit_edit_note(editor, text, hub, rq, context),
        Event::ToggleNear(ViewId::MainMenu, rect) => {
            toggle_main_menu(editor, *rect, None, rq, context);
            true
        }
        Event::Select(EntryId::Undo) => handle_undo(editor, bus, rq),
        Event::Select(EntryId::Redo) => handle_redo(editor, bus, rq),
        Event::Select(EntryId::Preview) => handle_preview(editor, bus),
        Event::Select(EntryId::SearchReplace) => handle_search_replace_init(editor, hub, rq, context),
        Event::SearchReplace => handle_search_replace(editor, rq, context),
        Event::Select(EntryId::ReplaceInChapter) => handle_replace_in_chapter(editor, hub, rq, context),
        Event::Select(EntryId::ReplaceInDocument) => handle_replace_in_document(editor, hub, rq, context),
        Event::Select(EntryId::CloseSearchReplace) => handle_close_search_replace(editor, rq),
        Event::Select(EntryId::ValidateContent) => handle_validate_content(editor, hub, rq, context),
        Event::Select(EntryId::RenameChapter) => handle_rename_chapter(editor, hub, rq, context),
        Event::Select(EntryId::DeleteChapter) => handle_delete_chapter(editor, hub, rq, context),
        Event::Select(EntryId::MoveChapterUp) => handle_move_chapter_up(editor, hub, rq, context),
        Event::Select(EntryId::MoveChapterDown) => handle_move_chapter_down(editor, hub, rq, context),
        Event::Select(EntryId::SpellCheck) => handle_spell_check(editor, hub, rq, context),
        Event::Select(EntryId::ExportChapter) => handle_export_chapter(editor, hub, rq, context),
        Event::Select(EntryId::ImportChapter) => handle_import_chapter(editor, hub, rq, context),
        Event::Select(EntryId::ChapterStatistics) => handle_chapter_statistics(editor, hub, rq, context),
        Event::Select(EntryId::GenerateTOC) => handle_generate_toc(editor, hub, rq, context),
        Event::Select(EntryId::ListImages) => handle_list_images(editor, hub, rq, context),
        Event::Select(EntryId::ClearHistory) => handle_clear_history(editor, hub, rq, context),
        Event::Select(EntryId::ListCSS) => handle_list_css(editor, hub, rq, context),
        Event::Select(EntryId::AddBookmark) => handle_add_bookmark(editor, hub, rq, context),
        Event::Select(EntryId::ReplaceAllInAllDocuments) => {
            handle_replace_all_in_all_documents(editor, hub, rq, context)
        }
        Event::Close(ViewId::EpubEditor) => handle_close(editor, rq),
        Event::Submit(ViewId::EpubEditorSearchInput, text) => {
            handle_submit_search_input(editor, text, rq, context)
        }
        Event::Submit(ViewId::EpubEditorReplaceInput, text) => {
            handle_submit_replace_input(editor, text)
        }
        _ => {
            for child in editor.children_mut().iter_mut() {
                if child.handle_event(event, hub, bus, rq, context) {
                    return true;
                }
            }
            false
        }
    }
}

fn handle_back(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    match editor.state {
        EditorState::EditingChapter { .. } => {
            if editor.modified {
                show_save_dialog(editor, hub, rq, context);
            } else {
                show_chapter_list(editor, hub, rq, context);
            }
            true
        }
        EditorState::ChapterList => {
            if editor.modified {
                show_save_dialog(editor, hub, rq, context);
            }
            false
        }
    }
}

fn handle_previous_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let EditorState::EditingChapter { index } = editor.state {
        if index > 0 {
            show_edit_view(editor, index - 1, hub, rq, context);
        }
    }
    true
}

fn handle_next_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let EditorState::EditingChapter { index } = editor.state {
        if index + 1 < editor.core.chapters.len() {
            show_edit_view(editor, index + 1, hub, rq, context);
        }
    }
    true
}

fn handle_toggle_regex(editor: &mut super::EpubEditor, rq: &mut RenderQueue) -> bool {
    if let Some(sr) = editor
        .children
        .iter_mut()
        .find(|c| c.is::<SearchReplaceView>())
    {
        if let Some(view) = sr.downcast_mut::<SearchReplaceView>() {
            view.toggle_regex();
            rq.add(RenderData::new(editor.id, editor.rect, UpdateMode::Gui));
        }
    }
    true
}

fn handle_toggle_case_sensitive(editor: &mut super::EpubEditor, rq: &mut RenderQueue) -> bool {
    if let Some(sr) = editor
        .children
        .iter_mut()
        .find(|c| c.is::<SearchReplaceView>())
    {
        if let Some(view) = sr.downcast_mut::<SearchReplaceView>() {
            view.toggle_case_sensitive();
            rq.add(RenderData::new(editor.id, editor.rect, UpdateMode::Gui));
        }
    }
    true
}

fn handle_toggle_whole_word(editor: &mut super::EpubEditor, rq: &mut RenderQueue) -> bool {
    if let Some(sr) = editor
        .children
        .iter_mut()
        .find(|c| c.is::<SearchReplaceView>())
    {
        if let Some(view) = sr.downcast_mut::<SearchReplaceView>() {
            view.toggle_whole_word();
            rq.add(RenderData::new(editor.id, editor.rect, UpdateMode::Gui));
        }
    }
    true
}

fn handle_save_metadata(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    let mut new_meta = editor.core.metadata.clone();
    if let Some(view) = editor.children.iter().find(|c| c.is::<InputField>()) {
        if let Some(input) = view.downcast_ref::<InputField>() {
            if input.view_id() == Some(ViewId::EditMetadataTitle) {
                new_meta.title = input.get_text().to_string();
            }
        }
    }
    if let Some(view) = editor.children.iter().find(|c| {
        c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataAuthor)
    }) {
        if let Some(input) = view.downcast_ref::<InputField>() {
            new_meta.author = input.get_text().to_string();
        }
    }
    if let Some(view) = editor.children.iter().find(|c| {
        c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataLanguage)
    }) {
        if let Some(input) = view.downcast_ref::<InputField>() {
            new_meta.language = input.get_text().to_string();
        }
    }
    if let Some(view) = editor.children.iter().find(|c| {
        c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataIdentifier)
    }) {
        if let Some(input) = view.downcast_ref::<InputField>() {
            new_meta.identifier = input.get_text().to_string();
        }
    }
    if let Some(view) = editor.children.iter().find(|c| {
        c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataPublisher)
    }) {
        if let Some(input) = view.downcast_ref::<InputField>() {
            new_meta.publisher = Some(input.get_text().to_string());
        }
    }
    if let Some(view) = editor
        .children
        .iter()
        .find(|c| c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataDate))
    {
        if let Some(input) = view.downcast_ref::<InputField>() {
            new_meta.date = Some(input.get_text().to_string());
        }
    }
    editor.core.set_metadata(new_meta);
    editor.modified = true;
    let notif = Notification::new("Metadata updated".to_string(), hub, rq, context);
    editor.children.push(Box::new(notif) as Box<dyn View>);
    show_chapter_list(editor, hub, rq, context);
    true
}

fn handle_save(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let Err(e) = editor.core.save() {
        let notif = Notification::new(format!("Error saving: {}", e), hub, rq, context);
        editor.children.push(Box::new(notif) as Box<dyn View>);
    } else {
        let notif = Notification::new("Changes saved!".to_string(), hub, rq, context);
        editor.children.push(Box::new(notif) as Box<dyn View>);
    }
    editor.modified = false;
    editor.modified_chapters.clear();
    false
}

fn handle_discard(editor: &mut super::EpubEditor) -> bool {
    editor.modified = false;
    editor.modified_chapters.clear();
    false
}

fn handle_submit_edit_note(
    editor: &mut super::EpubEditor,
    text: &str,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let EditorState::EditingChapter { index } = editor.state {
        if editor.update_chapter_content(index, text.to_string(), rq) {
            let notif = Notification::new(
                format!("Chapter {} saved!", editor.core.chapters[index].title),
                hub,
                rq,
                context,
            );
            editor.children.push(Box::new(notif) as Box<dyn View>);
        }
    }
    true
}

fn handle_undo(editor: &mut super::EpubEditor, bus: &mut Bus, rq: &mut RenderQueue) -> bool {
    if editor.undo(rq) {
        bus.push_back(Event::Render("Undone".to_string()));
    }
    true
}

fn handle_redo(editor: &mut super::EpubEditor, bus: &mut Bus, rq: &mut RenderQueue) -> bool {
    if editor.redo(rq) {
        bus.push_back(Event::Render("Redone".to_string()));
    }
    true
}

fn handle_preview(editor: &mut super::EpubEditor, bus: &mut Bus) -> bool {
    if let EditorState::EditingChapter { index } = editor.state {
        bus.push_back(Event::Render(format!(
            "Preview: {}",
            editor.core.chapters[index].title
        )));
    }
    true
}

fn handle_search_replace_init(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    editor.search_replace = Some(super::state::SearchReplaceState {
        search_text: String::with_capacity(32),
        replace_text: String::with_capacity(32),
    });
    show_search_replace(editor, hub, rq, context);
    true
}

fn handle_search_replace(
    editor: &mut super::EpubEditor,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let Some(state) = editor.search_replace.as_mut() {
        if let Some(view) = editor.children.iter().find(|c| c.is::<SearchReplaceView>()) {
            if let Some(sr_view) = view.downcast_ref::<SearchReplaceView>() {
                state.search_text = sr_view.get_search_text().to_string();
                state.replace_text = sr_view.get_replace_text().to_string();
            }
        }
    }
    do_search(editor, rq, context);
    true
}

fn handle_replace_in_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let Some(state) = editor.search_replace.as_mut() {
        if let Some(view) = editor.children.iter().find(|c| c.is::<SearchReplaceView>()) {
            if let Some(sr_view) = view.downcast_ref::<SearchReplaceView>() {
                state.search_text = sr_view.get_search_text().to_string();
                state.replace_text = sr_view.get_replace_text().to_string();
            }
        }
    }
    do_replace_in_chapter(editor, hub, rq, context);
    true
}

fn handle_replace_in_document(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let Some(state) = editor.search_replace.as_mut() {
        if let Some(view) = editor.children.iter().find(|c| c.is::<SearchReplaceView>()) {
            if let Some(sr_view) = view.downcast_ref::<SearchReplaceView>() {
                state.search_text = sr_view.get_search_text().to_string();
                state.replace_text = sr_view.get_replace_text().to_string();
            }
        }
    }
    if let Some(state) = &editor.search_replace {
        if state.search_text.is_empty() {
            let notif =
                Notification::new("Search text is empty".to_string(), hub, rq, context);
            editor.children.push(Box::new(notif) as Box<dyn View>);
            return true;
        }
        let options = editor
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
        match editor.core.replace_all_in_document(
            &state.search_text,
            &state.replace_text,
            options,
        ) {
            Ok(count) => {
                if count > 0 {
                    editor.modified = true;
                    let notif = Notification::new(
                        format!("Replaced {} occurrence(s) in document", count),
                        hub,
                        rq,
                        context,
                    );
                    editor.children.push(Box::new(notif) as Box<dyn View>);
                    if let EditorState::EditingChapter { index: _ } = editor.state {
                        update_input_field(editor, rq, context);
                    }
                } else {
                    let notif = Notification::new(
                        "No matches found in document".to_string(),
                        hub,
                        rq,
                        context,
                    );
                    editor.children.push(Box::new(notif) as Box<dyn View>);
                }
            }
            Err(e) => {
                let notif = Notification::new(
                    format!("Replace error: {}", e),
                    hub,
                    rq,
                    context,
                );
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
        }
    }
    true
}

fn handle_close_search_replace(editor: &mut super::EpubEditor, rq: &mut RenderQueue) -> bool {
    editor.search_replace = None;
    close_search_replace(editor, rq);
    true
}

fn handle_validate_content(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    let result = editor.core.validate_content();
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
        editor.children.push(Box::new(notif) as Box<dyn View>);
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
        editor.children.push(Box::new(notif) as Box<dyn View>);
    }
    true
}

fn handle_rename_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let EditorState::EditingChapter { index: _ } = editor.state {
        let notif = Notification::new(
            "Chapter rename feature - UI input needed".to_string(),
            hub,
            rq,
            context,
        );
        editor.children.push(Box::new(notif) as Box<dyn View>);
    }
    true
}

fn handle_delete_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let EditorState::EditingChapter { index } = editor.state {
        match editor.core.delete_chapter(index) {
            Ok(_) => {
                editor.modified = true;
                let notif =
                    Notification::new("Chapter deleted".to_string(), hub, rq, context);
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
            Err(e) => {
                let notif = Notification::new(
                    format!("Error deleting chapter: {}", e),
                    hub,
                    rq,
                    context,
                );
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
        }
    }
    true
}

fn handle_move_chapter_up(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let EditorState::EditingChapter { index } = editor.state {
        if index > 0 {
            match editor.core.reorder_chapters(index, index - 1) {
                Ok(_) => {
                    editor.modified = true;
                    let notif = Notification::new(
                        "Chapter moved up".to_string(),
                        hub,
                        rq,
                        context,
                    );
                    editor.children.push(Box::new(notif) as Box<dyn View>);
                }
                Err(e) => {
                    let notif = Notification::new(
                        format!("Error moving chapter: {}", e),
                        hub,
                        rq,
                        context,
                    );
                    editor.children.push(Box::new(notif) as Box<dyn View>);
                }
            }
        }
    }
    true
}

fn handle_move_chapter_down(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let EditorState::EditingChapter { index } = editor.state {
        if index < editor.core.chapters.len() - 1 {
            match editor.core.reorder_chapters(index, index + 1) {
                Ok(_) => {
                    editor.modified = true;
                    let notif = Notification::new(
                        "Chapter moved down".to_string(),
                        hub,
                        rq,
                        context,
                    );
                    editor.children.push(Box::new(notif) as Box<dyn View>);
                }
                Err(e) => {
                    let notif = Notification::new(
                        format!("Error moving chapter: {}", e),
                        hub,
                        rq,
                        context,
                    );
                    editor.children.push(Box::new(notif) as Box<dyn View>);
                }
            }
        }
    }
    true
}

fn handle_spell_check(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    let result = editor.core.spell_check();
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
        editor.children.push(Box::new(notif) as Box<dyn View>);
    } else {
        let notif = Notification::new(
            format!(
                "Found {} potential spelling errors in {} chapters",
                result.errors.len(),
                result
                    .errors
                    .iter()
                    .map(|e| e.chapter_index)
                    .collect::<FxHashSet<_>>()
                    .len()
            ),
            hub,
            rq,
            context,
        );
        editor.children.push(Box::new(notif) as Box<dyn View>);
    }
    true
}

fn handle_export_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let EditorState::EditingChapter { index } = editor.state {
        let export_path = format!("/tmp/chapter_{}.txt", index);
        let path = Path::new(&export_path);
        match editor.core.export_chapter(index, path) {
            Ok(_) => {
                let notif = Notification::new(
                    format!("Chapter exported to {}", export_path),
                    hub,
                    rq,
                    context,
                );
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
            Err(e) => {
                let notif = Notification::new(
                    format!("Error exporting chapter: {}", e),
                    hub,
                    rq,
                    context,
                );
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
        }
    }
    true
}

fn handle_import_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let EditorState::EditingChapter { index: _ } = editor.state {
        let notif = Notification::new(
            "Chapter import - file path selection needed".to_string(),
            hub,
            rq,
            context,
        );
        editor.children.push(Box::new(notif) as Box<dyn View>);
    }
    true
}

fn handle_chapter_statistics(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let EditorState::EditingChapter { index } = editor.state {
        if let Some(stats) = editor.core.get_chapter_statistics(index) {
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
            editor.children.push(Box::new(notif) as Box<dyn View>);
        }
    }
    true
}

fn handle_generate_toc(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    match editor.core.update_table_of_contents() {
        Ok(_) => {
            editor.modified = true;
            let notif = Notification::new(
                format!(
                    "Table of contents generated for {} chapters",
                    editor.core.chapters.len()
                ),
                hub,
                rq,
                context,
            );
            editor.children.push(Box::new(notif) as Box<dyn View>);
        }
        Err(e) => {
            let notif = Notification::new(
                format!("Error generating table of contents: {}", e),
                hub,
                rq,
                context,
            );
            editor.children.push(Box::new(notif) as Box<dyn View>);
        }
    }
    true
}

fn handle_list_images(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    let images = editor.core.list_images();
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
    editor.children.push(Box::new(notif) as Box<dyn View>);
    true
}

fn handle_clear_history(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    editor.core.clear_history();
    let notif =
        Notification::new("Undo/redo history cleared".to_string(), hub, rq, context);
    editor.children.push(Box::new(notif) as Box<dyn View>);
    true
}

fn handle_list_css(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    let css_files = editor.core.list_css();
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
    editor.children.push(Box::new(notif) as Box<dyn View>);
    true
}

fn handle_add_bookmark(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let EditorState::EditingChapter { index } = editor.state {
        editor.core.add_bookmark(index, 0, None);
        let notif = Notification::new(
            format!(
                "Bookmark added for chapter: {}",
                editor.core.chapters[index].title
            ),
            hub,
            rq,
            context,
        );
        editor.children.push(Box::new(notif) as Box<dyn View>);
    }
    true
}

fn handle_replace_all_in_all_documents(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let Some(state) = &editor.search_replace {
        if state.search_text.is_empty() {
            let notif =
                Notification::new("Search text is empty".to_string(), hub, rq, context);
            editor.children.push(Box::new(notif) as Box<dyn View>);
            return true;
        }
        let search_text = state.search_text.clone();
        let options = editor
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
        match editor.core.replace_all_in_all_chapters(
            &search_text,
            &state.replace_text,
            options,
        ) {
            Ok(count) => {
                editor.modified = true;
                let notif = Notification::new(
                    format!("Replaced {} occurrences across all chapters", count),
                    hub,
                    rq,
                    context,
                );
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
            Err(e) => {
                let notif = Notification::new(
                    format!("Error replacing in all chapters: {}", e),
                    hub,
                    rq,
                    context,
                );
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
        }
    }
    true
}

fn handle_close(editor: &mut super::EpubEditor, rq: &mut RenderQueue) -> bool {
    if editor.search_replace.is_some() {
        editor.search_replace = None;
        close_search_replace(editor, rq);
        true
    } else {
        false
    }
}

fn handle_submit_search_input(
    editor: &mut super::EpubEditor,
    text: &str,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let Some(state) = editor.search_replace.as_mut() {
        state.search_text = text.to_string();
    }
    do_search(editor, rq, context);
    true
}

fn handle_submit_replace_input(editor: &mut super::EpubEditor, text: &str) -> bool {
    if let Some(state) = editor.search_replace.as_mut() {
        state.replace_text = text.to_string();
    }
    true
}
