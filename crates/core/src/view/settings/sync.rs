use crate::context::Context;
use crate::geom::Rectangle;
use crate::view::button::Button;
use crate::view::label::Label;
use crate::view::{Align, Bus, EntryId, Event, RenderQueue, View};

pub const CHILD_COUNT: usize = 8;

pub fn build_rows(
    rect: &Rectangle,
    y_pos: i32,
    small_height: i32,
    padding: i32,
    max_label_width: i32,
    settings: &crate::settings::Settings,
) -> (Vec<Box<dyn View>>, i32) {
    let mut children = Vec::new();
    let mut y = y_pos;

    // Title
    let title = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.max.x - padding,
            y + small_height
        ],
        "Cross-Device Sync".to_string(),
        Align::Left(padding),
    );
    children.push(Box::new(title) as Box<dyn View>);
    y += small_height;

    // Sync reading position toggle
    let sync_reading_label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Sync Reading Position".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(sync_reading_label) as Box<dyn View>);

    let sync_reading_button = Button::new(
        rect![
            rect.min.x + max_label_width + padding * 2,
            y,
            rect.max.x - padding,
            y + small_height
        ],
        Event::Select(EntryId::ToggleSyncReadingPosition),
        if settings.cloud_sync.auto_sync {
            "On"
        } else {
            "Off"
        }
        .to_string(),
    );
    children.push(Box::new(sync_reading_button) as Box<dyn View>);
    y += small_height;

    // Sync highlights toggle
    let sync_highlights_label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Sync Highlights".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(sync_highlights_label) as Box<dyn View>);

    let sync_highlights_button = Button::new(
        rect![
            rect.min.x + max_label_width + padding * 2,
            y,
            rect.max.x - padding,
            y + small_height
        ],
        Event::Select(EntryId::ToggleSyncHighlights),
        if settings.cloud_sync.auto_sync {
            "On"
        } else {
            "Off"
        }
        .to_string(),
    );
    children.push(Box::new(sync_highlights_button) as Box<dyn View>);
    y += small_height;

    // Sync annotations toggle
    let sync_annotations_label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Sync Annotations".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(sync_annotations_label) as Box<dyn View>);

    let sync_annotations_button = Button::new(
        rect![
            rect.min.x + max_label_width + padding * 2,
            y,
            rect.max.x - padding,
            y + small_height
        ],
        Event::Select(EntryId::ToggleSyncAnnotations),
        if settings.cloud_sync.auto_sync {
            "On"
        } else {
            "Off"
        }
        .to_string(),
    );
    children.push(Box::new(sync_annotations_button) as Box<dyn View>);
    y += small_height;

    // Sync on open toggle
    let sync_on_open_label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Sync on Open".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(sync_on_open_label) as Box<dyn View>);

    let sync_on_open_button = Button::new(
        rect![
            rect.min.x + max_label_width + padding * 2,
            y,
            rect.max.x - padding,
            y + small_height
        ],
        Event::Select(EntryId::ToggleSyncOnOpen),
        if settings.background_sync.sync_on_open {
            "On"
        } else {
            "Off"
        }
        .to_string(),
    );
    children.push(Box::new(sync_on_open_button) as Box<dyn View>);
    y += small_height;

    // Sync on close toggle
    let sync_on_close_label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Sync on Close".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(sync_on_close_label) as Box<dyn View>);

    let sync_on_close_button = Button::new(
        rect![
            rect.min.x + max_label_width + padding * 2,
            y,
            rect.max.x - padding,
            y + small_height
        ],
        Event::Select(EntryId::ToggleSyncOnClose),
        if settings.background_sync.sync_on_close {
            "On"
        } else {
            "Off"
        }
        .to_string(),
    );
    children.push(Box::new(sync_on_close_button) as Box<dyn View>);
    y += small_height;

    // Manual sync button
    let sync_button = Button::new(
        rect![
            rect.min.x + padding,
            y,
            rect.max.x - padding,
            y + small_height
        ],
        Event::Select(EntryId::ManualSync),
        "Sync Now".to_string(),
    );
    children.push(Box::new(sync_button) as Box<dyn View>);
    y += small_height;

    (children, y)
}

pub fn handle_event(
    evt: &Event,
    children: &mut Vec<Box<dyn View>>,
    offset: usize,
    bus: &mut Bus,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    match evt {
        Event::Select(EntryId::ToggleSyncReadingPosition) => {
            context.settings.cloud_sync.auto_sync = !context.settings.cloud_sync.auto_sync;
            if let Some(btn) = children[offset + 1].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.cloud_sync.auto_sync {
                        "On"
                    } else {
                        "Off"
                    }
                    .to_string(),
                    rq,
                );
            }
            true
        }
        Event::Select(EntryId::ToggleSyncHighlights) => {
            context.settings.cloud_sync.auto_sync = !context.settings.cloud_sync.auto_sync;
            if let Some(btn) = children[offset + 3].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.cloud_sync.auto_sync {
                        "On"
                    } else {
                        "Off"
                    }
                    .to_string(),
                    rq,
                );
            }
            true
        }
        Event::Select(EntryId::ToggleSyncAnnotations) => {
            context.settings.cloud_sync.auto_sync = !context.settings.cloud_sync.auto_sync;
            if let Some(btn) = children[offset + 5].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.cloud_sync.auto_sync {
                        "On"
                    } else {
                        "Off"
                    }
                    .to_string(),
                    rq,
                );
            }
            true
        }
        Event::Select(EntryId::ToggleSyncOnOpen) => {
            context.settings.background_sync.sync_on_open =
                !context.settings.background_sync.sync_on_open;
            if let Some(btn) = children[offset + 7].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.background_sync.sync_on_open {
                        "On"
                    } else {
                        "Off"
                    }
                    .to_string(),
                    rq,
                );
            }
            true
        }
        Event::Select(EntryId::ToggleSyncOnClose) => {
            context.settings.background_sync.sync_on_close =
                !context.settings.background_sync.sync_on_close;
            if let Some(btn) = children[offset + 9].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.background_sync.sync_on_close {
                        "On"
                    } else {
                        "Off"
                    }
                    .to_string(),
                    rq,
                );
            }
            true
        }
        Event::Select(EntryId::ManualSync) => {
            // Trigger manual sync
            bus.push_back(Event::Render("Syncing...".to_string()));
            true
        }
        _ => false,
    }
}
