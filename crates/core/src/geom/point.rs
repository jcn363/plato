use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use serde::{Deserialize, Serialize};

use super::enums::{DiagDir, Dir};
use super::rectangle::Rectangle;
use super::vec2::Vec2;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Point {
    #[inline]
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    #[inline]
    pub fn min_max(self, other: Self) -> (Self, Self) {
        if self.x < other.x || (self.x == other.x && self.y < other.y) {
            (self, other)
        } else {
            (other, self)
        }
    }

    #[inline]
    pub fn dist2(self, pt: Point) -> u32 {
        ((pt.x - self.x).pow(2) + (pt.y - self.y).pow(2)) as u32
    }

    #[inline]
    pub fn rdist2(self, rect: &Rectangle) -> u32 {
        if rect.includes(self) {
            0
        } else if self.y >= rect.min.y && self.y < rect.max.y {
            if self.x < rect.min.x {
                (rect.min.x - self.x).pow(2) as u32
            } else {
                (self.x - rect.max.x + 1).pow(2) as u32
            }
        } else if self.x >= rect.min.x && self.x < rect.max.x {
            if self.y < rect.min.y {
                (rect.min.y - self.y).pow(2) as u32
            } else {
                (self.y - rect.max.y + 1).pow(2) as u32
            }
        } else if self.x < rect.min.x {
            if self.y < rect.min.y {
                self.dist2(rect.min)
            } else {
                self.dist2(Point::new(rect.min.x, rect.max.y - 1))
            }
        } else {
            if self.y < rect.min.y {
                self.dist2(Point::new(rect.max.x - 1, rect.min.y))
            } else {
                self.dist2(Point::new(rect.max.x - 1, rect.max.y - 1))
            }
        }
    }

    #[inline]
    pub fn length(self) -> f32 {
        ((self.x.pow(2) + self.y.pow(2)) as f32).sqrt()
    }

    #[inline]
    pub fn angle(self) -> f32 {
        (-self.y as f32).atan2(self.x as f32)
    }

    #[inline]
    pub fn dir(self) -> Dir {
        if self.x.abs() > self.y.abs() {
            if self.x.is_positive() {
                Dir::East
            } else {
                Dir::West
            }
        } else {
            if self.y.is_positive() {
                Dir::South
            } else {
                Dir::North
            }
        }
    }

    #[inline]
    pub fn diag_dir(self) -> DiagDir {
        if self.x.is_positive() {
            if self.y.is_positive() {
                DiagDir::SouthEast
            } else {
                DiagDir::NorthEast
            }
        } else {
            if self.y.is_positive() {
                DiagDir::SouthWest
            } else {
                DiagDir::NorthWest
            }
        }
    }
}

impl Default for Point {
    fn default() -> Self {
        Point::new(0, 0)
    }
}

impl Into<(f32, f32)> for Point {
    fn into(self) -> (f32, f32) {
        (self.x as f32, self.y as f32)
    }
}

impl From<Point> for Vec2 {
    fn from(pt: Point) -> Self {
        Vec2::new(pt.x as f32, pt.y as f32)
    }
}

impl From<Vec2> for Point {
    fn from(pt: Vec2) -> Self {
        Point::new(pt.x as i32, pt.y as i32)
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Point) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, rhs: Point) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<Point> for Point {
    type Output = Point;
    fn mul(self, rhs: Point) -> Point {
        Point {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl MulAssign<Point> for Point {
    fn mul_assign(&mut self, rhs: Point) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl Div<Point> for Point {
    type Output = Point;
    fn div(self, rhs: Point) -> Point {
        Point {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl DivAssign<Point> for Point {
    fn div_assign(&mut self, rhs: Point) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl Add<i32> for Point {
    type Output = Point;
    fn add(self, rhs: i32) -> Point {
        Point {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl Add<Point> for i32 {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point {
            x: self + rhs.x,
            y: self + rhs.y,
        }
    }
}

impl AddAssign<i32> for Point {
    fn add_assign(&mut self, rhs: i32) {
        self.x += rhs;
        self.y += rhs;
    }
}

impl Sub<i32> for Point {
    type Output = Point;
    fn sub(self, rhs: i32) -> Point {
        Point {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl Sub<Point> for i32 {
    type Output = Point;
    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self - rhs.x,
            y: self - rhs.y,
        }
    }
}

impl SubAssign<i32> for Point {
    fn sub_assign(&mut self, rhs: i32) {
        self.x -= rhs;
        self.y -= rhs;
    }
}

impl Mul<i32> for Point {
    type Output = Point;
    fn mul(self, rhs: i32) -> Point {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Point> for i32 {
    type Output = Point;
    fn mul(self, rhs: Point) -> Point {
        Point {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl MulAssign<i32> for Point {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<i32> for Point {
    type Output = Point;
    fn div(self, rhs: i32) -> Point {
        Point {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<Point> for i32 {
    type Output = Point;
    fn div(self, rhs: Point) -> Point {
        Point {
            x: self / rhs.x,
            y: self / rhs.y,
        }
    }
}

impl DivAssign<i32> for Point {
    fn div_assign(&mut self, rhs: i32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
