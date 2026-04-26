//! Event handlers for EPUB Editor

use rustc_hash::FxHashSet;

use crate::context::Context;
use crate::view::notification::Notification;
use crate::view::{Bus, EntryId, Event, Hub, RenderQueue, View, ViewId};

use super::chapter::{
    handle_add_bookmark, handle_chapter_statistics, handle_delete_chapter, handle_export_chapter,
    handle_generate_toc, handle_import_chapter, handle_list_css, handle_list_images,
    handle_move_chapter_down, handle_move_chapter_up, handle_rename_chapter,
};
use super::metadata::handle_save_metadata;
use super::navigation::{handle_back, handle_next_chapter, handle_previous_chapter};
use super::search_replace::{
    handle_close_search_replace, handle_replace_in_chapter, handle_replace_in_document,
    handle_search_replace, handle_search_replace_init, handle_submit_replace_input,
    handle_submit_search_input, handle_toggle_case_sensitive, handle_toggle_regex,
    handle_toggle_whole_word,
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
            super::helpers::show_edit_view(editor, *i, hub, rq, context);
            true
        }
        Event::Select(EntryId::EditMetadata) => {
            super::helpers::show_metadata_edit_view(editor, hub, rq, context);
            true
        }
        Event::Select(EntryId::PreviousChapter) => {
            handle_previous_chapter(editor, hub, rq, context)
        }
        Event::Select(EntryId::NextChapter) => handle_next_chapter(editor, hub, rq, context),
        Event::Select(EntryId::ToggleRegex) => handle_toggle_regex(editor, rq),
        Event::Select(EntryId::ToggleCaseSensitive) => handle_toggle_case_sensitive(editor, rq),
        Event::Select(EntryId::ToggleWholeWord) => handle_toggle_whole_word(editor, rq),
        Event::Select(EntryId::SaveMetadata) => handle_save_metadata(editor, hub, rq, context),
        Event::Select(EntryId::Save) => handle_save(editor, hub, rq, context),
        Event::Select(EntryId::Discard) => handle_discard(editor),
        Event::Submit(ViewId::EditNoteInput, text) => {
            handle_submit_edit_note(editor, text, hub, rq, context)
        }
        Event::ToggleNear(ViewId::MainMenu, rect) => {
            crate::view::common::toggle_main_menu(editor, *rect, None, rq, context);
            true
        }
        Event::Select(EntryId::Undo) => handle_undo(editor, bus, rq),
        Event::Select(EntryId::Redo) => handle_redo(editor, bus, rq),
        Event::Select(EntryId::Preview) => handle_preview(editor, bus),
        Event::Select(EntryId::SearchReplace) => {
            handle_search_replace_init(editor, hub, rq, context)
        }
        Event::SearchReplace => handle_search_replace(editor, rq, context),
        Event::Select(EntryId::ReplaceInChapter) => {
            handle_replace_in_chapter(editor, hub, rq, context)
        }
        Event::Select(EntryId::ReplaceInDocument) => {
            handle_replace_in_document(editor, hub, rq, context)
        }
        Event::Select(EntryId::CloseSearchReplace) => handle_close_search_replace(editor, rq),
        Event::Select(EntryId::ValidateContent) => {
            handle_validate_content(editor, hub, rq, context)
        }
        Event::Select(EntryId::RenameChapter) => handle_rename_chapter(editor, hub, rq, context),
        Event::Select(EntryId::DeleteChapter) => handle_delete_chapter(editor, hub, rq, context),
        Event::Select(EntryId::MoveChapterUp) => handle_move_chapter_up(editor, hub, rq, context),
        Event::Select(EntryId::MoveChapterDown) => {
            handle_move_chapter_down(editor, hub, rq, context)
        }
        Event::Select(EntryId::SpellCheck) => handle_spell_check(editor, hub, rq, context),
        Event::Select(EntryId::ExportChapter) => handle_export_chapter(editor, hub, rq, context),
        Event::Select(EntryId::ImportChapter) => handle_import_chapter(editor, hub, rq, context),
        Event::Select(EntryId::ChapterStatistics) => {
            handle_chapter_statistics(editor, hub, rq, context)
        }
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
        // BUG-2: commit the renamed chapter title entered by the user
        Event::Submit(ViewId::RenameDocumentInput, ref new_title) => {
            handle_submit_rename_chapter(editor, new_title, hub, rq, context)
        }
        // BUG-3: import a file chosen from the Downloads menu
        Event::Select(EntryId::ImportChapterFile(ref path)) => {
            handle_select_import_file(editor, path, hub, rq, context)
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
    // CQ-4: emit Event::OpenHtml so the main loop opens the chapter in the Reader.
    if let EditorState::EditingChapter { index } = editor.state {
        if index < editor.core.chapters.len() {
            let content = editor.core.chapters[index].content.clone();
            bus.push_back(Event::OpenHtml(content, None));
        }
    }
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

fn handle_clear_history(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    editor.core.clear_history();
    let notif = Notification::new("Undo/redo history cleared".to_string(), hub, rq, context);
    editor.children.push(Box::new(notif) as Box<dyn View>);
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
            let notif = Notification::new("Search text is empty".to_string(), hub, rq, context);
            editor.children.push(Box::new(notif) as Box<dyn View>);
            return true;
        }
        let search_text = state.search_text.clone();
        let options = editor
            .children
            .iter()
            .find(|c| c.is::<crate::view::search_replace::SearchReplaceView>())
            .and_then(|v| v.downcast_ref::<crate::view::search_replace::SearchReplaceView>())
            .map(|sr| {
                let (use_regex, case_sensitive, whole_word) = sr.get_search_options();
                epub_edit::SearchOptions {
                    use_regex,
                    case_sensitive,
                    whole_word,
                }
            })
            .unwrap_or_default();
        match editor
            .core
            .replace_all_in_all_chapters(&search_text, &state.replace_text, options)
        {
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
        super::helpers::close_search_replace(editor, rq);
        true
    } else {
        false
    }
}

/// BUG-2: Apply the new chapter title submitted via the rename input field.
fn handle_submit_rename_chapter(
    editor: &mut super::EpubEditor,
    new_title: &str,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    let new_title = new_title.trim();
    if new_title.is_empty() {
        return true;
    }
    if let EditorState::EditingChapter { index } = editor.state {
        match editor.core.rename_chapter(index, new_title) {
            Ok(_) => {
                editor.modified = true;
                // Dismiss the rename input and keyboard, refresh the edit view.
                super::helpers::show_edit_view(editor, index, hub, rq, context);
                let notif = Notification::new(
                    format!("Chapter renamed to \"{}\".", new_title),
                    hub,
                    rq,
                    context,
                );
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
            Err(e) => {
                let notif =
                    Notification::new(format!("Error renaming chapter: {}", e), hub, rq, context);
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
        }
    }
    true
}

/// BUG-3: Import chapter content from the file path selected in the Downloads menu.
fn handle_select_import_file(
    editor: &mut super::EpubEditor,
    path: &str,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let EditorState::EditingChapter { index } = editor.state {
        let import_path = std::path::Path::new(path);
        match editor.core.import_chapter(index, import_path) {
            Ok(_) => {
                editor.modified = true;
                if !editor.modified_chapters.contains(&index) {
                    editor.modified_chapters.push(index);
                }
                super::helpers::update_input_field(editor, rq, context);
                let notif = Notification::new(
                    format!(
                        "Imported content into \"{}\"",
                        editor.core.chapters[index].title
                    ),
                    hub,
                    rq,
                    context,
                );
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
            Err(e) => {
                let notif =
                    Notification::new(format!("Error importing chapter: {}", e), hub, rq, context);
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
        }
    }
    true
}
