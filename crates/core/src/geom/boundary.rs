use std::ops::{Div, DivAssign, Mul, MulAssign};

use super::point::Point;
use super::rectangle::Rectangle;
use super::vec2::Vec2;

#[derive(Debug, Copy, Clone)]
pub struct Boundary {
    pub min: Vec2,
    pub max: Vec2,
}

impl Boundary {
    #[inline]
    pub fn new(min: Vec2, max: Vec2) -> Boundary {
        Boundary { min, max }
    }

    #[inline]
    pub fn to_rect(&self) -> Rectangle {
        Rectangle {
            min: Point::new(self.min.x.floor() as i32, self.min.y.floor() as i32),
            max: Point::new(self.max.x.ceil() as i32, self.max.y.ceil() as i32),
        }
    }

    #[inline]
    pub fn overlaps(&self, rect: &Boundary) -> bool {
        self.min.x < rect.max.x
            && rect.min.x < self.max.x
            && self.min.y < rect.max.y
            && rect.min.y < self.max.y
    }

    #[inline]
    pub fn contains(&self, rect: &Boundary) -> bool {
        rect.min.x >= self.min.x
            && rect.max.x <= self.max.x
            && rect.min.y >= self.min.y
            && rect.max.y <= self.max.y
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
}

impl Mul<f32> for Boundary {
    type Output = Boundary;
    fn mul(self, rhs: f32) -> Boundary {
        Boundary {
            min: self.min * rhs,
            max: self.max * rhs,
        }
    }
}

impl Mul<Boundary> for f32 {
    type Output = Boundary;
    fn mul(self, rhs: Boundary) -> Boundary {
        Boundary {
            min: self * rhs.min,
            max: self * rhs.max,
        }
    }
}

impl MulAssign<f32> for Boundary {
    fn mul_assign(&mut self, rhs: f32) {
        self.min *= rhs;
        self.max *= rhs;
    }
}

impl Div<f32> for Boundary {
    type Output = Boundary;
    fn div(self, rhs: f32) -> Boundary {
        Boundary {
            min: self.min / rhs,
            max: self.max / rhs,
        }
    }
}

impl Div<Boundary> for f32 {
    type Output = Boundary;
    fn div(self, rhs: Boundary) -> Boundary {
        Boundary {
            min: rhs.min / self,
            max: rhs.max / self,
        }
    }
}

impl DivAssign<f32> for Boundary {
    fn div_assign(&mut self, rhs: f32) {
        self.min /= rhs;
        self.max /= rhs;
    }
}
