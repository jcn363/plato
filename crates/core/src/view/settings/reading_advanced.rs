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
        "MuPDF Search".to_string(),
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
        Event::Select(EntryId::ToggleMupdfSearch),
        if settings.reader.use_mupdf_search {
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
        "Show Time".to_string(),
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
        Event::Select(EntryId::ToggleShowTime),
        if settings.reader.show_time {
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
        "Show Battery".to_string(),
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
        Event::Select(EntryId::ToggleShowBattery),
        if settings.reader.show_battery {
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
        "Fast Page Turn".to_string(),
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
        Event::Select(EntryId::ToggleFastPageTurn),
        if settings.reader.fast_page_turn {
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
        "Page Animation".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let anim_text = match settings.reader.page_turn_animation {
        crate::settings::PageTurnAnimation::None => "None",
        crate::settings::PageTurnAnimation::Slide => "Slide",
        crate::settings::PageTurnAnimation::Fade => "Fade",
        crate::settings::PageTurnAnimation::Flip => "Flip",
    };
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::CyclePageTurnAnimation),
        anim_text.to_string(),
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
        Event::Select(EntryId::ToggleMupdfSearch) => {
            context.settings.reader.use_mupdf_search = !context.settings.reader.use_mupdf_search;
            if let Some(btn) = children[offset + 1].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.reader.use_mupdf_search {
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
        Event::Select(EntryId::ToggleShowTime) => {
            context.settings.reader.show_time = !context.settings.reader.show_time;
            if let Some(btn) = children[offset + 3].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.reader.show_time {
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
        Event::Select(EntryId::ToggleShowBattery) => {
            context.settings.reader.show_battery = !context.settings.reader.show_battery;
            if let Some(btn) = children[offset + 5].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.reader.show_battery {
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
        Event::Select(EntryId::ToggleFastPageTurn) => {
            context.settings.reader.fast_page_turn = !context.settings.reader.fast_page_turn;
            if let Some(btn) = children[offset + 7].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.reader.fast_page_turn {
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
        Event::Select(EntryId::CyclePageTurnAnimation) => {
            use crate::settings::PageTurnAnimation;
            context.settings.reader.page_turn_animation =
                match context.settings.reader.page_turn_animation {
                    PageTurnAnimation::None => PageTurnAnimation::Slide,
                    PageTurnAnimation::Slide => PageTurnAnimation::Fade,
                    PageTurnAnimation::Fade => PageTurnAnimation::Flip,
                    PageTurnAnimation::Flip => PageTurnAnimation::None,
                };
            let new_text = match context.settings.reader.page_turn_animation {
                PageTurnAnimation::None => "None",
                PageTurnAnimation::Slide => "Slide",
                PageTurnAnimation::Fade => "Fade",
                PageTurnAnimation::Flip => "Flip",
            };
            if let Some(btn) = children[offset + 9].downcast_mut::<Button>() {
                btn.update(new_text.to_string(), rq);
            }
            true
        }
        _ => false,
    }
}
