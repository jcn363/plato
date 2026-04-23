//! Gesture Processing
//!
//! Functions for interpreting touch segments and processing gesture pairs.

use crate::geom::{Axis, Dir, Point, Vec2};
use crate::geom::elbow;
use crate::geom::nearest_segment_point;
use crate::view::Event;
use std::sync::mpsc::Sender;

use super::types::GestureEvent;

pub fn process_segments(sg: &mut Vec<Vec<Point>>, ty: Sender<Event>, tap_jitter: f32) {
    let len = sg.len();
    if len == 1 {
        if let Some(seg) = sg.pop() {
            ty.send(Event::Gesture(interpret_segment(&seg, tap_jitter)))
                .ok();
        }
    } else if len == 2 {
        let ge1 = sg.pop().map(|seg| interpret_segment(&seg, tap_jitter));
        let ge2 = sg.pop().map(|seg| interpret_segment(&seg, tap_jitter));
        if let (Some(ge1), Some(ge2)) = (ge1, ge2) {
            process_gesture_pair(ge1, ge2, &ty);
        }
    } else {
        sg.clear();
    }
}

pub fn process_gesture_pair(ge1: GestureEvent, ge2: GestureEvent, ty: &Sender<Event>) {
    match (ge1, ge2) {
        (GestureEvent::Tap(c1), GestureEvent::Tap(c2)) => {
            ty.send(Event::Gesture(GestureEvent::MultiTap([c1, c2])))
                .ok();
        }
        (
            GestureEvent::Swipe {
                dir: d1,
                start: s1,
                end: e1,
                ..
            },
            GestureEvent::Swipe {
                dir: d2,
                start: s2,
                end: e2,
                ..
            },
        ) if d1 == d2 => {
            ty.send(Event::Gesture(GestureEvent::MultiSwipe {
                dir: d1,
                starts: [s1, s2],
                ends: [e1, e2],
            }))
            .ok();
        }
        (
            GestureEvent::Swipe {
                dir: d1,
                start: s1,
                end: e1,
                ..
            },
            GestureEvent::Swipe {
                dir: d2,
                start: s2,
                end: e2,
                ..
            },
        ) if d1 == d2.opposite() => {
            let center = (s1 + s2) / 2;
            let ds = (s2 - s1).length();
            let de = (e2 - e1).length();
            let factor = de / ds;
            if factor < 1.0 {
                ty.send(Event::Gesture(GestureEvent::Pinch {
                    axis: d1.axis(),
                    center,
                    factor,
                }))
                .ok();
            } else {
                ty.send(Event::Gesture(GestureEvent::Spread {
                    axis: d1.axis(),
                    center,
                    factor,
                }))
                .ok();
            }
        }
        (
            GestureEvent::SlantedSwipe {
                dir: d1,
                start: s1,
                end: e1,
                ..
            },
            GestureEvent::SlantedSwipe {
                dir: d2,
                start: s2,
                end: e2,
                ..
            },
        ) if d1 == d2.opposite() => {
            let center = (s1 + s2) / 2;
            let ds = (s2 - s1).length();
            let de = (e2 - e1).length();
            let factor = de / ds;
            if factor < 1.0 {
                ty.send(Event::Gesture(GestureEvent::Pinch {
                    axis: Axis::Diagonal,
                    center,
                    factor,
                }))
                .ok();
            } else {
                ty.send(Event::Gesture(GestureEvent::Spread {
                    axis: Axis::Diagonal,
                    center,
                    factor,
                }))
                .ok();
            }
        }
        (
            GestureEvent::Arrow {
                dir: Dir::East,
                start: s1,
                end: e1,
            },
            GestureEvent::Arrow {
                dir: Dir::West,
                start: s2,
                end: e2,
            },
        )
        | (
            GestureEvent::Arrow {
                dir: Dir::West,
                start: s2,
                end: e2,
            },
            GestureEvent::Arrow {
                dir: Dir::East,
                start: s1,
                end: e1,
            },
        ) if s1.x < s2.x => {
            ty.send(Event::Gesture(GestureEvent::Cross((s1 + e1 + s2 + e2) / 4)))
                .ok();
        }
        (
            GestureEvent::Arrow {
                dir: Dir::West,
                start: s1,
                end: e1,
            },
            GestureEvent::Arrow {
                dir: Dir::East,
                start: s2,
                end: e2,
            },
        )
        | (
            GestureEvent::Arrow {
                dir: Dir::East,
                start: s2,
                end: e2,
            },
            GestureEvent::Arrow {
                dir: Dir::West,
                start: s1,
                end: e1,
            },
        ) if s1.x < s2.x => {
            ty.send(Event::Gesture(GestureEvent::Diamond(
                (s1 + e1 + s2 + e2) / 4,
            )))
            .ok();
        }
        (
            GestureEvent::Arrow {
                dir: d1,
                start: s1,
                end: e1,
            },
            GestureEvent::Arrow {
                dir: d2,
                start: s2,
                end: e2,
            },
        ) if d1 == d2 => {
            ty.send(Event::Gesture(GestureEvent::MultiArrow {
                dir: d1,
                starts: [s1, s2],
                ends: [e1, e2],
            }))
            .ok();
        }
        (
            GestureEvent::Corner {
                dir: d1,
                start: s1,
                end: e1,
            },
            GestureEvent::Corner {
                dir: d2,
                start: s2,
                end: e2,
            },
        ) if d1 == d2 => {
            ty.send(Event::Gesture(GestureEvent::MultiCorner {
                dir: d1,
                starts: [s1, s2],
                ends: [e1, e2],
            }))
            .ok();
        }
        (
            GestureEvent::Tap(c),
            GestureEvent::Swipe {
                start: s, end: e, ..
            },
        )
        | (
            GestureEvent::Swipe {
                start: s, end: e, ..
            },
            GestureEvent::Tap(c),
        )
        | (
            GestureEvent::Tap(c),
            GestureEvent::Arrow {
                start: s, end: e, ..
            },
        )
        | (
            GestureEvent::Arrow {
                start: s, end: e, ..
            },
            GestureEvent::Tap(c),
        )
        | (
            GestureEvent::Tap(c),
            GestureEvent::Corner {
                start: s, end: e, ..
            },
        )
        | (
            GestureEvent::Corner {
                start: s, end: e, ..
            },
            GestureEvent::Tap(c),
        ) => {
            let angle = ((e - c).angle() - (s - c).angle()).to_degrees();
            let quarter_turns = (angle / 90.0).round() as i8;
            ty.send(Event::Gesture(GestureEvent::Rotate {
                angle,
                quarter_turns,
                center: c,
            }))
            .ok();
        }
        _ => (),
    }
}

pub fn interpret_segment(sp: &[Point], tap_jitter: f32) -> GestureEvent {
    let a = sp[0];
    let b = sp[sp.len() - 1];
    let ab = b - a;
    let d = ab.length();
    if d < tap_jitter {
        GestureEvent::Tap(a)
    } else {
        let p = sp[elbow(sp)];
        let (n, p) = {
            let p: Vec2 = p.into();
            let (n, _) = nearest_segment_point(p, a.into(), b.into());
            (n, p)
        };
        let np = p - n;
        let ds = np.length();
        if ds > d / 5.0 {
            let g = (np.x / np.y).abs();
            if !(0.5..=2.0).contains(&g) {
                GestureEvent::Arrow {
                    dir: np.dir(),
                    start: a,
                    end: b,
                }
            } else {
                GestureEvent::Corner {
                    dir: np.diag_dir(),
                    start: a,
                    end: b,
                }
            }
        } else {
            let g = (ab.x as f32 / ab.y as f32).abs();
            if !(0.5..=2.0).contains(&g) {
                GestureEvent::Swipe {
                    start: a,
                    end: b,
                    dir: ab.dir(),
                }
            } else {
                GestureEvent::SlantedSwipe {
                    start: a,
                    end: b,
                    dir: ab.diag_dir(),
                }
            }
        }
    }
}
