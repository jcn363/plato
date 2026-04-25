//! Chapter manipulation event handlers for EPUB Editor

use rustc_hash::FxHashSet;
use std::fs;
use std::path::Path;

use crate::context::Context;
use crate::view::notification::Notification;
use crate::view::{Hub, RenderQueue, View};

/// Handle rename chapter action (BUG-2).
///
/// Shows a pre-filled input field so the user can edit the chapter title.
/// The result arrives via `Event::Submit(ViewId::RenameDocumentInput, text)`.
pub fn handle_rename_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let super::state::EditorState::EditingChapter { index: _ } = editor.state {
        super::helpers::show_rename_input(editor, hub, rq, context);
    }
    true
}

/// Handle delete chapter action
pub fn handle_delete_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let super::state::EditorState::EditingChapter { index } = editor.state {
        match editor.core.delete_chapter(index) {
            Ok(_) => {
                editor.modified = true;
                let notif = Notification::new("Chapter deleted".to_string(), hub, rq, context);
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
            Err(e) => {
                let notif =
                    Notification::new(format!("Error deleting chapter: {}", e), hub, rq, context);
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
        }
    }
    true
}

/// Handle move chapter up action
pub fn handle_move_chapter_up(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let super::state::EditorState::EditingChapter { index } = editor.state {
        if index > 0 {
            match editor.core.reorder_chapters(index, index - 1) {
                Ok(_) => {
                    editor.modified = true;
                    let notif = Notification::new("Chapter moved up".to_string(), hub, rq, context);
                    editor.children.push(Box::new(notif) as Box<dyn View>);
                }
                Err(e) => {
                    let notif =
                        Notification::new(format!("Error moving chapter: {}", e), hub, rq, context);
                    editor.children.push(Box::new(notif) as Box<dyn View>);
                }
            }
        }
    }
    true
}

/// Handle move chapter down action
pub fn handle_move_chapter_down(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let super::state::EditorState::EditingChapter { index } = editor.state {
        if index < editor.core.chapters.len() - 1 {
            match editor.core.reorder_chapters(index, index + 1) {
                Ok(_) => {
                    editor.modified = true;
                    let notif =
                        Notification::new("Chapter moved down".to_string(), hub, rq, context);
                    editor.children.push(Box::new(notif) as Box<dyn View>);
                }
                Err(e) => {
                    let notif =
                        Notification::new(format!("Error moving chapter: {}", e), hub, rq, context);
                    editor.children.push(Box::new(notif) as Box<dyn View>);
                }
            }
        }
    }
    true
}

/// Handle export chapter action (BUG-4).
///
/// Exports the current chapter to `<library>/Exports/chapter_N_<title>.txt`
/// instead of the fragile relative `./tmp/` path.
pub fn handle_export_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let super::state::EditorState::EditingChapter { index } = editor.state {
        let library_path = context.settings.libraries[context.settings.selected_library]
            .path
            .clone();
        let exports_path = library_path.join("Exports");

        if !exports_path.exists() {
            if let Err(e) = fs::create_dir_all(&exports_path) {
                let notif = Notification::new(
                    format!("Error creating Exports directory: {}", e),
                    hub,
                    rq,
                    context,
                );
                editor.children.push(Box::new(notif) as Box<dyn View>);
                return true;
            }
        }

        let chapter_title = &editor.core.chapters[index].title;
        let safe_title: String = chapter_title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '_' })
            .collect();
        let file_name = format!("chapter_{}_{}.txt", index, safe_title.trim());
        let dest = exports_path.join(&file_name);

        match editor.core.export_chapter(index, Path::new(&dest)) {
            Ok(_) => {
                let notif = Notification::new(
                    format!("Chapter exported to Exports/{}", file_name),
                    hub,
                    rq,
                    context,
                );
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
            Err(e) => {
                let notif =
                    Notification::new(format!("Error exporting chapter: {}", e), hub, rq, context);
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
        }
    }
    true
}

/// Handle import chapter action (BUG-3).
///
/// Scans the library's `Downloads/` directory for `.xhtml`, `.html`, and `.txt`
/// files, then presents them as a menu. The selection is handled via
/// `Event::Select(EntryId::ImportChapterFile(path))` in `event_handlers.rs`.
pub fn handle_import_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let super::state::EditorState::EditingChapter { index: _ } = editor.state {
        super::helpers::show_import_chapter_menu(editor, hub, rq, context);
    }
    true
}

/// Handle chapter statistics action
pub fn handle_chapter_statistics(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let super::state::EditorState::EditingChapter { index } = editor.state {
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

/// Handle generate table of contents action
pub fn handle_generate_toc(
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

/// Handle list images action (CQ-2: uses project-standard FxHashSet).
pub fn handle_list_images(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    let images = editor.core.list_images();
    let chapter_count = images
        .iter()
        .map(|i| i.chapter_index)
        .collect::<FxHashSet<_>>()
        .len();
    let notif = Notification::new(
        format!("Found {} images across {} chapters", images.len(), chapter_count),
        hub,
        rq,
        context,
    );
    editor.children.push(Box::new(notif) as Box<dyn View>);
    true
}

/// Handle list CSS action (CQ-2: uses project-standard FxHashSet).
pub fn handle_list_css(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    let css_files = editor.core.list_css();
    let chapter_count = css_files
        .iter()
        .map(|c| c.chapter_index)
        .collect::<FxHashSet<_>>()
        .len();
    let notif = Notification::new(
        format!("Found {} CSS files across {} chapters", css_files.len(), chapter_count),
        hub,
        rq,
        context,
    );
    editor.children.push(Box::new(notif) as Box<dyn View>);
    true
}

/// Handle add bookmark action
pub fn handle_add_bookmark(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let super::state::EditorState::EditingChapter { index } = editor.state {
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
