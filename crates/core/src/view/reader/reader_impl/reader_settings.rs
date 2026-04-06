//! Reader Settings Module
//!
//! Handles all font, contrast, zoom settings menus and configuration.
//!
//! ## Methods Extracted
//! - `toggle_font_family_menu()` - Font selection ✓
//! - `toggle_font_size_menu()` - Font size selection ✓
//! - `toggle_text_align_menu()` - Text alignment ✓
//! - `toggle_line_height_menu()` - Line height settings ✓
//! - `toggle_contrast_exponent_menu()` - Contrast exponent ✓
//! - `toggle_contrast_gray_menu()` - Contrast gray level ✓
//! - `find_page_by_name()` - Page lookup utility ✓
//! - `build_toc()` - TOC building ✓
//! - `build_toc_aux()` - TOC recursive builder ✓

use crate::document::{Location, SimpleTocEntry, TocEntry, TocLocation};
use crate::font::family_names;
use crate::framebuffer::UpdateMode;
use crate::geom::Rectangle;
use crate::helpers::AsciiExtension;
use crate::log_error;
use crate::metadata::Info;
use crate::metadata::TextAlign;
use crate::settings::DEFAULT_FONT_FAMILY;
use crate::view::{EntryId, EntryKind, RenderData, RenderQueue, View, ViewId};
use septem::Roman;

/// Find page index by named page reference
#[allow(dead_code)]
pub(crate) fn find_page_by_name(info: &Info, name: &str) -> Option<usize> {
    info.reader.as_ref().and_then(|r| {
        if let Ok(a) = name.parse::<u32>() {
            r.page_names
                .iter()
                .filter_map(|(i, s)| s.parse::<u32>().ok().map(|b| (b, i)))
                .filter(|(b, _)| *b <= a)
                .max_by(|x, y| x.0.cmp(&y.0))
                .map(|(b, i)| *i + (a - b) as usize)
        } else if let Some(a) = name.chars().next().and_then(|c| c.to_alphabetic_digit()) {
            r.page_names
                .iter()
                .filter_map(|(i, s)| {
                    s.chars()
                        .next()
                        .and_then(|c| c.to_alphabetic_digit())
                        .map(|c| (c, i))
                })
                .filter(|(b, _)| *b <= a)
                .max_by(|x, y| x.0.cmp(&y.0))
                .map(|(b, i)| *i + (a - b) as usize)
        } else if let Ok(a) = name.parse::<Roman>() {
            let a_val = *a;
            r.page_names
                .iter()
                .filter_map(|(i, s)| s.parse::<Roman>().ok().map(|b| (b, i)))
                .filter(|(b, _)| {
                    (*b).cmp(&Roman::from_unchecked(a_val)) != std::cmp::Ordering::Greater
                })
                .max_by(|x, y| x.0.cmp(&y.0))
                .map(|(b, i)| *i + (a_val - *b) as usize)
        } else {
            None
        }
    })
}

/// Build table of contents from document structure
#[allow(dead_code)]
pub(crate) fn build_toc<F>(info: &Info, find_page_fn: F) -> Option<Vec<TocEntry>>
where
    F: Fn(&str) -> Option<usize> + Copy,
{
    let mut index = 0;
    info.toc
        .as_ref()
        .map(|simple_toc| build_toc_aux(simple_toc, &mut index, find_page_fn))
}

/// Recursively build table of contents entries
#[allow(dead_code)]
pub(crate) fn build_toc_aux<F>(
    simple_toc: &[SimpleTocEntry],
    index: &mut usize,
    find_page_fn: F,
) -> Vec<TocEntry>
where
    F: Fn(&str) -> Option<usize> + Copy,
{
    let mut toc = Vec::with_capacity(simple_toc.len());
    for entry in simple_toc {
        *index += 1;
        match entry {
            SimpleTocEntry::Leaf(title, location)
            | SimpleTocEntry::Container(title, location, _) => {
                let current_title = title.clone();
                let current_location = match location {
                    TocLocation::Uri(uri) if uri.starts_with('\'') => find_page_fn(&uri[1..])
                        .map(Location::Exact)
                        .unwrap_or_else(|| location.clone().into()),
                    _ => location.clone().into(),
                };
                let current_index = *index;
                let current_children = if let SimpleTocEntry::Container(_, _, children) = entry {
                    build_toc_aux(children, index, find_page_fn)
                } else {
                    Vec::new()
                };
                toc.push(TocEntry {
                    title: current_title,
                    location: current_location,
                    index: current_index,
                    children: current_children,
                });
            }
        }
    }
    toc
}

/// Toggle font family menu visibility
#[allow(dead_code)]
pub(crate) fn toggle_font_family_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_family: String,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    use crate::view::menu::{Menu, MenuKind};

    if let Some(index) = children
        .iter()
        .position(|c| c.view_id().map_or(false, |i| i == ViewId::FontFamilyMenu))
    {
        if let Some(true) = enable {
            return;
        }
        rq.add(RenderData::expose(*children[index].rect(), UpdateMode::Gui));
        children.remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
        let mut families = family_names(&context.settings.reader.font_path)
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
        let font_family_menu = Menu::new(
            rect,
            ViewId::FontFamilyMenu,
            MenuKind::DropDown,
            entries,
            context,
        );
        rq.add(RenderData::new(
            font_family_menu.id(),
            *font_family_menu.rect(),
            UpdateMode::Gui,
        ));
        children.push(Box::new(font_family_menu) as Box<dyn crate::view::View>);
    }
}

/// Toggle font size menu visibility
#[allow(dead_code)]
pub(crate) fn toggle_font_size_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_size: f32,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    use crate::view::menu::{Menu, MenuKind};

    if let Some(index) = children
        .iter()
        .position(|c| c.view_id().map_or(false, |i| i == ViewId::FontSizeMenu))
    {
        if let Some(true) = enable {
            return;
        }
        rq.add(RenderData::expose(*children[index].rect(), UpdateMode::Gui));
        children.remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
        let min_font_size = context.settings.reader.font_size / 2.0;
        let max_font_size = 3.0 * context.settings.reader.font_size / 2.0;
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
        let font_size_menu = Menu::new(
            rect,
            ViewId::FontSizeMenu,
            MenuKind::Contextual,
            entries,
            context,
        );
        rq.add(RenderData::new(
            font_size_menu.id(),
            *font_size_menu.rect(),
            UpdateMode::Gui,
        ));
        children.push(Box::new(font_size_menu) as Box<dyn crate::view::View>);
    }
}

/// Toggle text alignment menu visibility
#[allow(dead_code)]
pub(crate) fn toggle_text_align_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_align: TextAlign,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    use crate::view::menu::{Menu, MenuKind};

    if let Some(index) = children
        .iter()
        .position(|c| c.view_id().map_or(false, |i| i == ViewId::TextAlignMenu))
    {
        if let Some(true) = enable {
            return;
        }
        rq.add(RenderData::expose(*children[index].rect(), UpdateMode::Gui));
        children.remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
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
        let text_align_menu = Menu::new(
            rect,
            ViewId::TextAlignMenu,
            MenuKind::Contextual,
            entries,
            context,
        );
        rq.add(RenderData::new(
            text_align_menu.id(),
            *text_align_menu.rect(),
            UpdateMode::Gui,
        ));
        children.push(Box::new(text_align_menu) as Box<dyn crate::view::View>);
    }
}

/// Toggle line height menu visibility
#[allow(dead_code)]
pub(crate) fn toggle_line_height_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_height: f32,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    use crate::view::menu::{Menu, MenuKind};

    if let Some(index) = children
        .iter()
        .position(|c| c.view_id().map_or(false, |i| i == ViewId::LineHeightMenu))
    {
        if let Some(true) = enable {
            return;
        }
        rq.add(RenderData::expose(*children[index].rect(), UpdateMode::Gui));
        children.remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
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
        let line_height_menu = Menu::new(
            rect,
            ViewId::LineHeightMenu,
            MenuKind::DropDown,
            entries,
            context,
        );
        rq.add(RenderData::new(
            line_height_menu.id(),
            *line_height_menu.rect(),
            UpdateMode::Gui,
        ));
        children.push(Box::new(line_height_menu) as Box<dyn crate::view::View>);
    }
}

/// Toggle contrast exponent menu visibility
#[allow(dead_code)]
pub(crate) fn toggle_contrast_exponent_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_exponent: f32,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    use crate::view::menu::{Menu, MenuKind};

    if let Some(index) = children.iter().position(|c| {
        c.view_id()
            .map_or(false, |i| i == ViewId::ContrastExponentMenu)
    }) {
        if let Some(true) = enable {
            return;
        }
        rq.add(RenderData::expose(*children[index].rect(), UpdateMode::Gui));
        children.remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
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
        let contrast_exponent_menu = Menu::new(
            rect,
            ViewId::ContrastExponentMenu,
            MenuKind::DropDown,
            entries,
            context,
        );
        rq.add(RenderData::new(
            contrast_exponent_menu.id(),
            *contrast_exponent_menu.rect(),
            UpdateMode::Gui,
        ));
        children.push(Box::new(contrast_exponent_menu) as Box<dyn crate::view::View>);
    }
}

/// Toggle contrast gray level menu visibility
#[allow(dead_code)]
pub(crate) fn toggle_contrast_gray_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_gray: f32,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    use crate::view::menu::{Menu, MenuKind};

    if let Some(index) = children
        .iter()
        .position(|c| c.view_id().map_or(false, |i| i == ViewId::ContrastGrayMenu))
    {
        if let Some(true) = enable {
            return;
        }
        rq.add(RenderData::expose(*children[index].rect(), UpdateMode::Gui));
        children.remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
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
        let contrast_gray_menu = Menu::new(
            rect,
            ViewId::ContrastGrayMenu,
            MenuKind::DropDown,
            entries,
            context,
        );
        rq.add(RenderData::new(
            contrast_gray_menu.id(),
            *contrast_gray_menu.rect(),
            UpdateMode::Gui,
        ));
        children.push(Box::new(contrast_gray_menu) as Box<dyn crate::view::View>);
    }
}
