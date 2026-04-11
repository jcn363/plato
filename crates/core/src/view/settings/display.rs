use crate::context::Context;
use crate::geom::Rectangle;
use crate::settings::ThemeMode;
use crate::theme;
use crate::view::button::Button;
use crate::view::label::Label;
use crate::view::{Align, Bus, EntryId, Event, RenderQueue, View};

pub const CHILD_COUNT: usize = 18;

pub fn build_rows(
    rect: &Rectangle,
    y_pos: i32,
    small_height: i32,
    padding: i32,
    max_label_width: i32,
    settings: &crate::settings::Settings,
    context: &Context,
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
        "Frontlight".to_string(),
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
        Event::Select(EntryId::ToggleFrontlight),
        if settings.frontlight { "On" } else { "Off" }.to_string(),
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
        "WiFi".to_string(),
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
        Event::Select(EntryId::ToggleWifi),
        if settings.wifi { "On" } else { "Off" }.to_string(),
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
        "Inverted".to_string(),
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
        Event::Select(EntryId::ToggleInverted),
        if settings.inverted { "On" } else { "Off" }.to_string(),
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
        "Sleep Cover".to_string(),
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
        Event::Select(EntryId::ToggleSleepCover),
        if settings.sleep_cover { "On" } else { "Off" }.to_string(),
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
        "Dithering".to_string(),
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
        Event::Select(EntryId::ToggleDithered),
        if context.fb.dithered() { "On" } else { "Off" }.to_string(),
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
        "Ext. Storage".to_string(),
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
        Event::Select(EntryId::ToggleExternalStorage),
        if settings.external_storage.enabled {
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
        "Dark Mode".to_string(),
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
        Event::Select(EntryId::ToggleDarkMode),
        match settings.theme_settings.mode {
            ThemeMode::Light => "Off".to_string(),
            ThemeMode::Dark => "On".to_string(),
            ThemeMode::Sepia => "Sepia".to_string(),
            ThemeMode::Auto => "Auto".to_string(),
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
        "Auto Threshold".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let threshold_str = format!("{}", settings.theme_settings.auto_threshold);
    let threshold_btn = Button::new(
        ctrl_rect,
        Event::Select(EntryId::SetAutoThemeThreshold),
        threshold_str,
    );
    children.push(Box::new(threshold_btn) as Box<dyn View>);

    y += small_height;

    (children, y)
}

pub fn handle_event(
    evt: &Event,
    children: &mut [Box<dyn View>],
    offset: usize,
    bus: &mut Bus,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    match *evt {
        Event::Select(EntryId::ToggleFrontlight) => {
            context.set_frontlight(!context.settings.frontlight);
            if let Some(btn) = children[offset + 1].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.frontlight {
                        "On"
                    } else {
                        "Off"
                    }
                    .to_string(),
                    rq,
                );
            }
            bus.push_back(Event::ToggleFrontlight);
            true
        }
        Event::Select(EntryId::ToggleWifi) => {
            context.settings.wifi = !context.settings.wifi;
            if let Some(btn) = children[offset + 3].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.wifi { "On" } else { "Off" }.to_string(),
                    rq,
                );
            }
            bus.push_back(Event::Select(EntryId::ToggleWifi));
            true
        }
        Event::Select(EntryId::ToggleInverted) => {
            context.fb.toggle_inverted();
            context.settings.inverted = context.fb.inverted();
            if let Some(btn) = children[offset + 5].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.inverted {
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
        Event::Select(EntryId::ToggleSleepCover) => {
            context.settings.sleep_cover = !context.settings.sleep_cover;
            if let Some(btn) = children[offset + 7].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.sleep_cover {
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
        Event::Select(EntryId::ToggleExternalStorage) => {
            context.settings.external_storage.enabled = !context.settings.external_storage.enabled;
            if let Some(btn) = children[offset + 9].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.external_storage.enabled {
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
        Event::Select(EntryId::ToggleDithered) => {
            context.fb.toggle_dithered();
            if let Some(btn) = children[offset + 11].downcast_mut::<Button>() {
                btn.update(
                    if context.fb.dithered() { "On" } else { "Off" }.to_string(),
                    rq,
                );
            }
            true
        }
        Event::Select(EntryId::ToggleDarkMode) => {
            use crate::settings::ThemeMode;
            let current_mode = context.settings.theme_settings.mode;
            let new_mode = match current_mode {
                ThemeMode::Light => ThemeMode::Dark,
                ThemeMode::Dark => ThemeMode::Sepia,
                ThemeMode::Sepia => ThemeMode::Auto,
                ThemeMode::Auto => ThemeMode::Light,
            };
            context.settings.theme_settings.mode = new_mode;
            theme::set_theme_mode(new_mode);

            match new_mode {
                ThemeMode::Light | ThemeMode::Sepia => {
                    context.settings.dark_mode = false;
                    theme::set_dark_mode(false);
                }
                ThemeMode::Dark => {
                    context.settings.dark_mode = true;
                    theme::set_dark_mode(true);
                }
                ThemeMode::Auto => {
                    let dark = if crate::device::CURRENT_DEVICE.has_lightsensor() {
                        context.lightsensor.level().unwrap_or(100)
                            < context.settings.theme_settings.auto_threshold
                    } else {
                        false
                    };
                    context.settings.dark_mode = dark;
                    theme::set_dark_mode(dark);
                }
            }

            if let Some(btn) = children[offset + 13].downcast_mut::<Button>() {
                btn.update(
                    match new_mode {
                        ThemeMode::Light => "Off".to_string(),
                        ThemeMode::Dark => "On".to_string(),
                        ThemeMode::Sepia => "Sepia".to_string(),
                        ThemeMode::Auto => "Auto".to_string(),
                    },
                    rq,
                );
            }
            true
        }
        Event::Select(EntryId::SetAutoThemeThreshold) => {
            let current = context.settings.theme_settings.auto_threshold;
            let new_threshold = if current >= 200 { 50 } else { current + 50 };
            context.settings.theme_settings.auto_threshold = new_threshold;
            theme::set_auto_threshold(new_threshold);

            if let Some(btn) = children[offset + 15].downcast_mut::<Button>() {
                btn.update(format!("{}", new_threshold), rq);
            }

            if context.settings.theme_settings.mode == ThemeMode::Auto
                && crate::device::CURRENT_DEVICE.has_lightsensor()
            {
                if let Ok(level) = context.lightsensor.level() {
                    theme::update_from_light_sensor(level);
                    context.settings.dark_mode = theme::is_dark_mode();
                }
            }
            true
        }
        _ => false,
    }
}
