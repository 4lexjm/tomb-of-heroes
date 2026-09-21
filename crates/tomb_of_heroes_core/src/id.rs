//! Stable entity identifier and monotonic allocation module.
//!
//! Enforces `SPEC-REQ-ARCH-003` stable ID generation decoupled from host engines.

use serde::{Deserialize, Serialize};

/// Stable, deterministic identifier for core domain entities.
///
/// Specified in `SPEC-REQ-ARCH-003`.
/// Decoupled from Bevy's internal `Entity` allocation for strict determinism and replayability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LogicId(pub u64);

impl LogicId {
    /// First valid identifier allocated (`LogicId(1)`).
    pub const FIRST: Self = Self(1);

    /// Returns the inner raw `u64`.
    #[inline]
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for LogicId {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<LogicId> for u64 {
    fn from(id: LogicId) -> Self {
        id.0
    }
}

impl std::fmt::Display for LogicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LogicId({})", self.0)
    }
}

/// Errors occurring during `LogicId` allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicIdError {
    /// The 64-bit identifier space has overflowed.
    Overflow,
}

impl std::fmt::Display for LogicIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Overflow => write!(
                f,
                "LogicId allocation overflow: reached maximum u64 capacity"
            ),
        }
    }
}

impl std::error::Error for LogicIdError {}

/// Monotonic, deterministic generator for `LogicId`.
///
/// Starts at `LogicId(1)` and produces consecutive IDs:
/// id_{n+1} = id_n + 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicIdGenerator {
    next_raw: u64,
}

impl LogicIdGenerator {
    /// Creates a new generator starting at 1.
    #[must_use]
    pub const fn new() -> Self {
        Self { next_raw: 1 }
    }

    /// Creates a generator initialized to resume at the given raw ID.
    #[must_use]
    pub const fn from_raw(next_raw: u64) -> Self {
        Self { next_raw }
    }

    /// Returns the next raw ID that will be allocated.
    #[inline]
    #[must_use]
    pub const fn peek_next_raw(&self) -> u64 {
        self.next_raw
    }

    /// Allocates the next monotonic `LogicId`.
    ///
    /// Returns `Err(LogicIdError::Overflow)` without panicking if the generator is at `u64::MAX`.
    pub fn allocate(&mut self) -> Result<LogicId, LogicIdError> {
        if self.next_raw == u64::MAX {
            return Err(LogicIdError::Overflow);
        }
        let current = self.next_raw;
        self.next_raw = current.checked_add(1).ok_or(LogicIdError::Overflow)?;
        Ok(LogicId(current))
    }

    /// Alias for `allocate()`.
    #[inline]
    pub fn next_id(&mut self) -> Result<LogicId, LogicIdError> {
        self.allocate()
    }
}

impl Default for LogicIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}
