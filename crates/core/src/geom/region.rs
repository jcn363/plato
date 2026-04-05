use super::enums::{DiagDir, Dir};
use super::point::Point;
use super::rectangle::Rectangle;

#[derive(Debug, Copy, Clone)]
pub enum Region {
    Corner(DiagDir),
    Strip(Dir),
    Center,
}

impl Region {
    // pt ∈ rect
    // 0.0 < {corner,strip}_width < 1.0
    pub fn from_point(pt: Point, rect: Rectangle, strip_width: f32, corner_width: f32) -> Region {
        let w = rect.width() as i32;
        let h = rect.height() as i32;
        let m = w.min(h) as f32 / 2.0;

        let d = (m * corner_width).max(1.0) as i32;
        let x1 = rect.min.x + d - 1;
        let x2 = rect.max.x - d;

        // The four corners are on top of all the other regions.
        if pt.x <= x1 {
            let dx = x1 - pt.x;
            if pt.y <= rect.min.y + dx {
                return Region::Corner(DiagDir::NorthWest);
            } else if pt.y >= rect.max.y - 1 - dx {
                return Region::Corner(DiagDir::SouthWest);
            }
        } else if pt.x >= x2 {
            let dx = pt.x - x2;
            if pt.y <= rect.min.y + dx {
                return Region::Corner(DiagDir::NorthEast);
            } else if pt.y >= rect.max.y - 1 - dx {
                return Region::Corner(DiagDir::SouthEast);
            }
        }

        let d = (m * strip_width).max(1.0) as i32;
        let x1 = rect.min.x + d - 1;
        let x2 = rect.max.x - d;
        let y1 = rect.min.y + d - 1;
        let y2 = rect.max.y - d;

        // The four strips are above the center region.
        // Each of the diagonals between the strips has to belong to one of the strips.
        if pt.x <= x1 {
            let dx = pt.x - rect.min.x;
            if pt.y >= rect.min.y + dx && pt.y < rect.max.y - 1 - dx {
                return Region::Strip(Dir::West);
            }
        } else if pt.x >= x2 {
            let dx = rect.max.x - 1 - pt.x;
            if pt.y > rect.min.y + dx && pt.y <= rect.max.y - 1 - dx {
                return Region::Strip(Dir::East);
            }
        }

        if pt.y <= y1 {
            let dy = pt.y - rect.min.y;
            if pt.x > rect.min.x + dy && pt.y <= rect.max.x - 1 - dy {
                return Region::Strip(Dir::North);
            }
        } else if pt.y >= y2 {
            let dy = rect.max.y - 1 - pt.y;
            if pt.x >= rect.min.x + dy && pt.x < rect.max.x - 1 - dy {
                return Region::Strip(Dir::South);
            }
        }

        // The center rectangle is below everything else.
        Region::Center
    }
}
