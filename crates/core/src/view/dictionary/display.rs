use crate::color::BLACK;
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::geom::{halves, Rectangle};
use crate::unit::scale_by_dpi;
use crate::view::common::{locate, locate_by_id};
use crate::view::dictionary::Dictionary;
use crate::view::filler::Filler;
use crate::view::keyboard::Keyboard;
use crate::view::menu::{Menu, MenuKind};
use crate::view::named_input::NamedInput;
use crate::view::{
    EntryId, EntryKind, Event, Hub, RenderData, RenderQueue, View, ViewId, BIG_BAR_HEIGHT,
    SMALL_BAR_HEIGHT, THICKNESS_MEDIUM,
};

use super::bottom_bar::BottomBar;

impl Dictionary {
    pub fn toggle_title_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Some(index) = locate_by_id(self, ViewId::TitleMenu) {
            if let Some(true) = enable {
                return;
            }

            rq.add(RenderData::expose(
                *self.child(index).rect(),
                UpdateMode::Gui,
            ));
            self.children.remove(index);
        } else {
            if let Some(false) = enable {
                return;
            }
            let entries = vec![EntryKind::Command(
                "Reload Dictionaries".to_string(),
                EntryId::ReloadDictionaries,
            )];
            let title_menu = Menu::new(
                rect,
                ViewId::TitleMenu,
                MenuKind::DropDown,
                entries,
                context,
            );
            rq.add(RenderData::new(
                title_menu.id(),
                *title_menu.rect(),
                UpdateMode::Gui,
            ));
            self.children.push(Box::new(title_menu) as Box<dyn View>);
        }
    }

    pub fn toggle_search_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Some(index) = locate_by_id(self, ViewId::SearchMenu) {
            if let Some(true) = enable {
                return;
            }

            rq.add(RenderData::expose(
                *self.child(index).rect(),
                UpdateMode::Gui,
            ));
            self.children.remove(index);
        } else {
            if let Some(false) = enable {
                return;
            }
            let entries = vec![EntryKind::CheckBox(
                "Fuzzy".to_string(),
                EntryId::ToggleFuzzy,
                self.fuzzy,
            )];
            let search_menu = Menu::new(
                rect,
                ViewId::SearchMenu,
                MenuKind::Contextual,
                entries,
                context,
            );
            rq.add(RenderData::new(
                search_menu.id(),
                *search_menu.rect(),
                UpdateMode::Gui,
            ));
            self.children.push(Box::new(search_menu) as Box<dyn View>);
        }
    }

    pub fn toggle_search_target_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Some(index) = locate_by_id(self, ViewId::SearchTargetMenu) {
            if let Some(true) = enable {
                return;
            }

            rq.add(RenderData::expose(
                *self.child(index).rect(),
                UpdateMode::Gui,
            ));
            self.children.remove(index);
        } else {
            if let Some(false) = enable {
                return;
            }
            let mut entries = context
                .dictionaries
                .keys()
                .map(|k| {
                    EntryKind::RadioButton(
                        k.to_string(),
                        EntryId::SetSearchTarget(Some(k.to_string())),
                        self.target == Some(k.to_string()),
                    )
                })
                .collect::<Vec<EntryKind>>();
            if !entries.is_empty() {
                entries.push(EntryKind::Separator);
            }
            entries.push(EntryKind::RadioButton(
                "All".to_string(),
                EntryId::SetSearchTarget(None),
                self.target.is_none(),
            ));
            let search_target_menu = Menu::new(
                rect,
                ViewId::SearchTargetMenu,
                MenuKind::DropDown,
                entries,
                context,
            );
            rq.add(RenderData::new(
                search_target_menu.id(),
                *search_target_menu.rect(),
                UpdateMode::Gui,
            ));
            self.children
                .push(Box::new(search_target_menu) as Box<dyn View>);
        }
    }

    pub fn toggle_keyboard(
        &mut self,
        enable: bool,
        id: Option<ViewId>,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Some(index) = locate::<Keyboard>(self) {
            if enable {
                return;
            }

            let mut rect = *self.child(index).rect();
            rect.absorb(self.child(index - 1).rect());
            self.children.drain(index - 1..=index);

            context.kb_rect = Rectangle::default();
            rq.add(RenderData::expose(rect, UpdateMode::Gui));
            hub.send(Event::Focus(None)).ok();
        } else {
            if !enable {
                return;
            }

            let dpi = CURRENT_DEVICE.dpi;
            let (small_height, big_height) = (
                scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32,
                scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32,
            );
            let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
            let (small_thickness, big_thickness) = halves(thickness);

            let mut kb_rect = crate::rect![
                self.rect.min.x,
                self.rect.max.y - (small_height + 3 * big_height) as i32 + big_thickness,
                self.rect.max.x,
                self.rect.max.y - small_height - small_thickness
            ];

            let number = id == Some(ViewId::GoToPageInput);
            let Some(index) = locate::<BottomBar>(self) else {
                return;
            };
            let index = index + 1;

            let keyboard = Keyboard::new(&mut kb_rect, number, context);
            self.children
                .insert(index, Box::new(keyboard) as Box<dyn View>);

            let separator = Filler::new(
                crate::rect![
                    self.rect.min.x,
                    kb_rect.min.y - thickness,
                    self.rect.max.x,
                    kb_rect.min.y
                ],
                BLACK,
            );
            self.children
                .insert(index, Box::new(separator) as Box<dyn View>);

            for i in index..=index + 1 {
                rq.add(RenderData::new(
                    self.child(i).id(),
                    *self.child(i).rect(),
                    UpdateMode::Gui,
                ));
            }
        }
    }

    pub fn toggle_edit_languages(
        &mut self,
        enable: Option<bool>,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if let Some(index) = locate_by_id(self, ViewId::EditLanguages) {
            if let Some(true) = enable {
                return;
            }

            rq.add(RenderData::expose(
                *self.child(index).rect(),
                UpdateMode::Gui,
            ));
            self.children.remove(index);

            if self
                .focus
                .map(|focus_id| focus_id == ViewId::EditLanguagesInput)
                .unwrap_or(false)
            {
                self.toggle_keyboard(false, None, hub, rq, context);
            }
        } else {
            if let Some(false) = enable {
                return;
            }

            let mut edit_languages = NamedInput::new(
                "Languages".to_string(),
                ViewId::EditLanguages,
                ViewId::EditLanguagesInput,
                16,
                context,
            );
            if let Some(langs) = self
                .target
                .as_ref()
                .and_then(|name| context.settings.dictionary.languages.get(name))
                .filter(|langs| !langs.is_empty())
            {
                edit_languages.set_text(&langs.join(", "), &mut RenderQueue::new(), context);
            }

            rq.add(RenderData::new(
                edit_languages.id(),
                *edit_languages.rect(),
                UpdateMode::Gui,
            ));
            hub.send(Event::Focus(Some(ViewId::EditLanguagesInput)))
                .ok();

            self.children
                .push(Box::new(edit_languages) as Box<dyn View>);
        }
    }
}
