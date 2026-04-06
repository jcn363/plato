use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::pt;

use super::helpers::Edge;
use super::point::Point;

// Based on https://golang.org/pkg/image/#Rectangle
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Rectangle {
    pub min: Point,
    pub max: Point,
}

impl fmt::Display for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}, {}, {}, {}]",
            self.min.x, self.min.y, self.max.x, self.max.y
        )
    }
}

impl Rectangle {
    #[inline]
    #[must_use]
    pub fn new(min: Point, max: Point) -> Rectangle {
        Rectangle { min, max }
    }

    #[inline]
    #[must_use]
    pub fn from_point(pt: Point) -> Rectangle {
        Rectangle {
            min: pt,
            max: pt + 1,
        }
    }

    #[inline]
    #[must_use]
    pub fn from_disk(center: Point, radius: i32) -> Rectangle {
        Rectangle {
            min: center - radius,
            max: center + radius,
        }
    }

    #[inline]
    #[must_use]
    pub fn from_segment(start: Point, end: Point, start_radius: i32, end_radius: i32) -> Rectangle {
        let x_min = (start.x - start_radius).min(end.x - end_radius);
        let x_max = (start.x + start_radius).max(end.x + end_radius);
        let y_min = (start.y - start_radius).min(end.y - end_radius);
        let y_max = (start.y + start_radius).max(end.y + end_radius);
        Rectangle {
            min: pt!(x_min, y_min),
            max: pt!(x_max, y_max),
        }
    }

    #[inline]
    #[must_use]
    pub fn diag2(&self) -> u32 {
        self.min.dist2(self.max)
    }

    #[inline]
    #[must_use]
    pub fn includes(&self, pt: Point) -> bool {
        self.min.x <= pt.x && pt.x < self.max.x && self.min.y <= pt.y && pt.y < self.max.y
    }

    #[inline]
    #[must_use]
    pub fn contains(&self, rect: &Rectangle) -> bool {
        rect.min.x >= self.min.x
            && rect.max.x <= self.max.x
            && rect.min.y >= self.min.y
            && rect.max.y <= self.max.y
    }

    #[inline]
    #[must_use]
    pub fn overlaps(&self, rect: &Rectangle) -> bool {
        self.min.x < rect.max.x
            && rect.min.x < self.max.x
            && self.min.y < rect.max.y
            && rect.min.y < self.max.y
    }

    #[inline]
    #[must_use]
    pub fn extends(&self, rect: &Rectangle) -> bool {
        let dmin = [self.width(), self.height(), rect.width(), rect.height()]
            .into_iter()
            .min()
            .expect("conversion failed") as i32
            / 3;

        // rect is on top of self.
        if self.min.y >= rect.max.y && self.min.x < rect.max.x && rect.min.x < self.max.x {
            (self.min.y - rect.max.y) <= dmin
        // rect is at the right of self.
        } else if rect.min.x >= self.max.x && self.min.y < rect.max.y && rect.min.y < self.max.y {
            (rect.min.x - self.max.x) <= dmin
        // rect is on bottom of self.
        } else if rect.min.y >= self.max.y && self.min.x < rect.max.x && self.min.x < self.max.x {
            (rect.min.y - self.max.y) <= dmin
        // rect is at the left of self.
        } else if self.min.x >= rect.max.x && self.min.y < rect.max.y && rect.min.y < self.max.y {
            (self.min.x - rect.max.x) <= dmin
        } else {
            false
        }
    }

    #[inline]
    #[must_use]
    pub fn touches(&self, rect: &Rectangle) -> bool {
        ((self.min.x == rect.max.x
            || self.max.x == rect.min.x
            || self.min.x == rect.min.x
            || self.max.x == rect.max.x)
            && (self.max.y >= rect.min.y && self.min.y <= rect.max.y))
            || ((self.min.y == rect.max.y
                || self.max.y == rect.min.y
                || self.min.y == rect.min.y
                || self.max.y == rect.max.y)
                && (self.max.x >= rect.min.x && self.min.x <= rect.max.x))
    }

    #[inline]
    pub fn merge(&mut self, pt: Point) {
        if pt.x < self.min.x {
            self.min.x = pt.x;
        }
        if pt.x >= self.max.x {
            self.max.x = pt.x + 1;
        }
        if pt.y < self.min.y {
            self.min.y = pt.y;
        }
        if pt.y >= self.max.y {
            self.max.y = pt.y + 1;
        }
    }

    pub fn absorb(&mut self, rect: &Rectangle) {
        if self.min.x > rect.min.x {
            self.min.x = rect.min.x;
        }
        if self.max.x < rect.max.x {
            self.max.x = rect.max.x;
        }
        if self.min.y > rect.min.y {
            self.min.y = rect.min.y;
        }
        if self.max.y < rect.max.y {
            self.max.y = rect.max.y;
        }
    }

    #[inline]
    #[must_use]
    pub fn intersection(&self, rect: &Rectangle) -> Option<Rectangle> {
        if self.overlaps(rect) {
            Some(Rectangle::new(
                Point::new(self.min.x.max(rect.min.x), self.min.y.max(rect.min.y)),
                Point::new(self.max.x.min(rect.max.x), self.max.y.min(rect.max.y)),
            ))
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.max.x <= self.min.x || self.max.y <= self.min.y
    }

    #[inline]
    #[must_use]
    pub fn width(&self) -> u32 {
        (self.max.x - self.min.x) as u32
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> u32 {
        (self.max.y - self.min.y) as u32
    }

    #[inline]
    #[must_use]
    pub fn ratio(&self) -> f32 {
        self.width() as f32 / self.height() as f32
    }

    #[inline]
    #[must_use]
    pub fn area(&self) -> u32 {
        self.width() * self.height()
    }

    #[inline]
    #[must_use]
    pub fn center(&self) -> Point {
        (self.min + self.max) / 2
    }

    pub fn grow(&mut self, edges: &Edge) {
        self.min.x -= edges.left;
        self.min.y -= edges.top;
        self.max.x += edges.right;
        self.max.y += edges.bottom;
    }

    pub fn shrink(&mut self, edges: &Edge) {
        self.min.x += edges.left;
        self.min.y += edges.top;
        self.max.x -= edges.right;
        self.max.y -= edges.bottom;
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Rectangle::new(Point::default(), Point::default())
    }
}

impl From<(u32, u32)> for Rectangle {
    fn from(dims: (u32, u32)) -> Rectangle {
        Rectangle::new(Point::new(0, 0), Point::new(dims.0 as i32, dims.1 as i32))
    }
}

impl PartialOrd for Rectangle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rectangle {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.min.y >= other.max.y {
            Ordering::Greater
        } else if self.max.y <= other.min.y {
            Ordering::Less
        } else {
            if self.min.x >= other.max.x {
                Ordering::Greater
            } else if self.max.x <= other.min.x {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        }
    }
}

impl Add<Point> for Rectangle {
    type Output = Rectangle;
    fn add(self, rhs: Point) -> Rectangle {
        Rectangle {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

impl AddAssign<Point> for Rectangle {
    fn add_assign(&mut self, rhs: Point) {
        self.min += rhs;
        self.max += rhs;
    }
}

impl Sub<Point> for Rectangle {
    type Output = Rectangle;
    fn sub(self, rhs: Point) -> Rectangle {
        Rectangle {
            min: self.min - rhs,
            max: self.max - rhs,
        }
    }
}

impl SubAssign<Point> for Rectangle {
    fn sub_assign(&mut self, rhs: Point) {
        self.min -= rhs;
        self.max -= rhs;
    }
}
