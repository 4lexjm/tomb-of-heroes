//! Navigation and topology domain errors.

use core::fmt;
use serde::{Deserialize, Serialize};

/// Discrete navigation and topology errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NavigationError {
    /// Traversal attempted on a unidirectional vertical link in reverse direction.
    PassageUnidirectional,
    /// No navigable path exists between source and destination.
    NoPathFound,
    /// Coordinate is outside floor grid boundaries.
    OutOfBounds,
    /// Start coordinate is invalid or impassable.
    InvalidStartLocation,
    /// Goal coordinate is invalid or impassable.
    InvalidGoalLocation,
    /// Target tile is impassable.
    ImpassableTile,
    /// Floor ID was not found in the dungeon grid.
    FloorNotFound,
    /// Vertical link does not connect the specified coordinates.
    LinkNotFound,
}

impl fmt::Display for NavigationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PassageUnidirectional => {
                write!(
                    f,
                    "attempted traversal on unidirectional vertical link in reverse direction"
                )
            }
            Self::NoPathFound => {
                write!(f, "no navigable path exists between start and goal")
            }
            Self::OutOfBounds => {
                write!(f, "coordinate is out of grid boundaries")
            }
            Self::InvalidStartLocation => {
                write!(f, "start coordinate is invalid or impassable")
            }
            Self::InvalidGoalLocation => {
                write!(f, "goal coordinate is invalid or impassable")
            }
            Self::ImpassableTile => {
                write!(f, "target tile is impassable")
            }
            Self::FloorNotFound => {
                write!(f, "specified floor does not exist in dungeon grid")
            }
            Self::LinkNotFound => {
                write!(f, "vertical link not found at specified coordinate")
            }
        }
    }
}

impl std::error::Error for NavigationError {}
