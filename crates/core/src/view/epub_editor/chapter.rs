//! Chapter manipulation event handlers for EPUB Editor

use crate::context::Context;
use crate::view::notification::Notification;
use crate::view::{Hub, RenderQueue, View};

/// Handle rename chapter action
pub fn handle_rename_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let super::state::EditorState::EditingChapter { index: _ } = editor.state {
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

/// Handle export chapter action
pub fn handle_export_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let super::state::EditorState::EditingChapter { index } = editor.state {
        let export_path = format!("./tmp/chapter_{}.txt", index);
        let path = std::path::Path::new(&export_path);
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
                let notif =
                    Notification::new(format!("Error exporting chapter: {}", e), hub, rq, context);
                editor.children.push(Box::new(notif) as Box<dyn View>);
            }
        }
    }
    true
}

/// Handle import chapter action
pub fn handle_import_chapter(
    editor: &mut super::EpubEditor,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if let super::state::EditorState::EditingChapter { index: _ } = editor.state {
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

/// Handle list images action
pub fn handle_list_images(
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

/// Handle list CSS action
pub fn handle_list_css(
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
