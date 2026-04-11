use crate::color::separator as sep;
use crate::device::CURRENT_DEVICE;
use crate::geom::Rectangle;
use crate::metadata::ReaderInfo;
use crate::metadata::{DEFAULT_CONTRAST_EXPONENT, DEFAULT_CONTRAST_GRAY};
use crate::settings::ReaderSettings;
use crate::theme;
use crate::unit::scale_by_dpi;
use crate::view::filler::Filler;
use crate::view::icon::Icon;
use crate::view::labeled_icon::LabeledIcon;
use crate::view::slider::Slider;
use crate::view::{Event, SliderId, View, ViewId, THICKNESS_MEDIUM};

pub(super) fn build_reflowable_children(
    rect: Rectangle,
    reader_info: Option<&ReaderInfo>,
    reader_settings: &ReaderSettings,
    side: i32,
) -> Vec<Box<dyn View>> {
    let mut children = Vec::new();

    let mut remaining_width = rect.width() as i32 - 3 * side;
    let font_family_label_width = remaining_width / 2;
    remaining_width -= font_family_label_width;
    let margin_label_width = remaining_width / 2;
    let line_height_label_width = remaining_width - margin_label_width;

    // First row.
    let mut x_offset = rect.min.x;

    let margin_width = reader_info
        .and_then(|r| r.margin_width)
        .unwrap_or(reader_settings.margin_width);
    let margin_icon = LabeledIcon::new(
        "margin",
        rect![
            x_offset,
            rect.min.y,
            x_offset + side + margin_label_width,
            rect.min.y + side
        ],
        Event::Show(ViewId::MarginWidthMenu),
        format!("{} mm", margin_width),
    );
    children.push(Box::new(margin_icon) as Box<dyn View>);
    x_offset += side + margin_label_width;

    let font_family = reader_info
        .and_then(|r| r.font_family.clone())
        .unwrap_or_else(|| reader_settings.font_family.clone());
    let font_family_icon = LabeledIcon::new(
        "font_family",
        rect![
            x_offset,
            rect.min.y,
            x_offset + side + font_family_label_width,
            rect.min.y + side
        ],
        Event::Show(ViewId::FontFamilyMenu),
        font_family,
    );
    children.push(Box::new(font_family_icon) as Box<dyn View>);
    x_offset += side + font_family_label_width;

    let line_height = reader_info
        .and_then(|r| r.line_height)
        .unwrap_or(reader_settings.line_height);
    let line_height_icon = LabeledIcon::new(
        "line_height",
        rect![
            x_offset,
            rect.min.y,
            x_offset + side + line_height_label_width,
            rect.min.y + side
        ],
        Event::Show(ViewId::LineHeightMenu),
        format!("{:.1} em", line_height),
    );
    children.push(Box::new(line_height_icon) as Box<dyn View>);

    // Separator.
    let separator = Filler::new(
        rect![rect.min.x, rect.min.y + side, rect.max.x, rect.max.y - side],
        sep(theme::is_dark_mode()),
    );
    children.push(Box::new(separator) as Box<dyn View>);

    // Start of second row.
    let text_align = reader_info
        .and_then(|r| r.text_align)
        .unwrap_or(reader_settings.text_align);
    let text_align_rect = rect![rect.min.x, rect.max.y - side, rect.min.x + side, rect.max.y];
    let text_align_icon = Icon::new(
        text_align.icon_name(),
        text_align_rect,
        Event::ToggleNear(ViewId::TextAlignMenu, text_align_rect),
    );
    children.push(Box::new(text_align_icon) as Box<dyn View>);

    let font_size = reader_info
        .and_then(|r| r.font_size)
        .unwrap_or(reader_settings.font_size);
    let font_size_rect = rect![
        rect.min.x + side,
        rect.max.y - side,
        rect.min.x + 2 * side,
        rect.max.y
    ];
    let font_size_icon = Icon::new(
        "font_size",
        font_size_rect,
        Event::ToggleNear(ViewId::FontSizeMenu, font_size_rect),
    );
    children.push(Box::new(font_size_icon) as Box<dyn View>);

    let slider = Slider::new(
        rect![
            rect.min.x + 2 * side,
            rect.max.y - side,
            rect.max.x - 2 * side,
            rect.max.y
        ],
        SliderId::FontSize,
        font_size,
        reader_settings.min_font_size,
        reader_settings.max_font_size,
    );
    children.push(Box::new(slider) as Box<dyn View>);

    children
}

pub(super) fn build_fixed_children(
    rect: Rectangle,
    reader_info: Option<&ReaderInfo>,
    side: i32,
) -> Vec<Box<dyn View>> {
    let mut children = Vec::new();

    let remaining_width = rect.width() as i32 - 2 * side;
    let slider_width = remaining_width / 2;

    // First row.
    let contrast_icon_rect = rect![rect.min.x, rect.min.y, rect.min.x + side, rect.min.y + side];
    let contrast_icon = Icon::new(
        "contrast",
        contrast_icon_rect,
        Event::ToggleNear(ViewId::ContrastExponentMenu, contrast_icon_rect),
    );
    children.push(Box::new(contrast_icon) as Box<dyn View>);

    let contrast_exponent = reader_info
        .and_then(|r| r.contrast_exponent)
        .unwrap_or(DEFAULT_CONTRAST_EXPONENT);
    let slider = Slider::new(
        rect![
            rect.min.x + side,
            rect.min.y,
            rect.min.x + side + slider_width,
            rect.min.y + side
        ],
        SliderId::ContrastExponent,
        contrast_exponent,
        1.0,
        5.0,
    );
    children.push(Box::new(slider) as Box<dyn View>);

    let gray_icon_rect = rect![
        rect.min.x + side + slider_width,
        rect.min.y,
        rect.min.x + 2 * side + slider_width,
        rect.min.y + side
    ];
    let gray_icon = Icon::new(
        "gray",
        gray_icon_rect,
        Event::ToggleNear(ViewId::ContrastGrayMenu, gray_icon_rect),
    );
    children.push(Box::new(gray_icon) as Box<dyn View>);

    let contrast_gray = reader_info
        .and_then(|r| r.contrast_gray)
        .unwrap_or(DEFAULT_CONTRAST_GRAY);
    let slider = Slider::new(
        rect![
            rect.min.x + 2 * side + slider_width,
            rect.min.y,
            rect.max.x - side / 3,
            rect.min.y + side
        ],
        SliderId::ContrastGray,
        contrast_gray,
        0.0,
        255.0,
    );
    children.push(Box::new(slider) as Box<dyn View>);

    let filler = Filler::new(
        rect![
            rect.max.x - side / 3,
            rect.min.y,
            rect.max.x,
            rect.min.y + side
        ],
        crate::color::background(theme::is_dark_mode()),
    );
    children.push(Box::new(filler) as Box<dyn View>);

    // Separator.
    let separator = Filler::new(
        rect![rect.min.x, rect.min.y + side, rect.max.x, rect.max.y - side],
        sep(theme::is_dark_mode()),
    );
    children.push(Box::new(separator) as Box<dyn View>);

    // Start of second row.
    let crop_icon = Icon::new(
        "crop",
        rect![rect.min.x, rect.max.y - side, rect.min.x + side, rect.max.y],
        Event::Show(ViewId::MarginCropper),
    );
    children.push(Box::new(crop_icon) as Box<dyn View>);

    let remaining_width = rect.width() as i32 - 3 * side;
    let margin_label_width = (2 * side).min(remaining_width);
    let big_padding = (remaining_width - margin_label_width) / 2;
    let small_padding = remaining_width - margin_label_width - big_padding;

    let filler = Filler::new(
        rect![
            rect.min.x + side,
            rect.max.y - side,
            rect.min.x + side + small_padding,
            rect.max.y
        ],
        crate::color::background(theme::is_dark_mode()),
    );
    children.push(Box::new(filler) as Box<dyn View>);

    let margin_width = reader_info.and_then(|r| r.screen_margin_width).unwrap_or(0);
    let margin_icon = LabeledIcon::new(
        "margin",
        rect![
            rect.min.x + side + small_padding,
            rect.max.y - side,
            rect.max.x - 2 * side - big_padding,
            rect.max.y
        ],
        Event::Show(ViewId::MarginWidthMenu),
        format!("{} mm", margin_width),
    );
    children.push(Box::new(margin_icon) as Box<dyn View>);

    let filler = Filler::new(
        rect![
            rect.max.x - 2 * side - big_padding,
            rect.max.y - side,
            rect.max.x - 2 * side,
            rect.max.y
        ],
        crate::color::background(theme::is_dark_mode()),
    );
    children.push(Box::new(filler) as Box<dyn View>);

    children
}

pub(super) fn build_common_children(rect: Rectangle, side: i32) -> Vec<Box<dyn View>> {
    let mut children = Vec::new();

    let search_icon = Icon::new(
        "search",
        rect![
            rect.max.x - 2 * side,
            rect.max.y - side,
            rect.max.x - side,
            rect.max.y
        ],
        Event::Show(ViewId::SearchBar),
    );
    children.push(Box::new(search_icon) as Box<dyn View>);

    let toc_icon = Icon::new(
        "toc",
        rect![rect.max.x - side, rect.max.y - side, rect.max.x, rect.max.y],
        Event::Show(ViewId::TableOfContents),
    );
    children.push(Box::new(toc_icon) as Box<dyn View>);

    children
}

pub(super) fn calc_side(rect: Rectangle) -> i32 {
    let dpi = CURRENT_DEVICE.dpi;
    let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
    (rect.height() as i32 + thickness) / 2 - thickness
}
