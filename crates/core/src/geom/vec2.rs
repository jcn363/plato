use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use super::enums::{DiagDir, Dir};

#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[inline]
    pub fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x: x.clamp(-10000.0, 10000.0), y: y.clamp(-10000.0, 10000.0) }
    }

    #[inline]
    pub fn dot(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn cross(self, other: Vec2) -> f32 {
        self.x * other.y - self.y * other.x
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.x.hypot(self.y)
    }

    #[inline]
    pub fn angle(self) -> f32 {
        (-self.y).atan2(self.x)
    }

    #[inline]
    pub fn dir(self) -> Dir {
        if self.x.abs() > self.y.abs() {
            if self.x.is_sign_positive() {
                Dir::East
            } else {
                Dir::West
            }
        } else {
            if self.y.is_sign_positive() {
                Dir::South
            } else {
                Dir::North
            }
        }
    }

    #[inline]
    pub fn diag_dir(self) -> DiagDir {
        if self.x.is_sign_positive() {
            if self.y.is_sign_positive() {
                DiagDir::SouthEast
            } else {
                DiagDir::NorthEast
            }
        } else {
            if self.y.is_sign_positive() {
                DiagDir::SouthWest
            } else {
                DiagDir::NorthWest
            }
        }
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: (self.x + rhs.x).clamp(-10000.0, 10000.0),
            y: (self.y + rhs.y).clamp(-10000.0, 10000.0),
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x = (self.x + rhs.x).clamp(-10000.0, 10000.0);
        self.y = (self.y + rhs.y).clamp(-10000.0, 10000.0);
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: (self.x - rhs.x).clamp(-10000.0, 10000.0),
            y: (self.y - rhs.y).clamp(-10000.0, 10000.0),
        }
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x = (self.x - rhs.x).clamp(-10000.0, 10000.0);
        self.y = (self.y - rhs.y).clamp(-10000.0, 10000.0);
    }
}

impl Mul<Vec2> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: (self.x * rhs.x).clamp(-10000.0, 10000.0),
            y: (self.y * rhs.y).clamp(-10000.0, 10000.0),
        }
    }
}

impl MulAssign<Vec2> for Vec2 {
    fn mul_assign(&mut self, rhs: Vec2) {
        self.x = (self.x * rhs.x).clamp(-10000.0, 10000.0);
        self.y = (self.y * rhs.y).clamp(-10000.0, 10000.0);
    }
}

impl Div<Vec2> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: if rhs.x.abs() < f32::EPSILON { 0.0 } else { (self.x / rhs.x).clamp(-10000.0, 10000.0) },
            y: if rhs.y.abs() < f32::EPSILON { 0.0 } else { (self.y / rhs.y).clamp(-10000.0, 10000.0) },
        }
    }
}

impl DivAssign<Vec2> for Vec2 {
    fn div_assign(&mut self, rhs: Vec2) {
        self.x = if rhs.x.abs() < f32::EPSILON { 0.0 } else { (self.x / rhs.x).clamp(-10000.0, 10000.0) };
        self.y = if rhs.y.abs() < f32::EPSILON { 0.0 } else { (self.y / rhs.y).clamp(-10000.0, 10000.0) };
    }
}

impl Add<f32> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: (self.x + rhs).clamp(-10000.0, 10000.0),
            y: (self.y + rhs).clamp(-10000.0, 10000.0),
        }
    }
}

impl Add<Vec2> for f32 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: (self + rhs.x).clamp(-10000.0, 10000.0),
            y: (self + rhs.y).clamp(-10000.0, 10000.0),
        }
    }
}

impl AddAssign<f32> for Vec2 {
    fn add_assign(&mut self, rhs: f32) {
        self.x = (self.x + rhs).clamp(-10000.0, 10000.0);
        self.y = (self.y + rhs).clamp(-10000.0, 10000.0);
    }
}

impl Sub<f32> for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: (self.x - rhs).clamp(-10000.0, 10000.0),
            y: (self.y - rhs).clamp(-10000.0, 10000.0),
        }
    }
}

impl Sub<Vec2> for f32 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self - rhs.x,
            y: self - rhs.y,
        }
    }
}

impl SubAssign<f32> for Vec2 {
    fn sub_assign(&mut self, rhs: f32) {
        self.x -= rhs;
        self.y -= rhs;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<Vec2> for f32 {
    type Output = Vec2;
    fn div(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self / rhs.x,
            y: self / rhs.y,
        }
    }
}

impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
