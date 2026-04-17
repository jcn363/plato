//! Reader Gestures and Input Module
//!
//! Handles touch gestures, button input, stylus interaction, and event processing.
//!
//! This module contains extracted event handlers from `Reader::handle_event()` for improved
//! maintainability and testability.

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::framebuffer::UpdateMode;
use crate::frontlight::LightLevels;
use crate::geom::{Axis, CycleDir, DiagDir, Dir, LinearDir, Point, Rectangle};
use crate::gesture::GestureEvent;
use crate::input::{ButtonCode, ButtonStatus, DeviceEvent, FingerStatus};
use crate::metadata::{ScrollMode, ZoomMode};
use crate::settings::guess_frontlight;
use crate::settings::BottomRightGestureAction;

use super::reader::Reader;
use super::reader_core::State;
use crate::view::{Event, Hub, RenderData, RenderQueue};

#[allow(dead_code)] // Used in handle_selection_motion and handle_selection_up functions
const RECT_DIST_JITTER: f32 = 15.0;

/// Update selection rectangles in forward direction (for handle_selection_motion)
#[allow(dead_code)] // Selection rectangle updates for gesture handling
fn update_selection_rects_forward(
    rects: &[(Rectangle, Point)],
    boundary_low: Point,
    boundary_high: Point,
    rq: &mut RenderQueue,
    view_id: crate::view::Id,
) {
    let Some(mut i) = rects.iter().position(|(_, loc)| *loc == boundary_low) else {
        return;
    };

    let mut rect = rects[i].0;
    while rects[i].1 < boundary_high {
        let next_rect = rects[i + 1].0;
        let should_merge = rect.max.y.min(next_rect.max.y) - rect.min.y.max(next_rect.min.y)
            > rect.height().min(next_rect.height()) as i32 / 2;

        if should_merge {
            if rects[i + 1].1 == boundary_high {
                merge_rects_at_boundary(&mut rect, next_rect);
            } else {
                rect.absorb(&next_rect);
            }
        } else {
            rq.add(RenderData::new(view_id, rect, UpdateMode::Gui));
            rect = next_rect;
        }
        i += 1;
    }
    rq.add(RenderData::new(view_id, rect, UpdateMode::Gui));
}

/// Update selection rectangles in backward direction (for handle_selection_up)
#[allow(dead_code)] // Backward selection rectangle update for gesture handling
fn update_selection_rects_backward(
    rects: &[(Rectangle, Point)],
    boundary_low: Point,
    boundary_high: Point,
    rq: &mut RenderQueue,
    view_id: crate::view::Id,
) {
    let Some(mut i) = rects.iter().rposition(|(_, loc)| *loc == boundary_high) else {
        return;
    };

    let mut rect = rects[i].0;
    while rects[i].1 > boundary_low {
        let prev_rect = rects[i - 1].0;
        let should_merge = rect.max.y.min(prev_rect.max.y) - rect.min.y.max(prev_rect.min.y)
            > rect.height().min(prev_rect.height()) as i32 / 2;

        if should_merge {
            if rects[i - 1].1 == boundary_low {
                merge_rects_at_boundary(&mut rect, prev_rect);
            } else {
                rect.absorb(&prev_rect);
            }
        } else {
            rq.add(RenderData::new(view_id, rect, UpdateMode::Gui));
            rect = prev_rect;
        }
        i -= 1;
    }
    rq.add(RenderData::new(view_id, rect, UpdateMode::Gui));
}

/// Merge two rectangles at a selection boundary, adjusting the active rect's bounds
fn merge_rects_at_boundary(rect: &mut Rectangle, other: Rectangle) {
    if rect.min.x < other.min.x {
        rect.max.x = other.min.x;
    } else {
        rect.min.x = other.max.x;
    }
    rect.min.y = rect.min.y.min(other.min.y);
    rect.max.y = rect.max.y.max(other.max.y);
}

/// Update selection rectangles by dispatching to forward or backward handler
#[allow(dead_code)] // Used for selection rectangle updates in gesture handling
fn update_selection_rects(
    rects: &[(Rectangle, Point)],
    boundary_low: Point,
    boundary_high: Point,
    rq: &mut RenderQueue,
    view_id: crate::view::Id,
    is_forward: bool,
) {
    if boundary_low == boundary_high {
        return;
    }

    if is_forward {
        update_selection_rects_forward(rects, boundary_low, boundary_high, rq, view_id);
    } else {
        update_selection_rects_backward(rects, boundary_low, boundary_high, rq, view_id);
    }
}

impl Reader {
    #[allow(dead_code)] // Used by event handling system
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

    #[allow(dead_code)] // Used by event handling system
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

    #[allow(dead_code)] // Used by gesture event handling
    fn handle_selection_motion(
        &mut self,
        position: Point,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &Context,
    ) -> bool {
        let (nearest_word, rects) = self.find_nearest_word_and_rects(position);

        if let Some(word) = nearest_word {
            self.update_selection_from_word(word, rects, rq);
        }
        true
    }

    /// Find the nearest word to the given position and return rects
    #[allow(dead_code)] // Used by handle_selection_motion
    fn find_nearest_word_and_rects(
        &self,
        position: Point,
    ) -> (
        Option<crate::document::BoundedText>,
        Vec<(Rectangle, Point)>,
    ) {
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

        (nearest_word, rects)
    }

    /// Update selection bounds based on a new word and render changes
    #[allow(dead_code)] // Used by handle_selection_motion
    fn update_selection_from_word(
        &mut self,
        word: crate::document::BoundedText,
        rects: Vec<(Rectangle, Point)>,
        rq: &mut RenderQueue,
    ) {
        let Some((old_start, old_end)) = self.get_selection_bounds() else {
            return;
        };
        let Some(selection) = self.selection.as_mut() else {
            return;
        };

        let anchor = selection.anchor;
        let (start, end) = word.location.min_max(anchor);

        if start == old_start && end == old_end {
            return;
        }

        selection.start = start;
        selection.end = end;

        // Render changes after updating selection to avoid borrowing conflicts
        self.render_selection_changes(&rects, old_start, old_end, start, end, rq);
    }

    /// Get current selection bounds or return early if none exists
    #[allow(dead_code)] // Used by update_selection_from_word
    fn get_selection_bounds(&self) -> Option<(Point, Point)> {
        self.selection
            .as_ref()
            .map(|selection| (selection.start, selection.end))
    }

    /// Render selection changes using the shared rect update logic
    #[allow(dead_code)] // Used by update_selection_from_word
    fn render_selection_changes(
        &self,
        rects: &[(Rectangle, Point)],
        old_start: Point,
        old_end: Point,
        start: Point,
        end: Point,
        rq: &mut RenderQueue,
    ) {
        let (start_low, start_high) = old_start.min_max(start);
        let (end_low, end_high) = old_end.min_max(end);

        update_selection_rects(&rects, start_low, start_high, rq, self.id, true);
        update_selection_rects(&rects, end_low, end_high, rq, self.id, false);
    }

    #[allow(dead_code)] // Used by gesture event handling
    fn handle_selection_up(
        &mut self,
        center: Point,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &Context,
    ) -> bool {
        let (found_word, rects) = self.find_word_at_center(center);

        if let Some((word, index)) = found_word {
            self.finalize_selection(word, index, rects, rq);
        }
        true
    }

    /// Find the word at the given center position and return with rects
    #[allow(dead_code)] // Used by handle_selection_up
    fn find_word_at_center(
        &self,
        center: Point,
    ) -> (
        Option<(crate::document::BoundedText, usize)>,
        Vec<(Rectangle, Point)>,
    ) {
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

        (found, rects)
    }

    /// Finalize selection bounds based on the tapped word
    #[allow(dead_code)] // Used by handle_selection_up
    fn finalize_selection(
        &mut self,
        word: crate::document::BoundedText,
        index: usize,
        rects: Vec<(Rectangle, Point)>,
        rq: &mut RenderQueue,
    ) {
        let Some((old_start, old_end)) = self.get_selection_bounds() else {
            return;
        };
        let (start, end) = self.calculate_selection_bounds(word, index, &rects, old_start, old_end);

        if start == old_start && end == old_end {
            return;
        }

        self.render_selection_changes(&rects, old_start, old_end, start, end, rq);
        self.update_selection_bounds(start, end);
    }

    /// Update the selection bounds in the selection struct
    #[allow(dead_code)] // Used by finalize_selection
    fn update_selection_bounds(&mut self, start: Point, end: Point) {
        if let Some(selection) = self.selection.as_mut() {
            selection.start = start;
            selection.end = end;
        }
    }

    /// Calculate the final selection bounds based on word position
    #[allow(dead_code)] // Used by finalize_selection
    fn calculate_selection_bounds(
        &self,
        word: crate::document::BoundedText,
        index: usize,
        rects: &[(Rectangle, Point)],
        old_start: Point,
        old_end: Point,
    ) -> (Point, Point) {
        if word.location <= old_start {
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
        }
    }
}
