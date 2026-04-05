use crate::context::Context;
use crate::geom::Rectangle;
use crate::view::label::Label;
use crate::view::slider::Slider;
use crate::view::{Align, Bus, Event, RenderQueue, SliderId, View};

pub const CHILD_COUNT: usize = 4;

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
        "Auto Suspend (min)".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let slider = Slider::new(
        ctrl_rect,
        SliderId::AutoSuspend,
        settings.auto_suspend,
        5.0,
        60.0,
    );
    children.push(Box::new(slider) as Box<dyn View>);

    y += small_height;

    let label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Auto Power Off (h)".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let slider = Slider::new(
        ctrl_rect,
        SliderId::AutoPowerOff,
        settings.auto_power_off,
        0.5,
        12.0,
    );
    children.push(Box::new(slider) as Box<dyn View>);

    y += small_height;

    (children, y)
}

pub fn handle_event(
    evt: &Event,
    _children: &mut [Box<dyn View>],
    _offset: usize,
    _bus: &mut Bus,
    _rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    match *evt {
        Event::Slider(SliderId::AutoSuspend, value, _) => {
            context.settings.auto_suspend = value;
            true
        }
        Event::Slider(SliderId::AutoPowerOff, value, _) => {
            context.settings.auto_power_off = value;
            true
        }
        _ => false,
    }
}
