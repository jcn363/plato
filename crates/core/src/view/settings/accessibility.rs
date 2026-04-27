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
        "High Contrast".to_string(),
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
        Event::Select(EntryId::ToggleHighContrast),
        if settings.accessibility.high_contrast {
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
        "Letter Spacing".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let spacing_text = format!("{:.1}", settings.accessibility.letter_spacing);
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::IncreaseLetterSpacing),
        spacing_text,
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
        "Line Height".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let line_height_text = format!("{:.1}", settings.accessibility.line_height);
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::IncreaseLineHeight),
        line_height_text,
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
        Event::Select(EntryId::ToggleHighContrast) => {
            context.settings.accessibility.high_contrast =
                !context.settings.accessibility.high_contrast;
            if let Some(btn) = children[offset + 1].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.accessibility.high_contrast {
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
        Event::Select(EntryId::IncreaseLetterSpacing) => {
            context.settings.accessibility.letter_spacing =
                (context.settings.accessibility.letter_spacing + 0.1).min(1.0);
            let new_text = format!("{:.1}", context.settings.accessibility.letter_spacing);
            if let Some(btn) = children[offset + 3].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        Event::Select(EntryId::DecreaseLetterSpacing) => {
            context.settings.accessibility.letter_spacing =
                (context.settings.accessibility.letter_spacing - 0.1).max(0.0);
            let new_text = format!("{:.1}", context.settings.accessibility.letter_spacing);
            if let Some(btn) = children[offset + 3].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        Event::Select(EntryId::IncreaseLineHeight) => {
            context.settings.accessibility.line_height =
                (context.settings.accessibility.line_height + 0.1).min(3.0);
            let new_text = format!("{:.1}", context.settings.accessibility.line_height);
            if let Some(btn) = children[offset + 5].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        Event::Select(EntryId::DecreaseLineHeight) => {
            context.settings.accessibility.line_height =
                (context.settings.accessibility.line_height - 0.1).max(0.5);
            let new_text = format!("{:.1}", context.settings.accessibility.line_height);
            if let Some(btn) = children[offset + 5].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        _ => false,
    }
}
