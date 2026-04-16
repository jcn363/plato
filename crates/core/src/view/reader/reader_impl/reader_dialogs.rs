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

/// Helper to toggle a dialog view with common logic.
///
/// This reduces duplication across dialog toggle functions by handling:
/// - Checking if the view already exists
/// - Handling enable/disable flags
/// - Removing existing views with proper expose
/// - Adding new views with render data
fn toggle_dialog_view<F>(
    children: &mut Vec<Box<dyn View>>,
    id: ViewId,
    enable: Option<bool>,
    make_view: F,
    rq: &mut RenderQueue,
) -> bool
where
    F: FnOnce() -> Box<dyn View>,
{
    if let Some(index) = locate_by_id_in_vec(children, id) {
        if let Some(true) = enable {
            return false; // Already open
        }
        rq.add(RenderData::expose(*children[index].rect(), UpdateMode::Gui));
        children.remove(index);
        true
    } else {
        if let Some(false) = enable {
            return false; // Explicitly disabled
        }
        let view = make_view();
        rq.add(RenderData::new(view.id(), *view.rect(), UpdateMode::Gui));
        children.push(view);
        true
    }
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
    let created = toggle_dialog_view(
        children,
        ViewId::EditNote,
        enable,
        || {
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
            Box::new(edit_note) as Box<dyn View>
        },
        rq,
    );

    if created {
        hub.send(Event::Focus(Some(ViewId::EditNoteInput))).ok();
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
    let created = toggle_dialog_view(
        children,
        ViewId::NamePage,
        enable,
        || {
            Box::new(NamedInput::new(
                "Name page".to_string(),
                ViewId::NamePage,
                ViewId::NamePageInput,
                4,
                context,
            )) as Box<dyn View>
        },
        rq,
    );

    if created {
        hub.send(Event::Focus(Some(ViewId::NamePageInput))).ok();
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

    let created = toggle_dialog_view(
        children,
        id,
        enable,
        || Box::new(NamedInput::new(text.to_string(), id, input_id, 4, context)) as Box<dyn View>,
        rq,
    );

    if created {
        hub.send(Event::Focus(Some(input_id))).ok();
    }
}
