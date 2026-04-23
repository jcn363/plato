//! Menu Toggle Functions
//!
//! Functions to toggle various reader settings menus.

use crate::font::family_names;
use crate::geom::Rectangle;
use crate::log_error;
use crate::metadata::{CroppingMargins, Info, PageScheme, TextAlign};
use crate::settings::DEFAULT_FONT_FAMILY;
use crate::view::menu::{Menu, MenuKind};
use crate::view::menu_helpers::toggle_menu_vec;
use crate::view::{EntryId, EntryKind, RenderQueue, ViewId};

pub(crate) fn toggle_font_family_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_family: String,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    toggle_menu_vec(
        ViewId::FontFamilyMenu,
        |ctx| {
            let mut families = family_names(&ctx.settings.reader.font_path)
                .map_err(|e| log_error!("Can't get family names: {:#}.", e))
                .unwrap_or_default();
            families.insert(DEFAULT_FONT_FAMILY.to_string());
            let entries: Vec<_> = families
                .iter()
                .map(|f| {
                    EntryKind::RadioButton(
                        f.clone(),
                        EntryId::SetFontFamily(f.clone()),
                        *f == current_family,
                    )
                })
                .collect();
            Menu::new(
                rect,
                ViewId::FontFamilyMenu,
                MenuKind::DropDown,
                entries,
                ctx,
            )
        },
        children,
        enable,
        rq,
        context,
    );
}

pub(crate) fn toggle_font_size_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_size: f32,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    let min_font_size = context.settings.reader.font_size / 2.0;
    let max_font_size = 3.0 * context.settings.reader.font_size / 2.0;

    toggle_menu_vec(
        ViewId::FontSizeMenu,
        |ctx| {
            let entries: Vec<_> = (0..=20)
                .filter_map(|v| {
                    let fs = current_size - 1.0 + v as f32 / 10.0;
                    if fs >= min_font_size && fs <= max_font_size {
                        Some(EntryKind::RadioButton(
                            format!("{:.1}", fs),
                            EntryId::SetFontSize(v),
                            (fs - current_size).abs() < 0.05,
                        ))
                    } else {
                        None
                    }
                })
                .collect();
            Menu::new(
                rect,
                ViewId::FontSizeMenu,
                MenuKind::Contextual,
                entries,
                ctx,
            )
        },
        children,
        enable,
        rq,
        context,
    );
}

pub(crate) fn toggle_text_align_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_align: TextAlign,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    toggle_menu_vec(
        ViewId::TextAlignMenu,
        |ctx| {
            let choices = [
                TextAlign::Justify,
                TextAlign::Left,
                TextAlign::Right,
                TextAlign::Center,
            ];
            let entries: Vec<_> = choices
                .iter()
                .map(|v| {
                    EntryKind::RadioButton(
                        v.to_string(),
                        EntryId::SetTextAlign(*v),
                        current_align == *v,
                    )
                })
                .collect();
            Menu::new(
                rect,
                ViewId::TextAlignMenu,
                MenuKind::Contextual,
                entries,
                ctx,
            )
        },
        children,
        enable,
        rq,
        context,
    );
}

pub(crate) fn toggle_line_height_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_height: f32,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    toggle_menu_vec(
        ViewId::LineHeightMenu,
        |ctx| {
            let entries: Vec<_> = (0..=10)
                .map(|x| {
                    let lh = 1.0 + x as f32 / 10.0;
                    EntryKind::RadioButton(
                        format!("{:.1}", lh),
                        EntryId::SetLineHeight(x),
                        (lh - current_height).abs() < 0.05,
                    )
                })
                .collect();
            Menu::new(
                rect,
                ViewId::LineHeightMenu,
                MenuKind::DropDown,
                entries,
                ctx,
            )
        },
        children,
        enable,
        rq,
        context,
    );
}

pub(crate) fn toggle_contrast_exponent_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_exponent: f32,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    toggle_menu_vec(
        ViewId::ContrastExponentMenu,
        |ctx| {
            let entries: Vec<_> = (0..=8)
                .map(|x| {
                    let e = 1.0 + x as f32 / 2.0;
                    EntryKind::RadioButton(
                        format!("{:.1}", e),
                        EntryId::SetContrastExponent(x),
                        (e - current_exponent).abs() < f32::EPSILON,
                    )
                })
                .collect();
            Menu::new(
                rect,
                ViewId::ContrastExponentMenu,
                MenuKind::DropDown,
                entries,
                ctx,
            )
        },
        children,
        enable,
        rq,
        context,
    );
}

pub(crate) fn toggle_contrast_gray_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_gray: f32,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    toggle_menu_vec(
        ViewId::ContrastGrayMenu,
        |ctx| {
            let entries: Vec<_> = (1..=6)
                .map(|x| {
                    let g = ((1 << 8) - (1 << (8 - x))) as f32;
                    EntryKind::RadioButton(
                        format!("{:.1}", g),
                        EntryId::SetContrastGray(x),
                        (g - current_gray).abs() < f32::EPSILON,
                    )
                })
                .collect();
            Menu::new(
                rect,
                ViewId::ContrastGrayMenu,
                MenuKind::DropDown,
                entries,
                ctx,
            )
        },
        children,
        enable,
        rq,
        context,
    );
}

pub(crate) fn toggle_margin_width_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_margin_width: i32,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    let min_margin_width = context.settings.reader.min_margin_width;
    let max_margin_width = context.settings.reader.max_margin_width;

    toggle_menu_vec(
        ViewId::MarginWidthMenu,
        |ctx| {
            let entries: Vec<_> = (min_margin_width..=max_margin_width)
                .map(|mw| {
                    EntryKind::RadioButton(
                        format!("{}", mw),
                        EntryId::SetMarginWidth(mw),
                        mw == current_margin_width,
                    )
                })
                .collect();
            Menu::new(
                rect,
                ViewId::MarginWidthMenu,
                MenuKind::DropDown,
                entries,
                ctx,
            )
        },
        children,
        enable,
        rq,
        context,
    );
}

pub(crate) fn toggle_page_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_page: usize,
    info: &Info,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    let has_name = info
        .reader
        .as_ref()
        .map(|r| r.page_names.contains_key(&current_page))
        .unwrap_or(false);

    let mut entries = vec![EntryKind::Command("Name".to_string(), EntryId::SetPageName)];
    if has_name {
        entries.push(EntryKind::Command(
            "Remove Name".to_string(),
            EntryId::RemovePageName,
        ));
    }
    let names = info
        .reader
        .as_ref()
        .map(|r| {
            r.page_names
                .iter()
                .map(|(i, s)| EntryKind::Command(s.to_string(), EntryId::GoTo(*i)))
                .collect::<Vec<EntryKind>>()
        })
        .unwrap_or_default();
    if !names.is_empty() {
        entries.push(EntryKind::Separator);
        entries.push(EntryKind::SubMenu("Go To".to_string(), names));
    }

    let create_menu = |ctx: &mut crate::context::Context| -> Menu {
        Menu::new(rect, ViewId::PageMenu, MenuKind::DropDown, entries, ctx)
    };

    toggle_menu_vec(ViewId::PageMenu, create_menu, children, enable, rq, context);
}

pub(crate) fn toggle_margin_cropper_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_page: usize,
    info: &Info,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    let is_split = info
        .reader
        .as_ref()
        .and_then(|r| r.cropping_margins.as_ref().map(CroppingMargins::is_split));

    let (any_selected, even_odd_selected) = match is_split {
        Some(true) => (false, true),
        Some(false) => (true, false),
        None => (false, false),
    };

    let mut entries = vec![
        EntryKind::RadioButton(
            "Any".to_string(),
            EntryId::ApplyCroppings(current_page, PageScheme::Any),
            any_selected,
        ),
        EntryKind::RadioButton(
            "Even/Odd".to_string(),
            EntryId::ApplyCroppings(current_page, PageScheme::EvenOdd),
            even_odd_selected,
        ),
    ];

    let is_applied = info
        .reader
        .as_ref()
        .map(|r| r.cropping_margins.is_some())
        .unwrap_or(false);
    if is_applied {
        entries.extend_from_slice(&[
            EntryKind::Separator,
            EntryKind::Command("Remove".to_string(), EntryId::RemoveCroppings),
        ]);
    }

    let create_menu = |ctx: &mut crate::context::Context| -> Menu {
        Menu::new(
            rect,
            ViewId::MarginCropperMenu,
            MenuKind::DropDown,
            entries,
            ctx,
        )
    };

    toggle_menu_vec(
        ViewId::MarginCropperMenu,
        create_menu,
        children,
        enable,
        rq,
        context,
    );
}
