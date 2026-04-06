use crate::context::Context;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::metadata::TextAlign;
use crate::view::icon::Icon;
use crate::view::labeled_icon::LabeledIcon;
use crate::view::slider::Slider;
use crate::view::{Hub, RenderData, RenderQueue, View};

pub(super) fn resize_reflowable_children(
    children: &mut [Box<dyn View>],
    rect: Rectangle,
    side: i32,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    let mut index = 0;
    let mut remaining_width = rect.width() as i32 - 3 * side;
    let font_family_label_width = remaining_width / 2;
    remaining_width -= font_family_label_width;
    let margin_label_width = remaining_width / 2;
    let line_height_label_width = remaining_width - margin_label_width;

    // First row.
    let mut x_offset = rect.min.x;
    children[index].resize(
        rect![
            x_offset,
            rect.min.y,
            x_offset + side + margin_label_width,
            rect.min.y + side
        ],
        hub,
        rq,
        context,
    );
    index += 1;
    x_offset += side + margin_label_width;

    children[index].resize(
        rect![
            x_offset,
            rect.min.y,
            x_offset + side + font_family_label_width,
            rect.min.y + side
        ],
        hub,
        rq,
        context,
    );
    index += 1;
    x_offset += side + font_family_label_width;

    children[index].resize(
        rect![
            x_offset,
            rect.min.y,
            x_offset + side + line_height_label_width,
            rect.min.y + side
        ],
        hub,
        rq,
        context,
    );
    index += 1;

    // Separator.
    children[index].resize(
        rect![rect.min.x, rect.min.y + side, rect.max.x, rect.max.y - side],
        hub,
        rq,
        context,
    );
    index += 1;

    // Start of second row.
    let text_align_rect = rect![rect.min.x, rect.max.y - side, rect.min.x + side, rect.max.y];
    children[index].resize(text_align_rect, hub, rq, context);
    index += 1;

    let font_size_rect = rect![
        rect.min.x + side,
        rect.max.y - side,
        rect.min.x + 2 * side,
        rect.max.y
    ];
    children[index].resize(font_size_rect, hub, rq, context);
    index += 1;

    children[index].resize(
        rect![
            rect.min.x + 2 * side,
            rect.max.y - side,
            rect.max.x - 2 * side,
            rect.max.y
        ],
        hub,
        rq,
        context,
    );
}

pub(super) fn resize_fixed_children(
    children: &mut [Box<dyn View>],
    rect: Rectangle,
    side: i32,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    let mut index = 0;
    let remaining_width = rect.width() as i32 - 2 * side;
    let slider_width = remaining_width / 2;

    // First row.
    let contrast_icon_rect = rect![rect.min.x, rect.min.y, rect.min.x + side, rect.min.y + side];
    children[index].resize(contrast_icon_rect, hub, rq, context);
    index += 1;

    children[index].resize(
        rect![
            rect.min.x + side,
            rect.min.y,
            rect.min.x + side + slider_width,
            rect.min.y + side
        ],
        hub,
        rq,
        context,
    );
    index += 1;

    let gray_icon_rect = rect![
        rect.min.x + side + slider_width,
        rect.min.y,
        rect.min.x + 2 * side + slider_width,
        rect.min.y + side
    ];
    children[index].resize(gray_icon_rect, hub, rq, context);
    index += 1;

    children[index].resize(
        rect![
            rect.min.x + 2 * side + slider_width,
            rect.min.y,
            rect.max.x - side / 3,
            rect.min.y + side
        ],
        hub,
        rq,
        context,
    );
    index += 1;

    children[index].resize(
        rect![
            rect.max.x - side / 3,
            rect.min.y,
            rect.max.x,
            rect.min.y + side
        ],
        hub,
        rq,
        context,
    );
    index += 1;

    // Separator.
    children[index].resize(
        rect![rect.min.x, rect.min.y + side, rect.max.x, rect.max.y - side],
        hub,
        rq,
        context,
    );
    index += 1;

    // Start of second row.
    children[index].resize(
        rect![rect.min.x, rect.max.y - side, rect.min.x + side, rect.max.y],
        hub,
        rq,
        context,
    );
    index += 1;

    let remaining_width = rect.width() as i32 - 3 * side;
    let margin_label_width = children[index + 1].rect().width() as i32;
    let big_padding = (remaining_width - margin_label_width) / 2;
    let small_padding = remaining_width - margin_label_width - big_padding;

    children[index].resize(
        rect![
            rect.min.x + side,
            rect.max.y - side,
            rect.min.x + side + small_padding,
            rect.max.y
        ],
        hub,
        rq,
        context,
    );
    index += 1;

    children[index].resize(
        rect![
            rect.min.x + side + small_padding,
            rect.max.y - side,
            rect.max.x - 2 * side - big_padding,
            rect.max.y
        ],
        hub,
        rq,
        context,
    );
    index += 1;

    children[index].resize(
        rect![
            rect.max.x - 2 * side - big_padding,
            rect.max.y - side,
            rect.max.x - 2 * side,
            rect.max.y
        ],
        hub,
        rq,
        context,
    );
}

pub(super) fn resize_common_children(
    children: &mut [Box<dyn View>],
    rect: Rectangle,
    side: i32,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    let mut index = 0;

    children[index].resize(
        rect![
            rect.max.x - 2 * side,
            rect.max.y - side,
            rect.max.x - side,
            rect.max.y
        ],
        hub,
        rq,
        context,
    );
    index += 1;

    children[index].resize(
        rect![rect.max.x - side, rect.max.y - side, rect.max.x, rect.max.y],
        hub,
        rq,
        context,
    );
}

pub(super) fn update_margin_width(
    children: &mut Vec<Box<dyn View>>,
    margin_width: i32,
    rq: &mut RenderQueue,
    reflowable: bool,
) {
    let index = if reflowable { 0 } else { 8 };
    if let Some(labeled_icon) = children[index].downcast_mut::<LabeledIcon>() {
        labeled_icon.update(&format!("{} mm", margin_width), rq);
    }
}

pub(super) fn update_font_family(
    children: &mut Vec<Box<dyn View>>,
    font_family: String,
    rq: &mut RenderQueue,
) {
    if let Some(labeled_icon) = children[1].downcast_mut::<LabeledIcon>() {
        labeled_icon.update(&font_family, rq);
    }
}

pub(super) fn update_line_height(
    children: &mut Vec<Box<dyn View>>,
    line_height: f32,
    rq: &mut RenderQueue,
) {
    if let Some(labeled_icon) = children[2].downcast_mut::<LabeledIcon>() {
        labeled_icon.update(&format!("{:.1} em", line_height), rq);
    }
}

pub(super) fn update_text_align_icon(
    children: &mut Vec<Box<dyn View>>,
    text_align: TextAlign,
    rq: &mut RenderQueue,
) {
    let Some(icon) = children[4].as_mut().downcast_mut::<Icon>() else {
        return;
    };
    let name = text_align.icon_name();
    if icon.name != name {
        icon.name = name.to_string();
        rq.add(RenderData::new(icon.id(), *icon.rect(), UpdateMode::Gui));
    }
}

pub(super) fn update_font_size_slider(
    children: &mut Vec<Box<dyn View>>,
    font_size: f32,
    rq: &mut RenderQueue,
) {
    let Some(slider) = children[6].as_mut().downcast_mut::<Slider>() else {
        return;
    };
    slider.update(font_size, rq);
}

pub(super) fn update_contrast_exponent_slider(
    children: &mut Vec<Box<dyn View>>,
    exponent: f32,
    rq: &mut RenderQueue,
) {
    let Some(slider) = children[1].as_mut().downcast_mut::<Slider>() else {
        return;
    };
    slider.update(exponent, rq);
}

pub(super) fn update_contrast_gray_slider(
    children: &mut Vec<Box<dyn View>>,
    gray: f32,
    rq: &mut RenderQueue,
) {
    let Some(slider) = children[3].as_mut().downcast_mut::<Slider>() else {
        return;
    };
    slider.update(gray, rq);
}
