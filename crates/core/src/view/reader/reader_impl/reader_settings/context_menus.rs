//! Context Menu Functions
//!
//! Functions to toggle context menus for annotations, selections, and titles.

use crate::context::Context;
use crate::geom::Rectangle;
use crate::metadata::{Annotation, ScrollMode, ZoomMode};
use crate::view::menu::{Menu, MenuKind};
use crate::view::menu_helpers::toggle_menu_vec;
use crate::view::{AppCmd, EntryId, EntryKind, RenderQueue, View, ViewId};

pub(crate) fn toggle_annotation_menu(
    children: &mut Vec<Box<dyn View>>,
    annot: &Annotation,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
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

#[expect(
    clippy::too_many_arguments,
    reason = "Selection menu requires many parameters (children, page info, file info, hub) for comprehensive menu construction"
)]
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
    let mut entries = vec![
        EntryKind::Command("Highlight".to_string(), EntryId::HighlightSelection),
        EntryKind::Command("Add Note".to_string(), EntryId::AnnotateSelection),
    ];

    if file_kind == "epub" || file_kind == "kepub" {
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

#[expect(
    clippy::too_many_arguments,
    reason = "Title menu requires many parameters (children, rect, reflowable, file info, hub) for comprehensive menu construction"
)]
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

    if file_kind == "epub" || file_kind == "kepub" {
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
            #[cfg(any(target_os = "android", target_os = "ios", target_os = "linux"))]
            {
                entries.push(EntryKind::Command(
                    "Fill Forms".to_string(),
                    EntryId::FillForms(path.clone().into()),
                ));
            }
            #[cfg(target_os = "linux")]
            {
                entries.push(EntryKind::Command(
                    "Digital Signatures".to_string(),
                    EntryId::SignDocument(path.clone().into()),
                ));
            }
            #[cfg(target_os = "linux")]
            {
                entries.push(EntryKind::SubMenu(
                    "PDF/A Validation".to_string(),
                    vec![
                        EntryKind::Command(
                            "PDF/A-1b".to_string(),
                            EntryId::ValidatePdfA(
                                path.clone().into(),
                                crate::document::validation::PdfALevel::A1b,
                            ),
                        ),
                        EntryKind::Command(
                            "PDF/A-2b".to_string(),
                            EntryId::ValidatePdfA(
                                path.clone().into(),
                                crate::document::validation::PdfALevel::A2b,
                            ),
                        ),
                        EntryKind::Command(
                            "PDF/A-3b".to_string(),
                            EntryId::ValidatePdfA(
                                path.clone().into(),
                                crate::document::validation::PdfALevel::A3b,
                            ),
                        ),
                    ],
                ));
            }
            #[cfg(target_os = "linux")]
            {
                entries.push(EntryKind::SubMenu(
                    "PDF/X Validation".to_string(),
                    vec![
                        EntryKind::Command(
                            "PDF/X-1a".to_string(),
                            EntryId::ValidatePdfX(
                                path.clone().into(),
                                crate::document::validation::PdfXLevel::X1a,
                            ),
                        ),
                        EntryKind::Command(
                            "PDF/X-3".to_string(),
                            EntryId::ValidatePdfX(
                                path.clone().into(),
                                crate::document::validation::PdfXLevel::X3,
                            ),
                        ),
                        EntryKind::Command(
                            "PDF/X-4".to_string(),
                            EntryId::ValidatePdfX(
                                path.clone().into(),
                                crate::document::validation::PdfXLevel::X4,
                            ),
                        ),
                    ],
                ));
            }
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
    if let Some(entry) = title_menu
        .child_mut(1)
        .downcast_mut::<crate::view::menu_entry::MenuEntry>()
    {
        entry.set_disabled(zoom_mode != ZoomMode::FitToWidth, rq);
    }

    toggle_menu_vec(id, |_| title_menu, children, enable, rq, context);
}
