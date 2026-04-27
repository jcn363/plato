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

    let label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Dropbox Token".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let token_text = if settings.cloud_storage.dropbox_token.is_some() {
        "Set".to_string()
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
        Event::Select(EntryId::EditDropboxToken),
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
        "Google Drive Token".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let token_text = if settings.cloud_storage.google_drive_token.is_some() {
        "Set".to_string()
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
        Event::Select(EntryId::EditGoogleDriveToken),
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
        "OneDrive Token".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let token_text = if settings.cloud_storage.onedrive_token.is_some() {
        "Set".to_string()
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
        Event::Select(EntryId::EditOneDriveToken),
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
        Event::Select(EntryId::ToggleCloudStorageAutoSync),
        if settings.cloud_storage.auto_sync {
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
        Event::Select(EntryId::ToggleCloudStorageSyncProgress),
        if settings.cloud_storage.sync_progress {
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
        "Sync Annotations".to_string(),
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
        Event::Select(EntryId::ToggleCloudStorageSyncAnnotations),
        if settings.cloud_storage.sync_annotations {
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
        Event::Select(EntryId::ToggleCloudStorageAutoSync) => {
            context.settings.cloud_storage.auto_sync = !context.settings.cloud_storage.auto_sync;
            if let Some(btn) = children[offset + 7].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.cloud_storage.auto_sync {
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
        Event::Select(EntryId::ToggleCloudStorageSyncProgress) => {
            context.settings.cloud_storage.sync_progress =
                !context.settings.cloud_storage.sync_progress;
            if let Some(btn) = children[offset + 9].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.cloud_storage.sync_progress {
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
        Event::Select(EntryId::ToggleCloudStorageSyncAnnotations) => {
            context.settings.cloud_storage.sync_annotations =
                !context.settings.cloud_storage.sync_annotations;
            if let Some(btn) = children[offset + 11].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.cloud_storage.sync_annotations {
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
