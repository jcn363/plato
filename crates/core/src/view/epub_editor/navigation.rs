//! Navigation event handlers for EPUB Editor

use crate::context::Context;
use crate::view::{Hub, RenderQueue};

use super::helpers::{show_chapter_list, show_edit_view, show_save_dialog};
use super::state::EditorState;

/// Handle back button press
pub fn handle_back(
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

/// Handle previous chapter navigation
pub fn handle_previous_chapter(
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

/// Handle next chapter navigation
pub fn handle_next_chapter(
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
