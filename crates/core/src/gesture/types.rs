//! Gesture Types
//!
//! Defines the GestureEvent enum and related types for touch gesture recognition.

use crate::geom::{Axis, DiagDir, Dir, Point};
use crate::input::ButtonCode;
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum GestureEvent {
    Tap(Point),
    MultiTap([Point; 2]),
    Swipe {
        dir: Dir,
        start: Point,
        end: Point,
    },
    SlantedSwipe {
        dir: DiagDir,
        start: Point,
        end: Point,
    },
    MultiSwipe {
        dir: Dir,
        starts: [Point; 2],
        ends: [Point; 2],
    },
    Arrow {
        dir: Dir,
        start: Point,
        end: Point,
    },
    MultiArrow {
        dir: Dir,
        starts: [Point; 2],
        ends: [Point; 2],
    },
    Corner {
        dir: DiagDir,
        start: Point,
        end: Point,
    },
    MultiCorner {
        dir: DiagDir,
        starts: [Point; 2],
        ends: [Point; 2],
    },
    Pinch {
        axis: Axis,
        center: Point,
        factor: f32,
    },
    Spread {
        axis: Axis,
        center: Point,
        factor: f32,
    },
    Rotate {
        center: Point,
        quarter_turns: i8,
        angle: f32,
    },
    Cross(Point),
    Diamond(Point),
    HoldFingerShort(Point, i32),
    HoldFingerLong(Point, i32),
    HoldButtonShort(ButtonCode),
    HoldButtonLong(ButtonCode),
}

impl fmt::Display for GestureEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GestureEvent::Tap(pt) => write!(f, "Tap {}", pt),
            GestureEvent::MultiTap(pts) => write!(f, "Multitap {} {}", pts[0], pts[1]),
            GestureEvent::Swipe { dir, .. } => write!(f, "Swipe {}", dir),
            GestureEvent::SlantedSwipe { dir, .. } => write!(f, "SlantedSwipe {}", dir),
            GestureEvent::MultiSwipe { dir, .. } => write!(f, "Multiswipe {}", dir),
            GestureEvent::Arrow { dir, .. } => write!(f, "Arrow {}", dir),
            GestureEvent::MultiArrow { dir, .. } => write!(f, "Multiarrow {}", dir),
            GestureEvent::Corner { dir, .. } => write!(f, "Corner {}", dir),
            GestureEvent::MultiCorner { dir, .. } => write!(f, "Multicorner {}", dir),
            GestureEvent::Pinch {
                axis,
                center,
                factor,
                ..
            } => write!(f, "Pinch {} {} {:.2}", axis, center, factor),
            GestureEvent::Spread {
                axis,
                center,
                factor,
                ..
            } => write!(f, "Spread {} {} {:.2}", axis, center, factor),
            GestureEvent::Rotate {
                center,
                quarter_turns,
                ..
            } => write!(f, "Rotate {} {}", center, *quarter_turns as i32 * 90),
            GestureEvent::Cross(pt) => write!(f, "Cross {}", pt),
            GestureEvent::Diamond(pt) => write!(f, "Diamond {}", pt),
            GestureEvent::HoldFingerShort(pt, id) => write!(f, "Short-held finger {} {}", id, pt),
            GestureEvent::HoldFingerLong(pt, id) => write!(f, "Long-held finger {} {}", id, pt),
            GestureEvent::HoldButtonShort(code) => write!(f, "Short-held button {:?}", code),
            GestureEvent::HoldButtonLong(code) => write!(f, "Long-held button {:?}", code),
        }
    }
}

#[derive(Debug)]
pub struct TouchState {
    pub time: f64,
    pub held: bool,
    pub positions: Vec<Point>,
}
