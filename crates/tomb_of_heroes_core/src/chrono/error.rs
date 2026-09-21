//! Chronomancy and temporal rewind domain errors.
//!
//! Conforms to `SPEC-REQ-CHRONO-002`.

use crate::chrono::journal::JournalError;
use crate::time::Tick;

/// Errors arising during chronomantic rewind operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChronoError {
    /// Target tick is prior to the oldest snapshot available in the ring buffer.
    SnapshotExpired,
    /// Available temporal mana is insufficient for the requested rewind distance.
    InsufficientMana,
    /// Attempted to rewind to a future tick beyond the current simulation tick.
    FutureTargetTick {
        /// Requested target tick.
        target: Tick,
        /// Current simulation tick.
        current: Tick,
    },
    /// No snapshots are currently available in the ring buffer.
    NoSnapshotsAvailable,
    /// Underlying action journal error during command replay or truncation.
    Journal(JournalError),
}

impl std::fmt::Display for ChronoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SnapshotExpired => {
                write!(
                    f,
                    "Requested rewind target tick has expired from the snapshot ring buffer"
                )
            }
            Self::InsufficientMana => {
                write!(f, "Insufficient temporal mana to channel rewind")
            }
            Self::FutureTargetTick { target, current } => {
                write!(
                    f,
                    "Cannot rewind forward into the future (target: {target}, current: {current})"
                )
            }
            Self::NoSnapshotsAvailable => {
                write!(f, "No snapshots available in the ring buffer")
            }
            Self::Journal(err) => write!(f, "Action journal error: {err}"),
        }
    }
}

impl std::error::Error for ChronoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Journal(err) => Some(err),
            _ => None,
        }
    }
}

impl From<JournalError> for ChronoError {
    fn from(err: JournalError) -> Self {
        Self::Journal(err)
    }
}
