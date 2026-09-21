//! Discrete 2.5D coordinates and distance metrics.
//!
//! Specified in `SPEC-REQ-TOPO-001`.

use core::fmt;
use serde::{Deserialize, Serialize};

/// Discrete multi-floor level identifier.
///
/// 0 represents the upper entrance level, with increasing values descending
/// into deeper subterranean depths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FloorId(pub u8);

impl fmt::Display for FloorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Floor({})", self.0)
    }
}

impl From<u8> for FloorId {
    fn from(val: u8) -> Self {
        Self(val)
    }
}

impl From<FloorId> for u8 {
    fn from(id: FloorId) -> Self {
        id.0
    }
}

/// Signed 2D discrete grid coordinate on an individual floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GridCoord {
    /// Horizontal X coordinate (positive East, negative West).
    pub x: i32,
    /// Vertical Y coordinate (positive South/Down, negative North/Up).
    pub y: i32,
}

impl GridCoord {
    /// Zero origin coordinate (0, 0).
    pub const ZERO: Self = Self { x: 0, y: 0 };

    /// Creates a new discrete 2D grid coordinate.
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Computes orthogonal Manhattan distance: `|dx| + |dy|`.
    #[inline]
    pub fn manhattan_distance(&self, other: Self) -> u32 {
        self.x
            .abs_diff(other.x)
            .saturating_add(self.y.abs_diff(other.y))
    }

    /// Computes 8-direction Chebyshev distance: `max(|dx|, |dy|)`.
    #[inline]
    pub fn chebyshev_distance(&self, other: Self) -> u32 {
        self.x.abs_diff(other.x).max(self.y.abs_diff(other.y))
    }

    /// Computes integer squared Euclidean distance: `(dx)^2 + (dy)^2`.
    #[inline]
    pub fn dist_sq(&self, other: Self) -> u64 {
        let dx = self.x.abs_diff(other.x) as u64;
        let dy = self.y.abs_diff(other.y) as u64;
        dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy))
    }

    /// Returns the 4 orthogonal cardinal neighbors (North, East, South, West).
    pub fn neighbors_4(&self) -> [Self; 4] {
        [
            Self::new(self.x, self.y.saturating_sub(1)), // North
            Self::new(self.x.saturating_add(1), self.y), // East
            Self::new(self.x, self.y.saturating_add(1)), // South
            Self::new(self.x.saturating_sub(1), self.y), // West
        ]
    }
}

impl fmt::Display for GridCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Free function computing Manhattan distance between two grid coordinates.
#[inline]
pub fn manhattan_distance(a: GridCoord, b: GridCoord) -> u32 {
    a.manhattan_distance(b)
}

/// Free function computing Chebyshev distance between two grid coordinates.
#[inline]
pub fn chebyshev_distance(a: GridCoord, b: GridCoord) -> u32 {
    a.chebyshev_distance(b)
}

/// Free function computing integer squared Euclidean distance between two grid coordinates.
#[inline]
pub fn dist_sq(a: GridCoord, b: GridCoord) -> u64 {
    a.dist_sq(b)
}

/// Full 2.5D world coordinate identifying floor and 2D grid position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorldCoord {
    /// Floor identifier.
    pub floor: FloorId,
    /// 2D discrete grid coordinate on that floor.
    pub coord: GridCoord,
}

impl WorldCoord {
    /// Creates a new 2.5D world coordinate.
    pub const fn new(floor: FloorId, coord: GridCoord) -> Self {
        Self { floor, coord }
    }

    /// Convenience constructor from raw primitives.
    pub const fn from_raw(floor: u8, x: i32, y: i32) -> Self {
        Self {
            floor: FloorId(floor),
            coord: GridCoord { x, y },
        }
    }
}

impl fmt::Display for WorldCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Floor{}({}, {})",
            self.floor.0, self.coord.x, self.coord.y
        )
    }
}
