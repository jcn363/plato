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
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::frontlight::LightLevels;
use crate::geom::{Axis, CycleDir, DiagDir, Dir, LinearDir, Point};
use crate::gesture::GestureEvent;
use crate::input::{ButtonCode, ButtonStatus, DeviceEvent, FingerStatus};
use crate::metadata::{ScrollMode, ZoomMode};
use crate::settings::guess_frontlight;
use crate::settings::BottomRightGestureAction;

use super::reader::Reader; // Need access to Reader for state manipulation
use super::reader_core::State;
use crate::view::{Event, Hub, RenderData, RenderQueue};

const RECT_DIST_JITTER: f32 = 15.0;

/// Trait for customizable gesture processing.
///
/// Implement this trait to provide custom gesture handling for different device types
/// or to add new gesture behaviors without modifying the core Reader implementation.
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
        center: Point,
        reader: &mut Reader,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        // Delegate tap handling to Reader's existing methods
        reader.handle_tap_event(center, hub, rq, context)
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
