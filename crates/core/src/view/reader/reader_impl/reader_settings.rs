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
//! - `toggle_margin_width_menu()` - Margin width settings ✓
//! - `toggle_page_menu()` - Page navigation menu ✓
//! - `toggle_margin_cropper_menu()` - Margin cropping settings ✓
//! - `toggle_annotation_menu()` - Annotation context menu ✓
//! - `toggle_selection_menu()` - Text selection menu ✓
//! - `toggle_title_menu()` - Title bar menu ✓
//! - `find_page_by_name()` - Page lookup utility ✓
//! - `build_toc()` - TOC building ✓
//! - `build_toc_aux()` - TOC recursive builder ✓

use crate::context::Context;
use crate::document::{Location, SimpleTocEntry, TocEntry, TocLocation};
use crate::font::family_names;
use crate::geom::Rectangle;
use crate::helpers::AsciiExtension;
use crate::log_error;
use crate::metadata::{
    Annotation, CroppingMargins, Info, PageScheme, ScrollMode, TextAlign, ZoomMode,
};
use crate::settings::DEFAULT_FONT_FAMILY;
use crate::view::menu::Menu;
use crate::view::menu_entry::MenuEntry;
use crate::view::menu_helpers::toggle_menu_vec;
use crate::view::{AppCmd, EntryId, EntryKind, RenderQueue, View, ViewId};
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
    use crate::view::menu::MenuKind;
    use crate::view::menu_helpers::toggle_menu_vec;

    let current_family_clone = current_family.clone();
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
                        *f == current_family_clone,
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
    use crate::view::menu::MenuKind;
    use crate::view::menu_helpers::toggle_menu_vec;

    let min_font_size = context.settings.reader.font_size / 2.0;
    let max_font_size = 3.0 * context.settings.reader.font_size / 2.0;
    let current_size_clone = current_size;

    toggle_menu_vec(
        ViewId::FontSizeMenu,
        |ctx| {
            let entries: Vec<_> = (0..=20)
                .filter_map(|v| {
                    let fs = current_size_clone - 1.0 + v as f32 / 10.0;
                    if fs >= min_font_size && fs <= max_font_size {
                        Some(EntryKind::RadioButton(
                            format!("{:.1}", fs),
                            EntryId::SetFontSize(v),
                            (fs - current_size_clone).abs() < 0.05,
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
    use crate::view::menu::MenuKind;
    use crate::view::menu_helpers::toggle_menu_vec;

    let current_align_clone = current_align;

    toggle_menu_vec(
        ViewId::TextAlignMenu,
        |_ctx| {
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
                        current_align_clone == *v,
                    )
                })
                .collect();
            Menu::new(
                rect,
                ViewId::TextAlignMenu,
                MenuKind::Contextual,
                entries,
                _ctx,
            )
        },
        children,
        enable,
        rq,
        context,
    );
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
    use crate::view::menu::MenuKind;
    use crate::view::menu_helpers::toggle_menu_vec;

    let current_height_clone = current_height;

    toggle_menu_vec(
        ViewId::LineHeightMenu,
        |_ctx| {
            let entries: Vec<_> = (0..=10)
                .map(|x| {
                    let lh = 1.0 + x as f32 / 10.0;
                    EntryKind::RadioButton(
                        format!("{:.1}", lh),
                        EntryId::SetLineHeight(x),
                        (lh - current_height_clone).abs() < 0.05,
                    )
                })
                .collect();
            Menu::new(
                rect,
                ViewId::LineHeightMenu,
                MenuKind::DropDown,
                entries,
                _ctx,
            )
        },
        children,
        enable,
        rq,
        context,
    );
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
    use crate::view::menu::MenuKind;
    use crate::view::menu_helpers::toggle_menu_vec;

    let current_exponent_clone = current_exponent;

    toggle_menu_vec(
        ViewId::ContrastExponentMenu,
        |_ctx| {
            let entries: Vec<_> = (0..=8)
                .map(|x| {
                    let e = 1.0 + x as f32 / 2.0;
                    EntryKind::RadioButton(
                        format!("{:.1}", e),
                        EntryId::SetContrastExponent(x),
                        (e - current_exponent_clone).abs() < f32::EPSILON,
                    )
                })
                .collect();
            Menu::new(
                rect,
                ViewId::ContrastExponentMenu,
                MenuKind::DropDown,
                entries,
                _ctx,
            )
        },
        children,
        enable,
        rq,
        context,
    );
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
    use crate::view::menu::MenuKind;
    use crate::view::menu_helpers::toggle_menu_vec;

    let current_gray_clone = current_gray;

    toggle_menu_vec(
        ViewId::ContrastGrayMenu,
        |_ctx| {
            let entries: Vec<_> = (1..=6)
                .map(|x| {
                    let g = ((1 << 8) - (1 << (8 - x))) as f32;
                    EntryKind::RadioButton(
                        format!("{:.1}", g),
                        EntryId::SetContrastGray(x),
                        (g - current_gray_clone).abs() < f32::EPSILON,
                    )
                })
                .collect();
            Menu::new(
                rect,
                ViewId::ContrastGrayMenu,
                MenuKind::DropDown,
                entries,
                _ctx,
            )
        },
        children,
        enable,
        rq,
        context,
    );
}

/// Toggle margin width menu visibility
#[allow(dead_code)]
pub(crate) fn toggle_margin_width_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_margin_width: i32,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    use crate::view::menu::MenuKind;
    use crate::view::menu_helpers::toggle_menu_vec;

    let current_margin_clone = current_margin_width;
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
                        mw == current_margin_clone,
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

/// Toggle page menu visibility
#[allow(dead_code)]
pub(crate) fn toggle_page_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_page: usize,
    info: &Info,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    use crate::view::menu::{Menu, MenuKind};

    let has_name = info
        .reader
        .as_ref()
        .map_or(false, |r| r.page_names.contains_key(&current_page));

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

/// Toggle margin cropper menu visibility
#[allow(dead_code)]
pub(crate) fn toggle_margin_cropper_menu(
    children: &mut Vec<Box<dyn crate::view::View>>,
    current_page: usize,
    info: &Info,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut crate::context::Context,
) {
    use crate::view::menu::{Menu, MenuKind};

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

// ===========================================================================
// Public Menu Functions
// ===========================================================================

/// Toggle annotation menu for a specific annotation
pub(crate) fn toggle_annotation_menu(
    children: &mut Vec<Box<dyn View>>,
    annot: &Annotation,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    use crate::view::menu::{Menu, MenuKind};

    let sel = annot.selection;
    let mut entries = Vec::new();

    if annot.note.is_empty() {
        entries.push(EntryKind::Command(
            "Remove Highlight".to_string(),
            EntryId::RemoveAnnotation(sel),
        ));
        entries.push(EntryKind::Separator);
        entries.push(EntryKind::Command(
            "Add Note".to_string(),
            EntryId::EditAnnotationNote(sel),
        ));
    } else {
        entries.push(EntryKind::Command(
            "Remove Annotation".to_string(),
            EntryId::RemoveAnnotation(sel),
        ));
        entries.push(EntryKind::Separator);
        entries.push(EntryKind::Command(
            "Edit Note".to_string(),
            EntryId::EditAnnotationNote(sel),
        ));
        entries.push(EntryKind::Command(
            "Remove Note".to_string(),
            EntryId::RemoveAnnotationNote(sel),
        ));
    }

    let create_menu = |ctx: &mut Context| -> Menu {
        Menu::new(
            rect,
            ViewId::AnnotationMenu,
            MenuKind::Contextual,
            entries,
            ctx,
        )
    };

    toggle_menu_vec(
        ViewId::AnnotationMenu,
        create_menu,
        children,
        enable,
        rq,
        context,
    );
}

/// Toggle selection menu for text selection actions
pub(crate) fn toggle_selection_menu(
    children: &mut Vec<Box<dyn View>>,
    current_page: usize,
    file_kind: &str,
    file_path: Option<String>,
    has_page_names: bool,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    use crate::view::menu::{Menu, MenuKind};

    let mut entries = vec![
        EntryKind::Command("Highlight".to_string(), EntryId::HighlightSelection),
        EntryKind::Command("Add Note".to_string(), EntryId::AnnotateSelection),
    ];

    if file_kind == "epub" {
        if let Some(path) = file_path {
            entries.push(EntryKind::Command(
                "Edit".to_string(),
                EntryId::Launch(AppCmd::EpubEditor {
                    path,
                    chapter: Some(current_page),
                }),
            ));
        }
    }

    entries.push(EntryKind::Separator);
    entries.push(EntryKind::Command(
        "Define".to_string(),
        EntryId::DefineSelection,
    ));
    entries.push(EntryKind::Command(
        "Search".to_string(),
        EntryId::SearchForSelection,
    ));

    if has_page_names {
        entries.push(EntryKind::Command(
            "Go To".to_string(),
            EntryId::GoToSelectedPageName,
        ));
    }

    entries.push(EntryKind::Separator);
    entries.push(EntryKind::Command(
        "Adjust Selection".to_string(),
        EntryId::AdjustSelection,
    ));

    let create_menu = |ctx: &mut Context| -> Menu {
        Menu::new(
            rect,
            ViewId::SelectionMenu,
            MenuKind::Contextual,
            entries,
            ctx,
        )
    };

    toggle_menu_vec(
        ViewId::SelectionMenu,
        create_menu,
        children,
        enable,
        rq,
        context,
    );
}

/// Toggle title menu for document-level settings and navigation
pub(crate) fn toggle_title_menu(
    children: &mut Vec<Box<dyn View>>,
    rect: Rectangle,
    reflowable: bool,
    file_kind: &str,
    file_path: Option<String>,
    has_annotations: bool,
    has_bookmarks: bool,
    zoom_mode: ZoomMode,
    scroll_mode: ScrollMode,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    use crate::view::menu::{Menu, MenuKind};

    let sf = if let ZoomMode::Custom(sf) = zoom_mode {
        sf
    } else {
        1.0
    };

    let mut entries = if reflowable {
        vec![EntryKind::SubMenu(
            "Zoom Mode".to_string(),
            vec![
                EntryKind::RadioButton(
                    "Fit to Page".to_string(),
                    EntryId::SetZoomMode(ZoomMode::FitToPage),
                    zoom_mode == ZoomMode::FitToPage,
                ),
                EntryKind::RadioButton(
                    format!("Custom ({:.1}%)", 100.0 * sf),
                    EntryId::SetZoomMode(ZoomMode::Custom(sf)),
                    zoom_mode == ZoomMode::Custom(sf),
                ),
            ],
        )]
    } else {
        vec![EntryKind::SubMenu(
            "Zoom Mode".to_string(),
            vec![
                EntryKind::RadioButton(
                    "Fit to Page".to_string(),
                    EntryId::SetZoomMode(ZoomMode::FitToPage),
                    zoom_mode == ZoomMode::FitToPage,
                ),
                EntryKind::RadioButton(
                    "Fit to Width".to_string(),
                    EntryId::SetZoomMode(ZoomMode::FitToWidth),
                    zoom_mode == ZoomMode::FitToWidth,
                ),
                EntryKind::RadioButton(
                    format!("Custom ({:.1}%)", 100.0 * sf),
                    EntryId::SetZoomMode(ZoomMode::Custom(sf)),
                    zoom_mode == ZoomMode::Custom(sf),
                ),
            ],
        )]
    };

    entries.push(EntryKind::SubMenu(
        "Scroll Mode".to_string(),
        vec![
            EntryKind::RadioButton(
                "Screen".to_string(),
                EntryId::SetScrollMode(ScrollMode::Screen),
                scroll_mode == ScrollMode::Screen,
            ),
            EntryKind::RadioButton(
                "Page".to_string(),
                EntryId::SetScrollMode(ScrollMode::Page),
                scroll_mode == ScrollMode::Page,
            ),
        ],
    ));

    if has_annotations {
        entries.push(EntryKind::Command(
            "Annotations".to_string(),
            EntryId::Annotations,
        ));
    }

    if has_bookmarks {
        entries.push(EntryKind::Command(
            "Bookmarks".to_string(),
            EntryId::Bookmarks,
        ));
    }

    if !entries.is_empty() {
        entries.push(EntryKind::Separator);
    }

    if file_kind == "epub" {
        if let Some(path) = file_path.as_ref() {
            entries.push(EntryKind::Command(
                "Edit EPUB".to_string(),
                EntryId::Launch(AppCmd::EpubEditor {
                    path: path.clone(),
                    chapter: None,
                }),
            ));
            entries.push(EntryKind::Separator);
        }
    }

    if file_kind == "pdf" {
        if let Some(path) = file_path.as_ref() {
            entries.push(EntryKind::Command(
                "PDF Tools".to_string(),
                EntryId::Launch(AppCmd::OpenPdfManipulator(path.clone().into())),
            ));
            entries.push(EntryKind::Separator);
        }
    }

    entries.push(EntryKind::CheckBox(
        "Apply Dithering".to_string(),
        EntryId::ToggleDithered,
        context.fb.dithered(),
    ));

    let id = ViewId::TitleMenu;

    let mut title_menu = Menu::new(rect, id, MenuKind::DropDown, entries, context);
    title_menu
        .child_mut(1)
        .downcast_mut::<MenuEntry>()
        .map(|entry| entry.set_disabled(zoom_mode != ZoomMode::FitToWidth, rq));

    toggle_menu_vec(id, |_| title_menu, children, enable, rq, context);
}

// ===========================================================================
// Contrast Settings Helpers
// ===========================================================================

/// Update contrast exponent in reader info
///
/// Helper function to extract the repetitive contrast exponent update logic.
/// Updates both the Info struct and a Contrast struct with the new value.
///
/// # Arguments
/// - `info`: Reader info to update
/// - `contrast`: Contrast settings to update
/// - `exponent`: New contrast exponent value
pub(crate) fn update_contrast_exponent(
    info: &mut Info,
    contrast: &mut super::reader_core::Contrast,
    exponent: f32,
) {
    if let Some(ref mut r) = info.reader {
        r.contrast_exponent = Some(exponent);
    }
    contrast.exponent = exponent;
}

/// Update contrast gray level in reader info
///
/// Helper function to extract the repetitive contrast gray update logic.
/// Updates both the Info struct and a Contrast struct with the new value.
///
/// # Arguments
/// - `info`: Reader info to update
/// - `contrast`: Contrast settings to update
/// - `gray`: New contrast gray value
pub(crate) fn update_contrast_gray(
    info: &mut Info,
    contrast: &mut super::reader_core::Contrast,
    gray: f32,
) {
    if let Some(ref mut r) = info.reader {
        r.contrast_gray = Some(gray);
    }
    contrast.gray = gray;
}

// ===========================================================================
// View Port Settings Helpers
// ===========================================================================

/// Update scroll mode in viewport
///
/// Helper function to update the scroll mode setting.
/// Also resets page offset when changing modes.
///
/// # Arguments
/// - `scroll_mode_ref`: Mutable reference to scroll mode field
/// - `page_offset_ref`: Mutable reference to page offset field
/// - `scroll_mode`: New scroll mode
pub(crate) fn update_scroll_mode(
    scroll_mode_ref: &mut ScrollMode,
    page_offset_ref: &mut crate::geom::Point,
    scroll_mode: ScrollMode,
) {
    *scroll_mode_ref = scroll_mode;
    *page_offset_ref = crate::geom::Point { x: 0, y: 0 };
}

/// Update zoom mode in viewport
///
/// Helper function to update the zoom mode setting.
/// Optionally resets page offset based on the reset flag.
///
/// # Arguments
/// - `zoom_mode_ref`: Mutable reference to zoom mode field
/// - `page_offset_ref`: Mutable reference to page offset field
/// - `zoom_mode`: New zoom mode
/// - `reset_page_offset`: Whether to reset page offset to (0, 0)
pub(crate) fn update_zoom_mode(
    zoom_mode_ref: &mut ZoomMode,
    page_offset_ref: &mut crate::geom::Point,
    zoom_mode: ZoomMode,
    reset_page_offset: bool,
) {
    *zoom_mode_ref = zoom_mode;
    if reset_page_offset {
        *page_offset_ref = crate::geom::Point { x: 0, y: 0 };
    }
}
