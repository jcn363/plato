//! Metadata event handlers for EPUB Editor

use crate::context::Context;
use crate::view::input_field::InputField;
use crate::view::notification::Notification;
use crate::view::{Hub, RenderQueue, View, ViewId};

use super::helpers::show_chapter_list;

/// Handle save metadata action
pub fn handle_save_metadata(
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
    if let Some(view) = editor
        .children
        .iter()
        .find(|c| c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataAuthor))
    {
        if let Some(input) = view.downcast_ref::<InputField>() {
            new_meta.author = input.get_text().to_string();
        }
    }
    if let Some(view) = editor
        .children
        .iter()
        .find(|c| c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataLanguage))
    {
        if let Some(input) = view.downcast_ref::<InputField>() {
            new_meta.language = input.get_text().to_string();
        }
    }
    if let Some(view) = editor
        .children
        .iter()
        .find(|c| c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataIdentifier))
    {
        if let Some(input) = view.downcast_ref::<InputField>() {
            new_meta.identifier = input.get_text().to_string();
        }
    }
    if let Some(view) = editor
        .children
        .iter()
        .find(|c| c.is::<InputField>() && c.view_id() == Some(ViewId::EditMetadataPublisher))
    {
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
