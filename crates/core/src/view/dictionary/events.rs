use regex::Regex;

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::document::{Document, Location};
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::{halves, Rectangle};
use crate::gesture::GestureEvent;
use crate::input::{ButtonCode, ButtonStatus, DeviceEvent};
use crate::unit::scale_by_dpi;
use crate::view::common::{toggle_battery_menu, toggle_clock_menu, toggle_main_menu};
use crate::view::dictionary::Dictionary;
use crate::view::image::Image;
use crate::view::keyboard::Keyboard;
use crate::view::{
    Bus, EntryId, Event, Hub, RenderData, RenderQueue, View, ViewId, BIG_BAR_HEIGHT,
    SMALL_BAR_HEIGHT, THICKNESS_MEDIUM,
};

use super::bottom_bar::BottomBar;

impl View for Dictionary {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        _bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match *evt {
            Event::Define(ref query) => {
                self.define(Some(query), rq, context);
                true
            }
            Event::Submit(ViewId::DictionarySearchInput, ref text) => {
                if !text.is_empty() {
                    self.toggle_keyboard(false, None, hub, rq, context);
                    self.define(Some(text), rq, context);
                }
                true
            }
            Event::Page(dir) => {
                self.go_to_neighbor(dir, rq);
                true
            }
            Event::Gesture(GestureEvent::Swipe { dir, start, .. }) if self.rect.includes(start) => {
                match dir {
                    crate::geom::Dir::West => self.go_to_neighbor(crate::geom::CycleDir::Next, rq),
                    crate::geom::Dir::East => {
                        self.go_to_neighbor(crate::geom::CycleDir::Previous, rq)
                    }
                    _ => (),
                }
                true
            }
            Event::Device(DeviceEvent::Button {
                code,
                status: ButtonStatus::Released,
                ..
            }) => {
                let cd = match code {
                    ButtonCode::Backward => Some(crate::geom::CycleDir::Previous),
                    ButtonCode::Forward => Some(crate::geom::CycleDir::Next),
                    _ => None,
                };
                if let Some(cd) = cd {
                    let loc = self.location;
                    self.go_to_neighbor(cd, rq);
                    if self.location == loc {
                        hub.send(Event::Back).ok();
                    }
                }
                true
            }
            Event::Gesture(GestureEvent::Tap(center)) if self.rect.includes(center) => {
                self.follow_link(center, rq, context);
                true
            }
            Event::Gesture(GestureEvent::HoldFingerLong(pt, _)) => {
                if let Some(text) = self.underlying_word(pt) {
                    let query = text
                        .trim_matches(|c: char| !c.is_alphanumeric())
                        .to_string();
                    self.define(Some(&query), rq, context);
                }
                true
            }
            Event::Select(EntryId::SetSearchTarget(ref target)) => {
                if *target != self.target {
                    self.target = target.clone();
                    let name = self.target.as_deref().unwrap_or("All");
                    if let Some(bottom_bar) = self.children[6].downcast_mut::<BottomBar>() {
                        bottom_bar.update_name(name, rq);
                    }
                    if !self.query.is_empty() {
                        self.define(None, rq, context);
                    }
                }
                true
            }
            Event::Select(EntryId::ToggleFuzzy) => {
                self.fuzzy = !self.fuzzy;
                if !self.query.is_empty() {
                    self.define(None, rq, context);
                }
                true
            }
            Event::Select(EntryId::ReloadDictionaries) => {
                context.dictionaries.clear();
                context.load_dictionaries();
                if let Some(name) = self.target.as_ref() {
                    if !context.dictionaries.contains_key(name) {
                        self.target = None;
                        if let Some(bottom_bar) = self.child_mut(6).downcast_mut::<BottomBar>() {
                            bottom_bar.update_name("All", rq);
                        }
                    }
                }
                true
            }
            Event::EditLanguages => {
                if self.target.is_some() {
                    self.toggle_edit_languages(None, hub, rq, context);
                }
                true
            }
            Event::Submit(ViewId::EditLanguagesInput, ref text) => {
                if let Some(name) = self.target.as_ref() {
                    let re = match Regex::new(r"\s*,\s*") {
                        Ok(r) => r,
                        Err(_) => return true,
                    };
                    context
                        .settings
                        .dictionary
                        .languages
                        .insert(name.clone(), re.split(text).map(String::from).collect());
                    if self.target.is_none() && !self.query.is_empty() {
                        self.define(None, rq, context);
                    }
                }
                true
            }
            Event::Close(ViewId::EditLanguages) => {
                self.toggle_keyboard(false, None, hub, rq, context);
                false
            }
            Event::Close(ViewId::SearchBar) => {
                hub.send(Event::Back).ok();
                true
            }
            Event::Focus(v) => {
                self.focus = v;
                if v.is_some() {
                    self.toggle_keyboard(true, v, hub, rq, context);
                }
                true
            }
            Event::ToggleNear(ViewId::TitleMenu, rect) => {
                self.toggle_title_menu(rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::SearchMenu, rect) => {
                self.toggle_search_menu(rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::SearchTargetMenu, rect) => {
                self.toggle_search_target_menu(rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::MainMenu, rect) => {
                toggle_main_menu(self, rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::BatteryMenu, rect) => {
                toggle_battery_menu(self, rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::ClockMenu, rect) => {
                toggle_clock_menu(self, rect, None, rq, context);
                true
            }
            Event::Reseed => {
                self.reseed(rq, context);
                true
            }
            Event::Gesture(GestureEvent::Cross(_)) => {
                hub.send(Event::Back).ok();
                true
            }
            _ => false,
        }
    }

    fn render(&self, _fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {}

    fn resize(&mut self, rect: Rectangle, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        let dpi = CURRENT_DEVICE.dpi;
        let (small_height, big_height) = (
            scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32,
            scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32,
        );
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
        let (small_thickness, big_thickness) = halves(thickness);

        self.children[0].resize(
            crate::rect![
                rect.min.x,
                rect.min.y,
                rect.max.x,
                rect.min.y + small_height - small_thickness
            ],
            hub,
            rq,
            context,
        );

        self.children[1].resize(
            crate::rect![
                rect.min.x,
                rect.min.y + small_height - small_thickness,
                rect.max.x,
                rect.min.y + small_height + big_thickness
            ],
            hub,
            rq,
            context,
        );

        self.children[2].resize(
            crate::rect![
                rect.min.x,
                rect.min.y + small_height + big_thickness,
                rect.max.x,
                rect.min.y + 2 * small_height - small_thickness
            ],
            hub,
            rq,
            context,
        );

        self.children[3].resize(
            crate::rect![
                rect.min.x,
                rect.min.y + 2 * small_height - small_thickness,
                rect.max.x,
                rect.min.y + 2 * small_height + big_thickness
            ],
            hub,
            rq,
            context,
        );

        let image_rect = crate::rect![
            rect.min.x,
            rect.min.y + 2 * small_height + big_thickness,
            rect.max.x,
            rect.max.y - small_height - small_thickness
        ];
        self.doc.layout(
            image_rect.width(),
            image_rect.height(),
            context.settings.dictionary.font_size,
            dpi,
        );
        self.doc
            .set_margin_width(context.settings.dictionary.margin_width);
        if let Some(image) = self.children[4].downcast_mut::<Image>() {
            if let Some((pixmap, loc)) = self.doc.pixmap(
                Location::Exact(self.location),
                1.0,
                CURRENT_DEVICE.color_samples(),
            ) {
                image.update(pixmap, &mut RenderQueue::new());
                self.location = loc;
            }
        }
        self.children[4].resize(image_rect, hub, rq, context);

        self.children[5].resize(
            crate::rect![
                rect.min.x,
                rect.max.y - small_height - small_thickness,
                rect.max.x,
                rect.max.y - small_height + big_thickness
            ],
            hub,
            rq,
            context,
        );

        self.children[6].resize(
            crate::rect![
                rect.min.x,
                rect.max.y - small_height + big_thickness,
                rect.max.x,
                rect.max.y
            ],
            hub,
            rq,
            context,
        );
        if let Some(bottom_bar) = self.children[6].downcast_mut::<BottomBar>() {
            bottom_bar.update_icons(
                self.doc
                    .resolve_location(Location::Previous(self.location))
                    .is_some(),
                self.doc
                    .resolve_location(Location::Next(self.location))
                    .is_some(),
                &mut RenderQueue::new(),
            );
        }
        let mut index = 7;
        if self.len() >= 9 {
            if self.children[8].is::<Keyboard>() {
                let kb_rect = crate::rect![
                    rect.min.x,
                    rect.max.y - (small_height + 3 * big_height) as i32 + big_thickness,
                    rect.max.x,
                    rect.max.y - small_height - small_thickness
                ];
                self.children[8].resize(kb_rect, hub, rq, context);
                let kb_rect = *self.children[8].rect();
                self.children[7].resize(
                    crate::rect![
                        rect.min.x,
                        kb_rect.min.y - thickness,
                        rect.max.x,
                        kb_rect.min.y
                    ],
                    hub,
                    rq,
                    context,
                );
                index = 9;
            }
        }

        for i in index..self.children.len() {
            self.children[i].resize(rect, hub, rq, context);
        }

        self.rect = rect;
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Full));
    }

    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn rect_mut(&mut self) -> &mut Rectangle {
        &mut self.rect
    }

    fn children(&self) -> &Vec<Box<dyn View>> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn View>> {
        &mut self.children
    }

    fn id(&self) -> crate::view::Id {
        self.id
    }
}
