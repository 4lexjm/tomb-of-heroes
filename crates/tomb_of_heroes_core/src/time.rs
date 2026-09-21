//! Discrete logical simulation clock and tick types.
//!
//! Enforces `SPEC-REQ-ARCH-001` fixed-tick progression (20 Hz nominal).

use serde::{Deserialize, Serialize};

/// Discrete logical simulation tick counter.
///
/// Specified in `SPEC-REQ-ARCH-001`.
/// The tick increments monotonically by 1 each discrete step (20 Hz, 50 ms).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tick(pub u64);

impl Tick {
    /// Initial tick constant `Tick(0)`.
    pub const ZERO: Self = Self(0);

    /// Returns the tick as a raw `u64`.
    #[inline]
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Computes the subsequent monotonic tick.
    #[inline]
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Advances this tick by 1 in-place.
    #[inline]
    pub fn advance(&mut self) {
        self.0 = self.0.saturating_add(1);
    }
}

impl Default for Tick {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<u64> for Tick {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<Tick> for u64 {
    fn from(tick: Tick) -> Self {
        tick.0
    }
}

impl std::fmt::Display for Tick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tick({})", self.0)
    }
}

impl std::ops::Add<u64> for Tick {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0.saturating_add(rhs))
    }
}

impl std::ops::AddAssign<u64> for Tick {
    fn add_assign(&mut self, rhs: u64) {
        self.0 = self.0.saturating_add(rhs);
    }
}

impl std::ops::Sub for Tick {
    type Output = u64;
    fn sub(self, rhs: Self) -> Self::Output {
        self.0.saturating_sub(rhs.0)
    }
}

/// Headless simulation logical clock.
///
/// Encapsulates the discrete simulation clock advancing monotonically tick by tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LogicClock {
    current_tick: Tick,
}

impl LogicClock {
    /// Creates a new logical clock initialized to `Tick(0)`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current_tick: Tick::ZERO,
        }
    }

    /// Creates a logical clock initialized to a specific tick.
    #[must_use]
    pub const fn from_tick(tick: Tick) -> Self {
        Self { current_tick: tick }
    }

    /// Returns the current tick.
    #[inline]
    #[must_use]
    pub const fn current_tick(&self) -> Tick {
        self.current_tick
    }

    /// Advances the clock by 1 tick and returns the new tick.
    pub fn step(&mut self) -> Tick {
        self.current_tick.advance();
        self.current_tick
    }
}
