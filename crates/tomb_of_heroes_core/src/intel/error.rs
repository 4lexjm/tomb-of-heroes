//! Domain errors for retreat, guild intelligence, and veterancy systems.
//!
//! Enforces zero unwrap/expect/panic and exhaustive error reporting.

use serde::{Deserialize, Serialize};

/// Errors originating from retreat decisions, intel processing, and veterancy management.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntelError {
    /// Hero is deceased and cannot escape or transmit intelligence to the guild.
    HeroDead,
    /// Hero has not reached a designated extraction/exit portal tile.
    NotAtExitTile,
    /// Portcullis is closed or barred, blocking traversal.
    PortcullisClosed,
    /// Dimensional anchor inhibitor field prevents magical teleportation or recall.
    TeleportationInhibited,
    /// Coordinate is invalid or not found in known topological charts.
    InvalidCoordinate,
    /// Veteran roster capacity is zero or invalid.
    RosterCapacityZero,
}

impl std::fmt::Display for IntelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HeroDead => {
                write!(f, "deceased hero cannot escape or transmit intelligence")
            }
            Self::NotAtExitTile => {
                write!(f, "hero has not reached an extraction tile")
            }
            Self::PortcullisClosed => {
                write!(f, "portcullis is closed and blocks movement")
            }
            Self::TeleportationInhibited => {
                write!(f, "dimensional anchor field inhibits teleportation")
            }
            Self::InvalidCoordinate => {
                write!(f, "invalid world coordinate")
            }
            Self::RosterCapacityZero => {
                write!(f, "guild roster capacity cannot be zero")
            }
        }
    }
}

impl std::error::Error for IntelError {}
