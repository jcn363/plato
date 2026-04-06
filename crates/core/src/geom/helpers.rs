use std::f32::consts;

use crate::color::Color;

use super::point::Point;
use super::vec2::Vec2;

#[derive(Debug, Copy, Clone, Default)]
pub struct Edge {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl Edge {
    pub fn uniform(value: i32) -> Edge {
        Edge {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

impl std::ops::Add for Edge {
    type Output = Edge;
    fn add(self, rhs: Edge) -> Edge {
        Edge {
            top: self.top + rhs.top,
            right: self.right + rhs.right,
            bottom: self.bottom + rhs.bottom,
            left: self.left + rhs.left,
        }
    }
}

impl std::ops::AddAssign for Edge {
    fn add_assign(&mut self, rhs: Edge) {
        self.top += rhs.top;
        self.right += rhs.right;
        self.bottom += rhs.bottom;
        self.left += rhs.left;
    }
}

impl std::ops::Sub for Edge {
    type Output = Edge;
    fn sub(self, rhs: Edge) -> Edge {
        Edge {
            top: self.top - rhs.top,
            right: self.right - rhs.right,
            bottom: self.bottom - rhs.bottom,
            left: self.left - rhs.left,
        }
    }
}

impl std::ops::SubAssign for Edge {
    fn sub_assign(&mut self, rhs: Edge) {
        self.top -= rhs.top;
        self.right -= rhs.right;
        self.bottom -= rhs.bottom;
        self.left -= rhs.left;
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CornerSpec {
    Uniform(i32),
    North(i32),
    East(i32),
    South(i32),
    West(i32),
    Detailed {
        north_west: i32,
        north_east: i32,
        south_east: i32,
        south_west: i32,
    },
}

pub trait ColorSource {
    fn color(&self, x: i32, y: i32) -> Color;
}

impl<F> ColorSource for F
where
    F: Fn(i32, i32) -> Color,
{
    #[inline]
    fn color(&self, x: i32, y: i32) -> Color {
        (self)(x, y)
    }
}

impl ColorSource for Color {
    #[inline]
    fn color(&self, _: i32, _: i32) -> Color {
        *self
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BorderSpec {
    pub thickness: u16,
    pub color: Color,
}

const HALF_PIXEL_DIAGONAL: f32 = consts::SQRT_2 / 2.0;

// Takes the (signed) distance and angle from the center of a pixel to the closest point on a
// shape's boundary and returns the approximate shape area contained within that pixel (the
/// Computes the fraction of a pixel's surface area that lies inside a shape boundary.
///
/// Used for anti-aliased rendering. The boundary is defined by distance from the pixel center
/// and angle of the normal vector.
///
/// # Arguments
/// * `dist` - Signed distance from pixel center to boundary (positive = inside)
/// * `angle` - Angle of the boundary normal in radians
///
/// # Example
/// ```
/// let area = surface_area(0.0, 0.0);  // Boundary through center, parallel to pixel sides
/// let area = surface_area(0.25, std::f32::consts::FRAC_PI_4);  // Diagonal boundary
/// ```
#[inline]
pub fn surface_area(dist: f32, angle: f32) -> f32 {
    // Clearly {in,out}side of the shape.
    if dist.abs() > HALF_PIXEL_DIAGONAL {
        if dist.is_sign_positive() {
            return 0.0;
        } else {
            return 1.0;
        }
    }
    // If the boundary is parallel to the pixel's diagonals then the area is proportional to `dist²`.
    // If the boundary is parallel to the pixel's sides then the area is proportional to `dist`.
    // Hence we compute an interpolated exponent `expo` (`1 <= expo <= 2`) based on `angle`.
    let expo = 0.5 * (3.0 - (4.0 * angle).cos());
    // The *radius* of the pixel for the given *angle*
    let radius = 0.5 * expo.sqrt();
    if dist.is_sign_positive() {
        (radius - dist).max(0.0).powf(expo)
    } else {
        1.0 - (radius + dist).max(0.0).powf(expo)
    }
}

/// Finds the nearest point on line segment `ab` to point `p`.
///
/// Uses vector projection to find the closest point, then clamps to segment endpoints.
/// Returns both the nearest point and the parameter `t` (0.0 = at `a`, 1.0 = at `b`).
///
/// # Arguments
/// * `p` - Query point
/// * `a` - Segment start
/// * `b` - Segment end
///
/// # Returns
/// `(nearest_point, t)` where `t` is in range [0.0, 1.0]
///
/// # Example
/// ```
/// let p = Point::new(5.0, 5.0);
/// let (nearest, t) = nearest_segment_point(p.into(), Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0));
/// assert!(t > 0.0 && t < 1.0);  // Point projects onto segment interior
/// ```
#[inline]
pub fn nearest_segment_point(p: Vec2, a: Vec2, b: Vec2) -> (Vec2, f32) {
    let ab = b - a;
    let ap = p - a;
    let l2 = ab.dot(ab);

    // Will not happen in practice
    if l2 < f32::EPSILON {
        return (a, 0.0);
    }

    let t = (ap.dot(ab) / l2).clamp(0.0, 1.0);
    (a + t * ab, t)
}

pub fn elbow(sp: &[Point]) -> usize {
    let len = sp.len();
    let a: Vec2 = sp[0].into();
    let b: Vec2 = sp[len - 1].into();
    let i1 = len / 3;
    let i2 = 2 * len / 3;
    let p1: Vec2 = sp[i1].into();
    let p2: Vec2 = sp[i2].into();
    let (n1, _) = nearest_segment_point(p1, a, b);
    let (n2, _) = nearest_segment_point(p2, a, b);
    let d1 = (p1 - n1).length();
    let d2 = (p2 - n2).length();
    if d1 > f32::EPSILON || d2 > f32::EPSILON {
        ((d1 * i1 as f32 + d2 * i2 as f32) / (d1 + d2)) as usize
    } else {
        len / 2
    }
}

#[inline]
pub fn halves(n: i32) -> (i32, i32) {
    let small_half = n / 2;
    let big_half = n - small_half;
    (small_half, big_half)
}

#[inline]
pub fn small_half(n: i32) -> i32 {
    n / 2
}

#[inline]
/// Returns the larger half when splitting `n` into two parts.
///
/// # Example
/// ```
/// let (small, big) = halves(10);  // small=5, big=5
/// let (small, big) = halves(11);  // small=5, big=6
/// ```
pub fn big_half(n: i32) -> i32 {
    n - small_half(n)
}

/// Divides `n` into `p` roughly equal parts.
///
/// Returns a vector of size `p` where elements differ by at most 1.
/// Used for distributing space (e.g., column widths).
///
/// # Example
/// ```
/// let parts = divide(10, 3);  // [4, 3, 3] or [3, 4, 3]
/// ```
pub fn divide(n: i32, p: i32) -> Vec<i32> {
    let size = n.checked_div(p).unwrap_or(0);
    let mut rem = n - p * size;
    let tick = p.checked_div(rem).unwrap_or(0);
    let mut vec = Vec::with_capacity(p as usize);
    for i in 0..p {
        if rem > 0 && (i + 1) % tick == 0 {
            vec.push(size + 1);
            rem -= 1;
        } else {
            vec.push(size);
        }
    }
    vec
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

// Returns the clockwise and anti-clockwise modulo p distance from a to b.
#[inline]
pub fn circular_distances(a: u16, mut b: u16, p: u16) -> (u16, u16) {
    if b < a {
        b += p;
    }
    let d0 = b - a;
    let d1 = p - d0;
    (d0, d1)
}
