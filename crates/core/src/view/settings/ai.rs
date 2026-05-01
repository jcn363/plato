//! AI Settings View for Plato
//!
//! Provides UI for configuring AI features on desktop platforms.

use crate::context::Context;
use crate::geom::Rectangle;
use crate::settings::Settings;
use crate::view::button::Button;
use crate::view::label::Label;
use crate::view::{Align, Bus, EntryId, Event, RenderQueue, View};

pub const CHILD_COUNT: usize = 4;

pub fn build_rows(
    rect: &Rectangle,
    y_pos: i32,
    small_height: i32,
    padding: i32,
    max_label_width: i32,
    settings: &Settings,
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
        "AI Features".to_string(),
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
        Event::Select(EntryId::ToggleAiFeature),
        if settings.ai.enabled {
            "On".to_string()
        } else {
            "Off".to_string()
        },
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
        "Provider".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    y += small_height;

    let label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Model".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    (children, y)
}

pub fn handle_event(
    evt: &Event,
    children: &mut [Box<dyn View>],
    offset: usize,
    _bus: &mut Bus,
    _rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    match evt {
        Event::Select(EntryId::ToggleAiFeature) => {
            context.settings.ai.enabled = !context.settings.ai.enabled;
            if let Some(btn) = children[offset + 1].downcast_mut::<Button>() {
                let txt = if context.settings.ai.enabled {
                    "On"
                } else {
                    "Off"
                };
                btn.set_text(txt);
            }
            true
        }
        _ => false,
    }
}
