//! Reader Dialog Module
//!
//! Handles input dialogs and text entry interactions.
//!
//! ## Methods Extracted
//! - `toggle_edit_note()` - Note editing dialog ✓
//! - `toggle_name_page()` - Page naming dialog ✓
//! - `toggle_go_to_page()` - Go to page dialog ✓

use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::view::named_input::NamedInput;
use crate::view::{Event, Hub, RenderData, RenderQueue, View, ViewId};

/// Find child view index by ViewId in children vector
#[inline]
fn locate_by_id_in_vec(children: &[Box<dyn View>], id: ViewId) -> Option<usize> {
    children
        .iter()
        .position(|c| c.view_id().map_or(false, |i| i == id))
}

/// Toggle note editing dialog
pub(crate) fn toggle_edit_note(
    children: &mut Vec<Box<dyn View>>,
    text: Option<&str>,
    enable: Option<bool>,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    if let Some(index) = locate_by_id_in_vec(children, ViewId::EditNote) {
        if let Some(true) = enable {
            return;
        }

        rq.add(RenderData::expose(*children[index].rect(), UpdateMode::Gui));
        children.remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }

        let mut edit_note = NamedInput::new(
            "Note".to_string(),
            ViewId::EditNote,
            ViewId::EditNoteInput,
            32,
            context,
        );
        if let Some(text) = text.as_ref() {
            edit_note.set_text(text, &mut RenderQueue::new(), context);
        }

        rq.add(RenderData::new(
            edit_note.id(),
            *edit_note.rect(),
            UpdateMode::Gui,
        ));
        hub.send(Event::Focus(Some(ViewId::EditNoteInput))).ok();

        children.push(Box::new(edit_note) as Box<dyn View>);
    }
}

/// Toggle page naming dialog
pub(crate) fn toggle_name_page(
    children: &mut Vec<Box<dyn View>>,
    enable: Option<bool>,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    if let Some(index) = locate_by_id_in_vec(children, ViewId::NamePage) {
        if let Some(true) = enable {
            return;
        }

        rq.add(RenderData::expose(*children[index].rect(), UpdateMode::Gui));
        children.remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }

        let name_page = NamedInput::new(
            "Name page".to_string(),
            ViewId::NamePage,
            ViewId::NamePageInput,
            4,
            context,
        );
        rq.add(RenderData::new(
            name_page.id(),
            *name_page.rect(),
            UpdateMode::Gui,
        ));
        hub.send(Event::Focus(Some(ViewId::NamePageInput))).ok();

        children.push(Box::new(name_page) as Box<dyn View>);
    }
}

/// Toggle go to page dialog
pub(crate) fn toggle_go_to_page(
    children: &mut Vec<Box<dyn View>>,
    enable: Option<bool>,
    id: ViewId,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    let (text, input_id) = if id == ViewId::GoToPage {
        ("Go to page", ViewId::GoToPageInput)
    } else {
        ("Go to results page", ViewId::GoToResultsPageInput)
    };

    if let Some(index) = locate_by_id_in_vec(children, id) {
        if let Some(true) = enable {
            return;
        }

        rq.add(RenderData::expose(*children[index].rect(), UpdateMode::Gui));
        children.remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }

        let go_to_page = NamedInput::new(text.to_string(), id, input_id, 4, context);
        rq.add(RenderData::new(
            go_to_page.id(),
            *go_to_page.rect(),
            UpdateMode::Gui,
        ));
        hub.send(Event::Focus(Some(input_id))).ok();

        children.push(Box::new(go_to_page) as Box<dyn View>);
    }
}
