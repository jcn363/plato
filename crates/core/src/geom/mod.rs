#[cfg(test)]
mod geom_tests;

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

impl From<PrivBoundary> for PrivRectangle {
    fn from(val: PrivBoundary) -> Self {
        PrivRectangle {
            min: PrivPoint::new(val.min.x.floor() as i32, val.min.y.floor() as i32),
            max: PrivPoint::new(val.max.x.ceil() as i32, val.max.y.ceil() as i32),
        }
    }
}

impl From<PrivRectangle> for PrivBoundary {
    fn from(val: PrivRectangle) -> Self {
        PrivBoundary {
            min: PrivVec2::new(val.min.x as f32, val.min.y as f32),
            max: PrivVec2::new(val.max.x as f32, val.max.y as f32),
        }
    }
}
