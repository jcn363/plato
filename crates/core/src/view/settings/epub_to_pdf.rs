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
        "Page Size".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let page_size_text = settings.epub_to_pdf.page_size.clone();
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::CycleEpubPdfPageSize),
        page_size_text,
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
        "Margin Top (mm)".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let margin_text = format!("{:.1}", settings.epub_to_pdf.margin_top);
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::IncreaseEpubPdfMarginTop),
        margin_text,
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
        "Margin Bottom (mm)".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let margin_text = format!("{:.1}", settings.epub_to_pdf.margin_bottom);
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::IncreaseEpubPdfMarginBottom),
        margin_text,
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
        "Margin Left (mm)".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let margin_text = format!("{:.1}", settings.epub_to_pdf.margin_left);
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::IncreaseEpubPdfMarginLeft),
        margin_text,
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
        "Margin Right (mm)".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let margin_text = format!("{:.1}", settings.epub_to_pdf.margin_right);
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::IncreaseEpubPdfMarginRight),
        margin_text,
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
        "Embed Fonts".to_string(),
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
        Event::Select(EntryId::ToggleEpubPdfEmbedFonts),
        if settings.epub_to_pdf.embed_fonts {
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
        "Image Quality".to_string(),
        Align::Right(padding / 2),
    );
    children.push(Box::new(label) as Box<dyn View>);

    let quality_text = format!("{}", settings.epub_to_pdf.image_quality);
    let ctrl_rect = rect![
        rect.min.x + max_label_width + 2 * padding,
        y,
        rect.max.x - padding,
        y + small_height
    ];
    let toggle = Button::new(
        ctrl_rect,
        Event::Select(EntryId::IncreaseEpubPdfImageQuality),
        quality_text,
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
        Event::Select(EntryId::CycleEpubPdfPageSize) => {
            let sizes = ["A4", "A5", "letter", "custom"];
            let current = &context.settings.epub_to_pdf.page_size;
            let current_idx = sizes.iter().position(|&s| s == current).unwrap_or(0);
            let next_idx = (current_idx + 1) % sizes.len();
            context.settings.epub_to_pdf.page_size = sizes[next_idx].to_string();
            if let Some(btn) = children[offset + 1].downcast_mut::<Button>() {
                btn.update(sizes[next_idx].to_string(), rq);
            }
            true
        }
        Event::Select(EntryId::IncreaseEpubPdfMarginTop) => {
            context.settings.epub_to_pdf.margin_top =
                (context.settings.epub_to_pdf.margin_top + 5.0).min(50.0);
            let new_text = format!("{:.1}", context.settings.epub_to_pdf.margin_top);
            if let Some(btn) = children[offset + 3].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        Event::Select(EntryId::DecreaseEpubPdfMarginTop) => {
            context.settings.epub_to_pdf.margin_top =
                (context.settings.epub_to_pdf.margin_top - 5.0).max(0.0);
            let new_text = format!("{:.1}", context.settings.epub_to_pdf.margin_top);
            if let Some(btn) = children[offset + 3].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        Event::Select(EntryId::IncreaseEpubPdfMarginBottom) => {
            context.settings.epub_to_pdf.margin_bottom =
                (context.settings.epub_to_pdf.margin_bottom + 5.0).min(50.0);
            let new_text = format!("{:.1}", context.settings.epub_to_pdf.margin_bottom);
            if let Some(btn) = children[offset + 5].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        Event::Select(EntryId::DecreaseEpubPdfMarginBottom) => {
            context.settings.epub_to_pdf.margin_bottom =
                (context.settings.epub_to_pdf.margin_bottom - 5.0).max(0.0);
            let new_text = format!("{:.1}", context.settings.epub_to_pdf.margin_bottom);
            if let Some(btn) = children[offset + 5].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        Event::Select(EntryId::IncreaseEpubPdfMarginLeft) => {
            context.settings.epub_to_pdf.margin_left =
                (context.settings.epub_to_pdf.margin_left + 5.0).min(50.0);
            let new_text = format!("{:.1}", context.settings.epub_to_pdf.margin_left);
            if let Some(btn) = children[offset + 7].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        Event::Select(EntryId::DecreaseEpubPdfMarginLeft) => {
            context.settings.epub_to_pdf.margin_left =
                (context.settings.epub_to_pdf.margin_left - 5.0).max(0.0);
            let new_text = format!("{:.1}", context.settings.epub_to_pdf.margin_left);
            if let Some(btn) = children[offset + 7].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        Event::Select(EntryId::IncreaseEpubPdfMarginRight) => {
            context.settings.epub_to_pdf.margin_right =
                (context.settings.epub_to_pdf.margin_right + 5.0).min(50.0);
            let new_text = format!("{:.1}", context.settings.epub_to_pdf.margin_right);
            if let Some(btn) = children[offset + 9].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        Event::Select(EntryId::DecreaseEpubPdfMarginRight) => {
            context.settings.epub_to_pdf.margin_right =
                (context.settings.epub_to_pdf.margin_right - 5.0).max(0.0);
            let new_text = format!("{:.1}", context.settings.epub_to_pdf.margin_right);
            if let Some(btn) = children[offset + 9].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        Event::Select(EntryId::ToggleEpubPdfEmbedFonts) => {
            context.settings.epub_to_pdf.embed_fonts = !context.settings.epub_to_pdf.embed_fonts;
            if let Some(btn) = children[offset + 11].downcast_mut::<Button>() {
                btn.update(
                    if context.settings.epub_to_pdf.embed_fonts {
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
        Event::Select(EntryId::IncreaseEpubPdfImageQuality) => {
            context.settings.epub_to_pdf.image_quality =
                (context.settings.epub_to_pdf.image_quality + 10).min(100);
            let new_text = format!("{}", context.settings.epub_to_pdf.image_quality);
            if let Some(btn) = children[offset + 13].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        Event::Select(EntryId::DecreaseEpubPdfImageQuality) => {
            context.settings.epub_to_pdf.image_quality =
                (context.settings.epub_to_pdf.image_quality - 10).max(10);
            let new_text = format!("{}", context.settings.epub_to_pdf.image_quality);
            if let Some(btn) = children[offset + 13].downcast_mut::<Button>() {
                btn.update(new_text, rq);
            }
            true
        }
        _ => false,
    }
}
