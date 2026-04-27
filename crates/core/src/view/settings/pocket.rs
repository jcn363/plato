use crate::context::Context;
use crate::geom::Rectangle;
use crate::view::button::Button;
use crate::view::label::Label;
use crate::view::{Align, Bus, EntryId, Event, RenderQueue, View};

pub const CHILD_COUNT: usize = 6;

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

    let label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Consumer Key".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let key_text = settings
        .pocket
        .consumer_key
        .as_ref()
        .map_or("Not Set".to_string(), |k| k.clone());
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::EditPocketConsumerKey),
        key_text,
    );
    children.push(Box::new(toggle) as Box<dyn View>);

    y += small_height;

    let label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Access Token".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let token_text = if settings.pocket.access_token.is_some() {
        "******".to_string()
    } else {
        "Not Set".to_string()
    };
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::EditPocketAccessToken),
        token_text,
    );
    children.push(Box::new(toggle) as Box<dyn View>);

    y += small_height;

    let label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Auto Sync".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::TogglePocketAutoSync),
        if settings.pocket.auto_sync {
            "On"
        } else {
            "Off"
        }
        .to_string(),
    );
    children.push(Box::new(toggle) as Box<dyn View>);

    y += small_height;

    let label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Sync Progress".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::TogglePocketSyncProgress),
        if settings.pocket.sync_progress {
            "On"
        } else {
            "Off"
        }
        .to_string(),
    );
    children.push(Box::new(toggle) as Box<dyn View>);

    y += small_height;

    let label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Archive After Reading".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::TogglePocketArchiveAfterReading),
        if settings.pocket.archive_after_reading {
            "On"
        } else {
            "Off"
        }
        .to_string(),
    );
    children.push(Box::new(toggle) as Box<dyn View>);

    y += small_height;

    (children, y)
}

pub fn handle_event(
    evt: &Event,
    children: &mut [Box<dyn View>],
    offset: usize,
    _bus: &mut Bus,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    match *evt {
        Event::Select(EntryId::TogglePocketAutoSync) => {
            context.settings.pocket.auto_sync = !context.settings.pocket.auto_sync;
            if let Some(btn) = children[offset + 5].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.pocket.auto_sync {
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
        Event::Select(EntryId::TogglePocketSyncProgress) => {
            context.settings.pocket.sync_progress = !context.settings.pocket.sync_progress;
            if let Some(btn) = children[offset + 7].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.pocket.sync_progress {
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
        Event::Select(EntryId::TogglePocketArchiveAfterReading) => {
            context.settings.pocket.archive_after_reading =
                !context.settings.pocket.archive_after_reading;
            if let Some(btn) = children[offset + 9].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.pocket.archive_after_reading {
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
        _ => false,
    }
}
