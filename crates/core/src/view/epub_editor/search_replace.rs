//! Search and replace event handlers for EPUB Editor

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::view::search_replace::SearchReplaceView;
use crate::view::{Hub, RenderData, RenderQueue, View};

use super::helpers::{close_search_replace, do_replace_in_chapter, do_search, show_search_replace};

/// Handle toggle regex option
pub fn handle_toggle_regex(editor: &mut super::EpubEditor, rq: &mut RenderQueue) -> bool {
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

/// Handle toggle case sensitive option
pub fn handle_toggle_case_sensitive(editor: &mut super::EpubEditor, rq: &mut RenderQueue) -> bool {
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

/// Handle toggle whole word option
pub fn handle_toggle_whole_word(editor: &mut super::EpubEditor, rq: &mut RenderQueue) -> bool {
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

/// Handle search/replace initialization
pub fn handle_search_replace_init(
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

/// Handle search action
pub fn handle_search_replace(
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

/// Handle replace in chapter action
pub fn handle_replace_in_chapter(
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

/// Handle replace in document action
pub fn handle_replace_in_document(
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
            let notif = crate::view::notification::Notification::new(
                "Search text is empty".to_string(),
                hub,
                rq,
                context,
            );
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
        match editor
            .core
            .replace_all_in_document(&state.search_text, &state.replace_text, options)
        {
            Ok(count) => {
                if count > 0 {
                    editor.modified = true;
                    let notif = crate::view::notification::Notification::new(
                        format!("Replaced {} occurrence(s) in document", count),
                        hub,
                        rq,
                        context,
                    );
                    editor.children.push(Box::new(notif) as Box<dyn View>);
                    if let super::state::EditorState::EditingChapter { index: _ } = editor.state {
                        super::helpers::update_input_field(editor, rq, context);
                    }
                } else {
                    let notif = crate::view::notification::Notification::new(
                        "No matches found in document".to_string(),
                        hub,
                        rq,
                        context,
                    );
                    editor.children.push(Box::new(notif) as Box<dyn View>);
                }
            }
            Err(e) => {
                let notif = crate::view::notification::Notification::new(
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

/// Handle close search/replace
pub fn handle_close_search_replace(editor: &mut super::EpubEditor, rq: &mut RenderQueue) -> bool {
    editor.search_replace = None;
    close_search_replace(editor, rq);
    true
}

/// Handle submit search input
pub fn handle_submit_search_input(
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

/// Handle submit replace input
pub fn handle_submit_replace_input(editor: &mut super::EpubEditor, text: &str) -> bool {
    if let Some(state) = editor.search_replace.as_mut() {
        state.replace_text = text.to_string();
    }
    true
}
