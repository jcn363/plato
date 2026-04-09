//! Reader Gestures and Input Module
//!
//! Handles touch gestures, button input, stylus interaction, and event processing.
//!
//! This module contains extracted event handlers from `Reader::handle_event()` for improved
//! maintainability and testability.
//!
//! ## GestureProcessor Trait
//!
//! The [`GestureProcessor`] trait provides an abstraction layer for gesture handling,
//! allowing different device types to customize gesture behavior without modifying
//! the core Reader implementation.

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::frontlight::LightLevels;
use crate::geom::{Axis, CycleDir, DiagDir, Dir, LinearDir, Point};
use crate::gesture::GestureEvent;
use crate::input::{ButtonCode, ButtonStatus, DeviceEvent, FingerStatus};
use crate::metadata::{ScrollMode, ZoomMode};
use crate::settings::guess_frontlight;
use crate::settings::BottomRightGestureAction;

use super::reader::Reader;
use super::reader_core::State;
use crate::view::{Event, Hub, RenderData, RenderQueue};

const RECT_DIST_JITTER: f32 = 15.0;

/// Trait for customizable gesture processing.
///
/// Implement this trait to provide custom gesture handling for different device types
/// or to add new gesture behaviors without modifying the core Reader implementation.
#[allow(dead_code)]
pub trait GestureProcessor {
    /// Process a rotate gesture (typically device rotation).
    fn process_rotate(&self, quarter_turns: i32, hub: &Hub, context: &Context) -> bool;

    /// Process a swipe gesture.
    fn process_swipe(
        &self,
        dir: Dir,
        start: Point,
        end: Point,
        reader: &mut Reader,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool;

    /// Process a tap gesture on a specific point.
    fn process_tap(
        &self,
        center: Point,
        reader: &mut Reader,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool;

    /// Process a corner tap gesture.
    fn process_corner(
        &self,
        dir: DiagDir,
        reader: &mut Reader,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool;

    /// Process a multi-corner gesture (two-finger corner).
    fn process_multi_corner(
        &self,
        dir: DiagDir,
        reader: &mut Reader,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool;

    /// Process an arrow gesture (directional input).
    fn process_arrow(
        &self,
        dir: Dir,
        reader: &mut Reader,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool;

    /// Process a pinch or spread gesture.
    fn process_pinch_spread(
        &self,
        axis: Axis,
        center: Point,
        factor: f32,
        reader: &mut Reader,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool;
}

/// Default gesture processor implementing standard Kobo device gesture behavior.
#[allow(dead_code)]
pub struct DefaultGestureProcessor;

impl DefaultGestureProcessor {
    #[allow(dead_code)]
    pub fn new() -> Self {
        DefaultGestureProcessor
    }
}

impl Default for DefaultGestureProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureProcessor for DefaultGestureProcessor {
    fn process_rotate(&self, quarter_turns: i32, hub: &Hub, context: &Context) -> bool {
        let (_, dir) = CURRENT_DEVICE.mirroring_scheme();
        let n = (4 + (context.display.rotation - dir * quarter_turns as i8)) % 4;
        hub.send(Event::Select(crate::view::EntryId::Rotate(n)))
            .ok();
        true
    }

    fn process_swipe(
        &self,
        dir: Dir,
        start: Point,
        end: Point,
        reader: &mut Reader,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match reader.view_port.zoom_mode {
            ZoomMode::FitToPage | ZoomMode::FitToWidth => match dir {
                Dir::West => reader.go_to_neighbor(CycleDir::Next, hub, rq, context),
                Dir::East => reader.go_to_neighbor(CycleDir::Previous, hub, rq, context),
                Dir::South | Dir::North => {
                    reader.vertical_scroll(start.y - end.y, hub, rq, context);
                }
            },
            ZoomMode::Custom(_) => match dir {
                Dir::West | Dir::East => {
                    reader.directional_scroll(pt!(start.x - end.x, 0), hub, rq, context);
                }
                Dir::South | Dir::North => {
                    reader.directional_scroll(pt!(0, start.y - end.y), hub, rq, context);
                }
            },
        }
        true
    }

    fn process_tap(
        &self,
        _center: Point,
        _reader: &mut Reader,
        _hub: &Hub,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        false
    }

    fn process_corner(
        &self,
        dir: DiagDir,
        reader: &mut Reader,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match dir {
            DiagDir::NorthWest => reader.go_to_bookmark(CycleDir::Previous, hub, rq, context),
            DiagDir::NorthEast => reader.go_to_bookmark(CycleDir::Next, hub, rq, context),
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

    fn process_multi_corner(
        &self,
        dir: DiagDir,
        reader: &mut Reader,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match dir {
            DiagDir::NorthWest => {
                reader.go_to_annotation(CycleDir::Previous, hub, rq, context);
            }
            DiagDir::NorthEast => reader.go_to_annotation(CycleDir::Next, hub, rq, context),
            _ => (),
        }
        true
    }

    fn process_arrow(
        &self,
        dir: Dir,
        reader: &mut Reader,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match dir {
            Dir::West => {
                if reader.search.is_none() {
                    reader.go_to_chapter(CycleDir::Previous, hub, rq, context);
                } else {
                    reader.go_to_results_page(0, hub, rq, context);
                }
            }
            Dir::East => {
                if reader.search.is_none() {
                    reader.go_to_chapter(CycleDir::Next, hub, rq, context);
                } else if let Some(ref search) = reader.search {
                    let last_page = search.highlights.len() - 1;
                    reader.go_to_results_page(last_page, hub, rq, context);
                }
            }
            Dir::North => {
                reader.search_direction = LinearDir::Backward;
                reader.toggle_search_bar(true, hub, rq, context);
            }
            Dir::South => {
                reader.search_direction = LinearDir::Forward;
                reader.toggle_search_bar(true, hub, rq, context);
            }
        }
        true
    }

    fn process_pinch_spread(
        &self,
        axis: Axis,
        center: Point,
        factor: f32,
        reader: &mut Reader,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match axis {
            Axis::Horizontal => {
                if !reader.reflowable {
                    reader.set_zoom_mode(ZoomMode::FitToWidth, true, hub, rq, context);
                } else {
                    reader.set_zoom_mode(ZoomMode::FitToPage, true, hub, rq, context);
                }
            }
            Axis::Vertical => {
                if !reader.reflowable {
                    reader.set_scroll_mode(ScrollMode::Screen, hub, rq, context);
                } else {
                    reader.set_scroll_mode(ScrollMode::Page, hub, rq, context);
                }
            }
            Axis::Diagonal => {
                if factor.is_finite() && reader.rect.includes(center) {
                    reader.scale_page(center, factor, hub, rq, context);
                }
            }
        }
        true
    }
}

#[allow(dead_code)]
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
            ZoomMode::FitToPage | ZoomMode::FitToWidth => match dir {
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

    pub(crate) fn handle_button_event(
        &mut self,
        evt: &DeviceEvent,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match evt {
            DeviceEvent::Button {
                code,
                status: ButtonStatus::Released,
                ..
            } => {
                if !self.held_buttons.remove(code) {
                    match code {
                        ButtonCode::Backward => {
                            if self.search.is_none() {
                                self.go_to_neighbor(CycleDir::Previous, hub, rq, context);
                            } else {
                                self.go_to_results_neighbor(CycleDir::Previous, hub, rq, context);
                            }
                        }
                        ButtonCode::Forward => {
                            if self.search.is_none() {
                                self.go_to_neighbor(CycleDir::Next, hub, rq, context);
                            } else {
                                self.go_to_results_neighbor(CycleDir::Next, hub, rq, context);
                            }
                        }
                        _ => (),
                    }
                }
                true
            }
            DeviceEvent::Finger {
                position,
                status: FingerStatus::Motion,
                id,
                ..
            } if self.state == State::Selection(*id as usize) => {
                self.handle_selection_motion(*position, hub, rq, context)
            }
            DeviceEvent::Finger {
                position,
                status: FingerStatus::Up,
                id,
                ..
            } if self.state == State::Selection(*id as usize) => {
                self.handle_selection_up(*position, hub, rq, context)
            }
            _ => false,
        }
    }

    fn handle_selection_motion(
        &mut self,
        position: Point,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &Context,
    ) -> bool {
        use crate::unit::scale_by_dpi;

        let mut nearest_word = None;
        let mut dmin = u32::MAX;
        let dmax = (scale_by_dpi(RECT_DIST_JITTER, CURRENT_DEVICE.dpi) as i32).pow(2) as u32;
        let mut rects = Vec::new();

        for chunk in &self.chunks {
            for word in &self.text[&chunk.location] {
                let rect = (word.rect * chunk.scale).to_rect() - chunk.frame.min + chunk.position;
                rects.push((rect, word.location));
                let d = position.rdist2(&rect);
                if d < dmax && d < dmin {
                    dmin = d;
                    nearest_word = Some(word.clone());
                }
            }
        }

        let Some(selection) = self.selection.as_mut() else {
            return true;
        };

        if let Some(word) = nearest_word {
            let old_start = selection.start;
            let old_end = selection.end;
            let (start, end) = word.location.min_max(selection.anchor);

            if start == old_start && end == old_end {
                return true;
            }

            let (start_low, start_high) = old_start.min_max(start);
            let (end_low, end_high) = old_end.min_max(end);

            if start_low != start_high {
                if let Some(mut i) = rects.iter().position(|(_, loc)| *loc == start_low) {
                    let mut rect = rects[i].0;
                    while rects[i].1 < start_high {
                        let next_rect = rects[i + 1].0;
                        if rect.max.y.min(next_rect.max.y) - rect.min.y.max(next_rect.min.y)
                            > rect.height().min(next_rect.height()) as i32 / 2
                        {
                            if rects[i + 1].1 == start_high {
                                if rect.min.x < next_rect.min.x {
                                    rect.max.x = next_rect.min.x;
                                } else {
                                    rect.min.x = next_rect.max.x;
                                }
                                rect.min.y = rect.min.y.min(next_rect.min.y);
                                rect.max.y = rect.max.y.max(next_rect.max.y);
                            } else {
                                rect.absorb(&next_rect);
                            }
                        } else {
                            rq.add(RenderData::new(self.id, rect, UpdateMode::Gui));
                            rect = next_rect;
                        }
                        i += 1;
                    }
                    rq.add(RenderData::new(self.id, rect, UpdateMode::Gui));
                }
            }

            if end_low != end_high {
                if let Some(mut i) = rects.iter().rposition(|(_, loc)| *loc == end_high) {
                    let mut rect = rects[i].0;
                    while rects[i].1 > end_low {
                        let prev_rect = rects[i - 1].0;
                        if rect.max.y.min(prev_rect.max.y) - rect.min.y.max(prev_rect.min.y)
                            > rect.height().min(prev_rect.height()) as i32 / 2
                        {
                            if rects[i - 1].1 == end_low {
                                if rect.min.x < prev_rect.min.x {
                                    rect.max.x = prev_rect.min.x;
                                } else {
                                    rect.min.x = prev_rect.max.x;
                                }
                                rect.min.y = rect.min.y.min(prev_rect.min.y);
                                rect.max.y = rect.max.y.max(prev_rect.max.y);
                            } else {
                                rect.absorb(&prev_rect);
                            }
                        } else {
                            rq.add(RenderData::new(self.id, rect, UpdateMode::Gui));
                            rect = prev_rect;
                        }
                        i -= 1;
                    }
                    rq.add(RenderData::new(self.id, rect, UpdateMode::Gui));
                }
            }

            selection.start = start;
            selection.end = end;
        }
        true
    }

    fn handle_selection_up(
        &mut self,
        center: Point,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &Context,
    ) -> bool {
        use crate::unit::scale_by_dpi;

        let dmax = (scale_by_dpi(RECT_DIST_JITTER, CURRENT_DEVICE.dpi) as i32).pow(2) as u32;
        let mut dmin = u32::MAX;
        let mut found = None;
        let mut rects = Vec::new();

        for chunk in &self.chunks {
            for word in &self.text[&chunk.location] {
                let rect = (word.rect * chunk.scale).to_rect() - chunk.frame.min + chunk.position;
                rects.push((rect, word.location));
                let d = center.rdist2(&rect);
                if d < dmax && d < dmin {
                    dmin = d;
                    found = Some((word.clone(), rects.len() - 1));
                }
            }
        }

        let Some(selection) = self.selection.as_mut() else {
            return true;
        };

        if let Some((word, index)) = found {
            let old_start = selection.start;
            let old_end = selection.end;

            let (start, end) = if word.location <= old_start {
                (word.location, old_end)
            } else if word.location >= old_end {
                (old_start, word.location)
            } else {
                let (start_index, end_index) = (
                    rects.iter().position(|(_, loc)| *loc == old_start),
                    rects.iter().position(|(_, loc)| *loc == old_end),
                );
                match (start_index, end_index) {
                    (Some(s), Some(e)) => {
                        if index - s > e - index {
                            (old_start, word.location)
                        } else {
                            (word.location, old_end)
                        }
                    }
                    (Some(..), None) => (word.location, old_end),
                    (None, Some(..)) => (old_start, word.location),
                    (None, None) => (old_start, old_end),
                }
            };

            if start == old_start && end == old_end {
                return true;
            }

            let (start_low, start_high) = old_start.min_max(start);
            if start_low != start_high {
                if let Some(mut i) = rects.iter().position(|(_, loc)| *loc == start_low) {
                    let mut rect = rects[i].0;
                    while rects[i].1 < start_high {
                        let next_rect = rects[i + 1].0;
                        if rect.max.y.min(next_rect.max.y) - rect.min.y.max(next_rect.min.y)
                            > rect.height().min(next_rect.height()) as i32 / 2
                        {
                            if rects[i + 1].1 == start_high {
                                if rect.min.x < next_rect.min.x {
                                    rect.max.x = next_rect.min.x;
                                } else {
                                    rect.min.x = next_rect.max.x;
                                }
                                rect.min.y = rect.min.y.min(next_rect.min.y);
                                rect.max.y = rect.max.y.max(next_rect.max.y);
                            } else {
                                rect.absorb(&next_rect);
                            }
                        } else {
                            rq.add(RenderData::new(self.id, rect, UpdateMode::Gui));
                            rect = next_rect;
                        }
                        i += 1;
                    }
                    rq.add(RenderData::new(self.id, rect, UpdateMode::Gui));
                }
            }

            let (end_low, end_high) = old_end.min_max(end);
            if end_low != end_high {
                if let Some(mut i) = rects.iter().rposition(|(_, loc)| *loc == end_high) {
                    let mut rect = rects[i].0;
                    while rects[i].1 > end_low {
                        let prev_rect = rects[i - 1].0;
                        if rect.max.y.min(prev_rect.max.y) - rect.min.y.max(prev_rect.min.y)
                            > rect.height().min(prev_rect.height()) as i32 / 2
                        {
                            if rects[i - 1].1 == end_low {
                                if rect.min.x < prev_rect.min.x {
                                    rect.max.x = prev_rect.min.x;
                                } else {
                                    rect.min.x = prev_rect.max.x;
                                }
                                rect.min.y = rect.min.y.min(prev_rect.min.y);
                                rect.max.y = rect.max.y.max(prev_rect.max.y);
                            } else {
                                rect.absorb(&prev_rect);
                            }
                        } else {
                            rq.add(RenderData::new(self.id, rect, UpdateMode::Gui));
                            rect = prev_rect;
                        }
                        i -= 1;
                    }
                    rq.add(RenderData::new(self.id, rect, UpdateMode::Gui));
                }
            }

            selection.start = start;
            selection.end = end;
        }
        true
    }
}
