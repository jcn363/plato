//! Reader Gestures and Input Module
//!
//! Handles touch gestures, button input, stylus interaction, and event processing.
//!
//! This module contains extracted event handlers from `Reader::handle_event()` for improved
//! maintainability and testability.

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::frontlight::LightLevels;
use crate::geom::{Axis, CycleDir, DiagDir, Dir, LinearDir, Point};
use crate::gesture::GestureEvent;
use crate::metadata::{ScrollMode, ZoomMode};
use crate::settings::guess_frontlight;
use crate::settings::BottomRightGestureAction;

use super::reader::Reader;
use crate::view::{Event, Hub, RenderQueue};

impl Reader {
    pub(crate) fn handle_gesture_event(
        &mut self,
        evt: &GestureEvent,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match evt {
            GestureEvent::Rotate { quarter_turns, .. } if *quarter_turns != 0 => {
                let (_, dir) = CURRENT_DEVICE.mirroring_scheme();
                let n = (4 + (context.display.rotation - dir * quarter_turns)) % 4;
                hub.send(Event::Select(crate::view::EntryId::Rotate(n)))
                    .ok();
                true
            }
            GestureEvent::Swipe { dir, start, end } if self.rect.includes(*start) => {
                self.handle_swipe_gesture(*dir, *start, *end, hub, rq, context)
            }
            GestureEvent::SlantedSwipe { start, end, .. }
                if self.rect.includes(*start)
                    && matches!(self.view_port.zoom_mode, ZoomMode::Custom(_)) =>
            {
                self.directional_scroll(*start - *end, hub, rq, context);
                true
            }
            GestureEvent::Spread {
                axis: Axis::Horizontal,
                center,
                ..
            } if self.rect.includes(*center) && !self.reflowable => {
                self.set_zoom_mode(ZoomMode::FitToWidth, true, hub, rq, context);
                true
            }
            GestureEvent::Pinch {
                axis: Axis::Horizontal,
                center,
                ..
            } if self.rect.includes(*center) => {
                self.set_zoom_mode(ZoomMode::FitToPage, true, hub, rq, context);
                true
            }
            GestureEvent::Spread {
                axis: Axis::Vertical,
                center,
                ..
            } if self.rect.includes(*center) && !self.reflowable => {
                self.set_scroll_mode(ScrollMode::Screen, hub, rq, context);
                true
            }
            GestureEvent::Pinch {
                axis: Axis::Vertical,
                center,
                ..
            } if self.rect.includes(*center) && !self.reflowable => {
                self.set_scroll_mode(ScrollMode::Page, hub, rq, context);
                true
            }
            GestureEvent::Spread {
                axis: Axis::Diagonal,
                center,
                factor,
            }
            | GestureEvent::Pinch {
                axis: Axis::Diagonal,
                center,
                factor,
            } if factor.is_finite() && self.rect.includes(*center) => {
                self.scale_page(*center, *factor, hub, rq, context);
                true
            }
            GestureEvent::Arrow { dir, .. } => self.handle_arrow_gesture(*dir, hub, rq, context),
            GestureEvent::Corner { dir, .. } => self.handle_corner_gesture(*dir, hub, rq, context),
            GestureEvent::MultiCorner { dir, .. } => {
                self.handle_multi_corner_gesture(*dir, hub, rq, context)
            }
            GestureEvent::Cross(_) => {
                self.quit(context);
                hub.send(Event::Back).ok();
                true
            }
            GestureEvent::Diamond(_) => {
                self.toggle_bars(None, hub, rq, context);
                true
            }
            _ => false,
        }
    }

    fn handle_swipe_gesture(
        &mut self,
        dir: Dir,
        start: Point,
        end: Point,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match self.view_port.zoom_mode {
            ZoomMode::FitToPage | ZoomMode::FitToWidth | ZoomMode::Fit(_) => match dir {
                Dir::West => self.go_to_neighbor(CycleDir::Next, hub, rq, context),
                Dir::East => self.go_to_neighbor(CycleDir::Previous, hub, rq, context),
                Dir::South | Dir::North => {
                    self.vertical_scroll(start.y - end.y, hub, rq, context);
                }
            },
            ZoomMode::Custom(_) => match dir {
                Dir::West | Dir::East => {
                    self.directional_scroll(pt!(start.x - end.x, 0), hub, rq, context);
                }
                Dir::South | Dir::North => {
                    self.directional_scroll(pt!(0, start.y - end.y), hub, rq, context);
                }
            },
        }
        true
    }

    fn handle_arrow_gesture(
        &mut self,
        dir: Dir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match dir {
            Dir::West => {
                if self.search.is_none() {
                    self.go_to_chapter(CycleDir::Previous, hub, rq, context);
                } else {
                    self.go_to_results_page(0, hub, rq, context);
                }
            }
            Dir::East => {
                if self.search.is_none() {
                    self.go_to_chapter(CycleDir::Next, hub, rq, context);
                } else if let Some(ref search) = self.search {
                    let last_page = search.highlights.len() - 1;
                    self.go_to_results_page(last_page, hub, rq, context);
                }
            }
            Dir::North => {
                self.search_direction = LinearDir::Backward;
                self.toggle_search_bar(true, hub, rq, context);
            }
            Dir::South => {
                self.search_direction = LinearDir::Forward;
                self.toggle_search_bar(true, hub, rq, context);
            }
        }
        true
    }

    fn handle_corner_gesture(
        &mut self,
        dir: DiagDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match dir {
            DiagDir::NorthWest => self.go_to_bookmark(CycleDir::Previous, hub, rq, context),
            DiagDir::NorthEast => self.go_to_bookmark(CycleDir::Next, hub, rq, context),
            DiagDir::SouthEast => match context.settings.reader.bottom_right_gesture {
                BottomRightGestureAction::ToggleDithered => {
                    hub.send(Event::Select(crate::view::EntryId::ToggleDithered))
                        .ok();
                }
                BottomRightGestureAction::ToggleInverted => {
                    hub.send(Event::Select(crate::view::EntryId::ToggleInverted))
                        .ok();
                }
            },
            DiagDir::SouthWest => {
                if context.settings.frontlight_presets.len() > 1 {
                    if context.settings.frontlight {
                        let lightsensor_level = if CURRENT_DEVICE.has_lightsensor() {
                            context.lightsensor.level().ok()
                        } else {
                            None
                        };
                        if let Some(frontlight_levels) = guess_frontlight(
                            lightsensor_level,
                            &context.settings.frontlight_presets,
                        ) {
                            let LightLevels { intensity, warmth } = frontlight_levels;
                            context.frontlight.set_intensity(intensity);
                            context.frontlight.set_warmth(warmth);
                        }
                    }
                } else {
                    hub.send(Event::ToggleFrontlight).ok();
                }
            }
        };
        true
    }

    fn handle_multi_corner_gesture(
        &mut self,
        dir: DiagDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match dir {
            DiagDir::NorthWest => {
                self.go_to_annotation(CycleDir::Previous, hub, rq, context);
            }
            DiagDir::NorthEast => self.go_to_annotation(CycleDir::Next, hub, rq, context),
            _ => (),
        }
        true
    }
}
