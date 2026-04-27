use crate::context::Context;
use crate::geom::Rectangle;
use crate::view::button::Button;
use crate::view::label::Label;
use crate::view::{Align, Bus, EntryId, Event, RenderQueue, View};

pub const CHILD_COUNT: usize = 10;

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
        "Server Host".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let host_text = if settings.calibre.host.is_empty() {
        "Not Set".to_string()
    } else {
        settings.calibre.host.clone()
    };
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::EditCalibreHost),
        host_text,
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
        "Server Port".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let port_text = if settings.calibre.port == 0 {
        "Default (8080)".to_string()
    } else {
        settings.calibre.port.to_string()
    };
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::EditCalibrePort),
        port_text,
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
        "Username".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let username_text = settings
        .calibre
        .username
        .as_ref()
        .map_or("None".to_string(), |u| u.clone());
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::EditCalibreUsername),
        username_text,
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
        "Password".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let password_text = if settings.calibre.password.is_some() {
        "******".to_string()
    } else {
        "None".to_string()
    };
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::EditCalibrePassword),
        password_text,
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
        Event::Select(EntryId::ToggleCalibreAutoSync),
        if settings.calibre.auto_sync {
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
        "Sync Metadata".to_string(),
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
        Event::Select(EntryId::ToggleCalibreSyncMetadata),
        if settings.calibre.sync_metadata {
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
        "Test Connection".to_string(),
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
        Event::Select(EntryId::TestCalibreConnection),
        "Test".to_string(),
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
        Event::Select(EntryId::ToggleCalibreAutoSync) => {
            context.settings.calibre.auto_sync = !context.settings.calibre.auto_sync;
            if let Some(btn) = children[offset + 13].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.calibre.auto_sync {
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
        Event::Select(EntryId::ToggleCalibreSyncMetadata) => {
            context.settings.calibre.sync_metadata = !context.settings.calibre.sync_metadata;
            if let Some(btn) = children[offset + 15].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.calibre.sync_metadata {
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
        Event::Select(EntryId::TestCalibreConnection) => {
            // TODO: Implement connection test
            if let Some(btn) = children[offset + 17].downcast_mut::<Button>() {
                btn.update("Testing...".to_string(), rq);
            }
            true
        }
        _ => false,
    }
}
