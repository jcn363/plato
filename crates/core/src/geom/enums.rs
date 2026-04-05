use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Dir {
    North,
    East,
    South,
    West,
}

impl Dir {
    #[inline]
    pub fn opposite(self) -> Dir {
        match self {
            Dir::North => Dir::South,
            Dir::South => Dir::North,
            Dir::East => Dir::West,
            Dir::West => Dir::East,
        }
    }

    #[inline]
    pub fn axis(self) -> Axis {
        match self {
            Dir::North | Dir::South => Axis::Vertical,
            Dir::East | Dir::West => Axis::Horizontal,
        }
    }
}

impl fmt::Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Dir::North => write!(f, "north"),
            Dir::East => write!(f, "east"),
            Dir::South => write!(f, "south"),
            Dir::West => write!(f, "west"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DiagDir {
    NorthWest,
    NorthEast,
    SouthEast,
    SouthWest,
}

impl DiagDir {
    pub fn opposite(self) -> DiagDir {
        match self {
            DiagDir::NorthWest => DiagDir::SouthEast,
            DiagDir::NorthEast => DiagDir::SouthWest,
            DiagDir::SouthEast => DiagDir::NorthWest,
            DiagDir::SouthWest => DiagDir::NorthEast,
        }
    }
}

impl fmt::Display for DiagDir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiagDir::NorthWest => write!(f, "northwest"),
            DiagDir::NorthEast => write!(f, "northeast"),
            DiagDir::SouthEast => write!(f, "southeast"),
            DiagDir::SouthWest => write!(f, "southwest"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
    Diagonal,
}

impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Axis::Horizontal => write!(f, "horizontal"),
            Axis::Vertical => write!(f, "vertical"),
            Axis::Diagonal => write!(f, "diagonal"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CycleDir {
    Next,
    Previous,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum LinearDir {
    Backward,
    Forward,
}

impl LinearDir {
    pub fn opposite(self) -> LinearDir {
        match self {
            LinearDir::Backward => LinearDir::Forward,
            LinearDir::Forward => LinearDir::Backward,
        }
    }
}
