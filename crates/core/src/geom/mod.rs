mod boundary;
mod constants;
mod enums;
mod helpers;
mod point;
mod rectangle;
mod region;
mod vec2;

pub use boundary::Boundary;
pub use constants::{SQRT2, SQRT2_INV};
pub use enums::{Axis, CycleDir, DiagDir, Dir, LinearDir};
pub use helpers::{
    big_half, circular_distances, divide, elbow, halves, lerp, nearest_segment_point, small_half,
    surface_area, BorderSpec, ColorSource, CornerSpec, Edge,
};
pub use point::Point;
pub use rectangle::Rectangle;
pub use region::Region;
pub use vec2::Vec2;

#[macro_export]
macro_rules! pt {
    ($x:expr, $y:expr $(,)* ) => {
        $crate::geom::Point::new($x, $y)
    };
    ($a:expr) => {
        $crate::geom::Point::new($a, $a)
    };
}

#[macro_export]
macro_rules! vec2 {
    ($x:expr, $y:expr $(,)* ) => {
        $crate::geom::Vec2::new($x, $y)
    };
    ($a:expr) => {
        $crate::geom::Vec2::new($a, $a)
    };
}

#[macro_export]
macro_rules! rect {
    ($x0:expr, $y0:expr, $x1:expr, $y1:expr $(,)* ) => {
        $crate::geom::Rectangle::new(
            $crate::geom::Point::new($x0, $y0),
            $crate::geom::Point::new($x1, $y1),
        )
    };
    ($min:expr, $max:expr $(,)* ) => {
        $crate::geom::Rectangle::new($min, $max)
    };
}

#[macro_export]
macro_rules! bndr {
    ($x0:expr, $y0:expr, $x1:expr, $y1:expr $(,)* ) => {
        $crate::geom::Boundary::new(
            $crate::geom::Vec2::new($x0, $y0),
            $crate::geom::Vec2::new($x1, $y1),
        )
    };
    ($min:expr, $max:expr $(,)* ) => {
        $crate::geom::Boundary::new($min, $max)
    };
}

use boundary::Boundary as PrivBoundary;
use point::Point as PrivPoint;
use rectangle::Rectangle as PrivRectangle;
use vec2::Vec2 as PrivVec2;

impl PrivRectangle {
    #[inline]
    pub fn to_boundary(&self) -> PrivBoundary {
        PrivBoundary {
            min: PrivVec2::new(self.min.x as f32, self.min.y as f32),
            max: PrivVec2::new(self.max.x as f32, self.max.y as f32),
        }
    }
}

impl Into<PrivRectangle> for PrivBoundary {
    fn into(self) -> PrivRectangle {
        PrivRectangle {
            min: PrivPoint::new(self.min.x.floor() as i32, self.min.y.floor() as i32),
            max: PrivPoint::new(self.max.x.ceil() as i32, self.max.y.ceil() as i32),
        }
    }
}

impl Into<PrivBoundary> for PrivRectangle {
    fn into(self) -> PrivBoundary {
        PrivBoundary {
            min: PrivVec2::new(self.min.x as f32, self.min.y as f32),
            max: PrivVec2::new(self.max.x as f32, self.max.y as f32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{divide, LinearDir};
    use crate::{pt, rect};

    #[test]
    fn test_linear_dir_opposite() {
        assert_eq!(LinearDir::Forward.opposite(), LinearDir::Backward);
        assert_eq!(LinearDir::Backward.opposite(), LinearDir::Forward);
    }

    #[test]
    fn overlaping_rectangles() {
        let a = rect![2, 2, 10, 10];
        let b = rect![2, 5, 3, 6];
        let c = rect![1, 3, 2, 7];
        let d = rect![9, 9, 12, 12];
        let e = rect![4, 3, 5, 6];
        assert!(b.overlaps(&a));
        assert!(!c.overlaps(&a));
        assert!(d.overlaps(&a));
        assert!(e.overlaps(&a));
        assert!(a.overlaps(&e));
    }

    #[test]
    fn contained_rectangles() {
        let a = rect![2, 2, 10, 10];
        let b = rect![4, 3, 5, 6];
        let c = rect![4, 3, 12, 9];
        assert!(a.contains(&b));
        assert!(!b.contains(&a));
        assert!(!a.contains(&c));
        assert!(c.contains(&b));
    }

    #[test]
    fn extended_rectangles() {
        let a = rect![30, 30, 60, 60];
        let b = rect![23, 0, 67, 28];
        let c = rect![60, 40, 110, 80];
        let d = rect![26, 62, 55, 96];
        let e = rect![0, 25, 29, 60];
        assert!(b.extends(&a));
        assert!(c.extends(&a));
        assert!(d.extends(&a));
        assert!(e.extends(&a));
        assert!(!b.extends(&d));
        assert!(!c.extends(&e));
        assert!(!e.extends(&b));
    }

    #[test]
    fn divide_integers() {
        let a: i32 = 73;
        let b: i32 = 23;
        let v = divide(a, b);
        let s: i32 = v.iter().sum();
        assert_eq!(v.len(), b as usize);
        assert_eq!(s, a);
        assert_eq!(v.iter().max(), Some(&4));
        assert_eq!(v.iter().min(), Some(&3));
    }

    #[test]
    fn point_rectangle_distance() {
        let pt1 = pt!(4, 5);
        let pt2 = pt!(1, 2);
        let pt3 = pt!(3, 8);
        let pt4 = pt!(8, 6);
        let pt5 = pt!(7, 4);
        let rect = rect![2, 3, 7, 6];
        assert_eq!(pt1.rdist2(&rect), 0);
        assert_eq!(pt2.rdist2(&rect), 2);
        assert_eq!(pt3.rdist2(&rect), 9);
        assert_eq!(pt4.rdist2(&rect), 5);
        assert_eq!(pt5.rdist2(&rect), 1);
    }
}
