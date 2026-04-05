use crate::context::Context;
use crate::geom::Rectangle;
use crate::view::button::Button;
use crate::view::label::Label;
use crate::view::{Align, Bus, EntryId, Event, RenderQueue, View};

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
        "Auto Dual Page".to_string(),
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
        Event::Select(EntryId::ToggleDualPage),
        if settings.reader.auto_dual_page {
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
        "Finished Action".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let finished_text = match settings.reader.finished {
        crate::settings::FinishedAction::Notify => "Notify",
        crate::settings::FinishedAction::Close => "Close",
        crate::settings::FinishedAction::GoToNext => "Go To Next",
    };
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::CycleFinishedAction),
        finished_text.to_string(),
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
        "Language".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let lang_text = if settings.language == "es" {
        "Español"
    } else {
        "English"
    };
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::CycleLanguage),
        lang_text.to_string(),
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
        "UI Font".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let uifont_text = match settings.ui_font {
        crate::settings::UiFont::SansSerif => "Sans Serif",
        crate::settings::UiFont::Serif => "Serif",
    };
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::CycleUiFont),
        uifont_text.to_string(),
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
        "Manga Mode".to_string(),
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
        Event::Select(EntryId::ToggleMangaMode),
        if settings.reader.manga_mode {
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
        Event::Select(EntryId::ToggleDualPage) => {
            context.settings.reader.auto_dual_page = !context.settings.reader.auto_dual_page;
            if let Some(btn) = children[offset + 1].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.reader.auto_dual_page {
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
        Event::Select(EntryId::CycleFinishedAction) => {
            use crate::settings::FinishedAction;
            context.settings.reader.finished = match context.settings.reader.finished {
                FinishedAction::Notify => FinishedAction::Close,
                FinishedAction::Close => FinishedAction::GoToNext,
                FinishedAction::GoToNext => FinishedAction::Notify,
            };
            let new_text = match context.settings.reader.finished {
                FinishedAction::Notify => "Notify",
                FinishedAction::Close => "Close",
                FinishedAction::GoToNext => "Go To Next",
            };
            if let Some(btn) = children[offset + 3].downcast_mut::<Button>() {
                btn.update(new_text.to_string(), rq);
            }
            true
        }
        Event::Select(EntryId::CycleLanguage) => {
            use crate::i18n::Language;
            context.settings.language = if context.settings.language == "es" {
                "en".to_string()
            } else {
                "es".to_string()
            };
            if let Some(lang) = Language::from_code(&context.settings.language) {
                crate::i18n::set_language(lang);
            }
            let new_text = if context.settings.language == "es" {
                "Español"
            } else {
                "English"
            };
            if let Some(btn) = children[offset + 5].downcast_mut::<Button>() {
                btn.update(new_text.to_string(), rq);
            }
            true
        }
        Event::Select(EntryId::CycleUiFont) => {
            use crate::settings::UiFont;
            context.settings.ui_font = match context.settings.ui_font {
                UiFont::SansSerif => UiFont::Serif,
                UiFont::Serif => UiFont::SansSerif,
            };
            let new_text = match context.settings.ui_font {
                UiFont::SansSerif => "Sans Serif",
                UiFont::Serif => "Serif",
            };
            if let Some(btn) = children[offset + 7].downcast_mut::<Button>() {
                btn.update(new_text.to_string(), rq);
            }
            true
        }
        Event::Select(EntryId::ToggleMangaMode) => {
            context.settings.reader.manga_mode = !context.settings.reader.manga_mode;
            if let Some(btn) = children[offset + 9].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.reader.manga_mode {
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
