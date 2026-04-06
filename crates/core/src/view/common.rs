use super::menu::{Menu, MenuKind};
use super::notification::Notification;
use super::{AppCmd, EntryId, EntryKind, RenderData, RenderQueue, View, ViewId};
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::geom::{Point, Rectangle};
use crate::settings::{ButtonScheme, RotationLock};
use chrono::Local;
use std::sync::mpsc;

/// Convenience macro for executing code when a child view with a specific ID exists.
///
/// This macro eliminates boilerplate by combining locate_by_id lookup with
/// conditional execution, reducing repeated patterns across the codebase.
///
/// # Examples
///
/// ```ignore
/// with_child!(self, ViewId::SortMenu, |index| {
///     self.children_mut().remove(index);
/// });
/// ```
#[macro_export]
macro_rules! with_child {
    ($view:expr, $id:expr, $body:expr) => {
        if let Some(index) = $crate::view::common::locate_by_id($view, $id) {
            $body(index)
        }
    };
}

pub fn close_view(id: ViewId, view: &mut dyn View, rq: &mut RenderQueue) {
    if let Some(index) = locate_by_id(view, id) {
        let rect = overlapping_rectangle(view.child(index));
        rq.add(RenderData::expose(rect, UpdateMode::Gui));
        view.children_mut().remove(index);
    }
}

pub fn toggle_view<F>(
    id: ViewId,
    make_view: F,
    view: &mut dyn View,
    enable: Option<bool>,
    rq: &mut RenderQueue,
) where
    F: FnOnce() -> Box<dyn View>,
{
    if let Some(index) = locate_by_id(view, id) {
        if let Some(true) = enable {
            return;
        }
        rq.add(RenderData::expose(
            overlapping_rectangle(view.child(index)),
            UpdateMode::Gui,
        ));
        view.children_mut().remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
        let next_view = make_view();
        rq.add(RenderData::new(
            next_view.id(),
            *next_view.rect(),
            UpdateMode::Gui,
        ));
        view.children_mut().push(next_view);
    }
}

pub fn shift(view: &mut dyn View, delta: Point) {
    *view.rect_mut() += delta;
    for child in view.children_mut().iter_mut() {
        shift(child.as_mut(), delta);
    }
}

pub fn locate<T: View>(view: &dyn View) -> Option<usize> {
    for (index, child) in view.children().iter().enumerate() {
        if child.as_ref().is::<T>() {
            return Some(index);
        }
    }
    None
}

pub fn rlocate<T: View>(view: &dyn View) -> Option<usize> {
    for (index, child) in view.children().iter().enumerate().rev() {
        if child.as_ref().is::<T>() {
            return Some(index);
        }
    }
    None
}

pub fn locate_by_id(view: &dyn View, id: ViewId) -> Option<usize> {
    view.children()
        .iter()
        .position(|c| c.view_id().map_or(false, |i| i == id))
}

/// Add a menu to a view's children and queue it for rendering.
///
/// This convenience function eliminates boilerplate by combining menu creation,
/// render queue addition, and child insertion into a single operation.
///
/// # Arguments
///
/// * `menu` - The menu to add (already created via Menu::new())
/// * `view` - The parent view to add the menu to
/// * `rq` - The render queue to add this render operation to
///
/// # Examples
///
/// ```ignore
/// let menu = Menu::new(rect, id, kind, entries, context);
/// add_menu(menu, self, rq);
/// ```
pub fn add_menu(menu: Menu, view: &mut dyn View, rq: &mut RenderQueue) {
    rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
    view.children_mut().push(Box::new(menu) as Box<dyn View>);
}

pub fn overlapping_rectangle(view: &dyn View) -> Rectangle {
    let mut rect = *view.rect();
    for child in view.children() {
        rect.absorb(&overlapping_rectangle(child.as_ref()));
    }
    rect
}

// Transfer the notifications from the view1 to the view2.
pub fn transfer_notifications(
    view1: &mut dyn View,
    view2: &mut dyn View,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    for index in (0..view1.len()).rev() {
        if view1.child(index).is::<Notification>() {
            let mut child = view1.children_mut().remove(index);
            if view2.rect() != view1.rect() {
                let (tx, _rx) = mpsc::channel();
                child.resize(*view2.rect(), &tx, rq, context);
            }
            view2.children_mut().push(child);
        }
    }
}

pub fn toggle_main_menu(
    view: &mut dyn View,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    toggle_view(
        ViewId::MainMenu,
        || {
            let rotation = CURRENT_DEVICE.to_canonical(context.display.rotation);
            let rotate = (0..4)
                .map(|n| {
                    EntryKind::RadioButton(
                        (n as i16 * 90).to_string(),
                        EntryId::Rotate(CURRENT_DEVICE.from_canonical(n)),
                        n == rotation,
                    )
                })
                .collect::<Vec<EntryKind>>();

            let apps = vec![
                EntryKind::Command(
                    "Dictionary".to_string(),
                    EntryId::Launch(AppCmd::Dictionary {
                        query: "".to_string(),
                        language: "".to_string(),
                    }),
                ),
                EntryKind::Command(
                    "Calculator".to_string(),
                    EntryId::Launch(AppCmd::Calculator),
                ),
                EntryKind::Command("Sketch".to_string(), EntryId::Launch(AppCmd::Sketch)),
                EntryKind::Command(
                    "EPUB Editor".to_string(),
                    EntryId::Launch(AppCmd::EpubEditor {
                        path: "".to_string(),
                        chapter: None,
                    }),
                ),
                EntryKind::Command(
                    "Cover Editor".to_string(),
                    EntryId::Launch(AppCmd::CoverEditor),
                ),
                EntryKind::Command(
                    "Statistics".to_string(),
                    EntryId::Launch(AppCmd::Statistics),
                ),
                EntryKind::Command(
                    "PDF Tools".to_string(),
                    EntryId::Launch(AppCmd::PdfManipulator),
                ),
                EntryKind::Separator,
                EntryKind::Command(
                    "Touch Events".to_string(),
                    EntryId::Launch(AppCmd::TouchEvents),
                ),
                EntryKind::Command(
                    "Rotation Values".to_string(),
                    EntryId::Launch(AppCmd::RotationValues),
                ),
            ];
            let mut entries = vec![
                EntryKind::Command("About".to_string(), EntryId::About),
                EntryKind::Command("System Info".to_string(), EntryId::SystemInfo),
                EntryKind::Separator,
                EntryKind::CheckBox(
                    "Invert Colors".to_string(),
                    EntryId::ToggleInverted,
                    context.fb.inverted(),
                ),
                EntryKind::CheckBox(
                    "Enable WiFi".to_string(),
                    EntryId::ToggleWifi,
                    context.settings.wifi,
                ),
                EntryKind::Separator,
                EntryKind::SubMenu("Rotate".to_string(), rotate),
                EntryKind::Command("Take Screenshot".to_string(), EntryId::TakeScreenshot),
                EntryKind::Separator,
                EntryKind::SubMenu("Applications".to_string(), apps),
                EntryKind::Separator,
                EntryKind::Command("Settings".to_string(), EntryId::OpenSettingsEditor),
                EntryKind::Separator,
            ];

            entries.push(EntryKind::Command("Reboot".to_string(), EntryId::Reboot));
            entries.push(EntryKind::Command("Quit".to_string(), EntryId::Quit));

            if CURRENT_DEVICE.has_page_turn_buttons() {
                let button_scheme = context.settings.button_scheme;
                let button_schemes = vec![
                    EntryKind::RadioButton(
                        ButtonScheme::Natural.to_string(),
                        EntryId::SetButtonScheme(ButtonScheme::Natural),
                        button_scheme == ButtonScheme::Natural,
                    ),
                    EntryKind::RadioButton(
                        ButtonScheme::Inverted.to_string(),
                        EntryId::SetButtonScheme(ButtonScheme::Inverted),
                        button_scheme == ButtonScheme::Inverted,
                    ),
                ];
                entries.insert(
                    5,
                    EntryKind::SubMenu("Button Scheme".to_string(), button_schemes),
                );
            }

            if CURRENT_DEVICE.has_gyroscope() {
                let rotation_lock = context.settings.rotation_lock;
                let gyro = vec![
                    EntryKind::RadioButton(
                        "Auto".to_string(),
                        EntryId::SetRotationLock(None),
                        rotation_lock.is_none(),
                    ),
                    EntryKind::Separator,
                    EntryKind::RadioButton(
                        "Portrait".to_string(),
                        EntryId::SetRotationLock(Some(RotationLock::Portrait)),
                        rotation_lock == Some(RotationLock::Portrait),
                    ),
                    EntryKind::RadioButton(
                        "Landscape".to_string(),
                        EntryId::SetRotationLock(Some(RotationLock::Landscape)),
                        rotation_lock == Some(RotationLock::Landscape),
                    ),
                    EntryKind::RadioButton(
                        "Ignore".to_string(),
                        EntryId::SetRotationLock(Some(RotationLock::Current)),
                        rotation_lock == Some(RotationLock::Current),
                    ),
                ];
                entries.insert(5, EntryKind::SubMenu("Gyroscope".to_string(), gyro));
            }

            Box::new(Menu::new(
                rect,
                ViewId::MainMenu,
                MenuKind::DropDown,
                entries,
                context,
            )) as Box<dyn View>
        },
        view,
        enable,
        rq,
    );
}

pub fn toggle_battery_menu(
    view: &mut dyn View,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    toggle_view(
        ViewId::BatteryMenu,
        || {
            let mut entries = Vec::new();

            match context
                .battery
                .status()
                .ok()
                .zip(context.battery.capacity().ok())
            {
                Some((status, capacity)) => {
                    for (_i, (s, c)) in status.iter().zip(capacity.iter()).enumerate() {
                        entries.push(EntryKind::Message(format!("{:?} {}%", s, c), None));
                    }
                }
                _ => {
                    entries.push(EntryKind::Message(
                        "Information Unavailable".to_string(),
                        None,
                    ));
                }
            }

            Box::new(Menu::new(
                rect,
                ViewId::BatteryMenu,
                MenuKind::DropDown,
                entries,
                context,
            )) as Box<dyn View>
        },
        view,
        enable,
        rq,
    );
}

pub fn toggle_clock_menu(
    view: &mut dyn View,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    toggle_view(
        ViewId::ClockMenu,
        || {
            let text = Local::now()
                .format(&context.settings.date_format)
                .to_string();
            let entries = vec![EntryKind::Message(text, None)];
            Box::new(Menu::new(
                rect,
                ViewId::ClockMenu,
                MenuKind::DropDown,
                entries,
                context,
            )) as Box<dyn View>
        },
        view,
        enable,
        rq,
    );
}

pub fn toggle_input_history_menu(
    view: &mut dyn View,
    id: ViewId,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    if let Some(index) = locate_by_id(view, ViewId::InputHistoryMenu) {
        if let Some(true) = enable {
            return;
        }
        rq.add(RenderData::expose(
            overlapping_rectangle(view.child(index)),
            UpdateMode::Gui,
        ));
        view.children_mut().remove(index);
    } else {
        if let Some(false) = enable {
            return;
        }
        let entries = context.input_history.get(&id).map(|h| {
            h.iter()
                .map(|s| {
                    EntryKind::Command(s.to_string(), EntryId::SetInputText(id, s.to_string()))
                })
                .collect::<Vec<EntryKind>>()
        });
        if let Some(entries) = entries {
            let menu_kind = match id {
                ViewId::HomeSearchInput
                | ViewId::ReaderSearchInput
                | ViewId::DictionarySearchInput
                | ViewId::CalculatorInput => MenuKind::DropDown,
                _ => MenuKind::Contextual,
            };
            let input_history_menu =
                Menu::new(rect, ViewId::InputHistoryMenu, menu_kind, entries, context);
            rq.add(RenderData::new(
                input_history_menu.id(),
                *input_history_menu.rect(),
                UpdateMode::Gui,
            ));
            view.children_mut()
                .push(Box::new(input_history_menu) as Box<dyn View>);
        }
    }
}

pub fn toggle_keyboard_layout_menu(
    view: &mut dyn View,
    rect: Rectangle,
    enable: Option<bool>,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    toggle_view(
        ViewId::KeyboardLayoutMenu,
        || {
            let entries = context
                .keyboard_layouts
                .keys()
                .map(|s| {
                    EntryKind::Command(s.to_string(), EntryId::SetKeyboardLayout(s.to_string()))
                })
                .collect::<Vec<EntryKind>>();
            Box::new(Menu::new(
                rect,
                ViewId::KeyboardLayoutMenu,
                MenuKind::Contextual,
                entries,
                context,
            )) as Box<dyn View>
        },
        view,
        enable,
        rq,
    );
}
