use crate::context::Context;
use crate::geom::Rectangle;
use crate::view::button::Button;
use crate::view::label::Label;
use crate::view::{Align, Bus, EntryId, Event, RenderQueue, View};

pub const CHILD_COUNT: usize = 12;

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

    let gestures = [
        ("Swipe Left", settings.gestures.swipe_left),
        ("Swipe Right", settings.gestures.swipe_right),
        ("Swipe Up", settings.gestures.swipe_up),
        ("Swipe Down", settings.gestures.swipe_down),
        ("Double Tap", settings.gestures.double_tap),
        ("Long Press", settings.gestures.long_press),
    ];

    for (name, action) in gestures {
        let label = Label::new(
            rect![
                rect.min.x + padding,
                y,
                rect.min.x + max_label_width + padding,
                y + small_height
            ],
            name.to_string(),
            Align::Right(padding / 2),
        );
        children.push(Box::new(label) as Box<dyn View>);

        let action_text = gesture_action_label(action);
        let ctrl_rect = rect![
            rect.min.x + max_label_width + 2 * padding,
            y,
            rect.max.x - padding,
            y + small_height
        ];
        let toggle = Button::new(
            ctrl_rect,
            Event::Select(EntryId::CycleGestureAction(name.to_string())),
            action_text,
        );
        children.push(Box::new(toggle) as Box<dyn View>);

        y += small_height;
    }

    let label = Label::new(
        rect![
            rect.min.x + padding,
            y,
            rect.min.x + max_label_width + padding,
            y + small_height
        ],
        "Corner Tap".to_string(),
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
        Event::Select(EntryId::ToggleCornerTap),
        if settings.gestures.corner_tap {
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
        "Pinch to Zoom".to_string(),
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
        Event::Select(EntryId::TogglePinchToZoom),
        if settings.gestures.pinch_to_zoom {
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
        Event::Select(EntryId::CycleGestureAction(ref gesture_name)) => {
            let action = match gesture_name.as_str() {
                "Swipe Left" => &mut context.settings.gestures.swipe_left,
                "Swipe Right" => &mut context.settings.gestures.swipe_right,
                "Swipe Up" => &mut context.settings.gestures.swipe_up,
                "Swipe Down" => &mut context.settings.gestures.swipe_down,
                "Double Tap" => &mut context.settings.gestures.double_tap,
                "Long Press" => &mut context.settings.gestures.long_press,
                _ => return false,
            };
            *action = cycle_gesture_action(*action);
            let new_text = gesture_action_label(*action);

            let index = match gesture_name.as_str() {
                "Swipe Left" => 1,
                "Swipe Right" => 3,
                "Swipe Up" => 5,
                "Swipe Down" => 7,
                "Double Tap" => 9,
                "Long Press" => 11,
                _ => return false,
            };

            if let Some(btn) = children[offset + index].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        Event::Select(EntryId::ToggleCornerTap) => {
            context.settings.gestures.corner_tap = !context.settings.gestures.corner_tap;
            if let Some(btn) = children[offset + 13].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.gestures.corner_tap {
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
        Event::Select(EntryId::TogglePinchToZoom) => {
            context.settings.gestures.pinch_to_zoom = !context.settings.gestures.pinch_to_zoom;
            if let Some(btn) = children[offset + 15].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.gestures.pinch_to_zoom {
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

fn gesture_action_label(action: crate::settings::GestureAction) -> String {
    match action {
        crate::settings::GestureAction::NextPage => "Next Page",
        crate::settings::GestureAction::PreviousPage => "Previous Page",
        crate::settings::GestureAction::ToggleBars => "Toggle Bars",
        crate::settings::GestureAction::GoToPage => "Go To Page",
        crate::settings::GestureAction::ToggleInverted => "Toggle Inverted",
        crate::settings::GestureAction::ToggleDithered => "Toggle Dithered",
        crate::settings::GestureAction::None => "None",
    }
    .to_string()
}

fn cycle_gesture_action(action: crate::settings::GestureAction) -> crate::settings::GestureAction {
    use crate::settings::GestureAction;
    match action {
        GestureAction::NextPage => GestureAction::PreviousPage,
        GestureAction::PreviousPage => GestureAction::ToggleBars,
        GestureAction::ToggleBars => GestureAction::GoToPage,
        GestureAction::GoToPage => GestureAction::ToggleInverted,
        GestureAction::ToggleInverted => GestureAction::ToggleDithered,
        GestureAction::ToggleDithered => GestureAction::None,
        GestureAction::None => GestureAction::NextPage,
    }
}
