//! Cryptographic state hash and deterministic hashing module.
//!
//! Enforces `SPEC-REQ-ARCH-005` state footprint hashing for divergence detection.

use serde::{Deserialize, Serialize};

/// Cryptographic / deterministic 64-bit state signature.
///
/// Specified in `SPEC-REQ-ARCH-005`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct StateHash(pub u64);

impl StateHash {
    /// Zero state hash constant.
    pub const ZERO: Self = Self(0);

    /// Returns the inner raw `u64`.
    #[inline]
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for StateHash {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<StateHash> for u64 {
    fn from(hash: StateHash) -> Self {
        hash.0
    }
}

impl std::fmt::Display for StateHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StateHash({:#018x})", self.0)
    }
}

impl std::fmt::LowerHex for StateHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::LowerHex::fmt(&self.0, f)
    }
}

impl std::fmt::UpperHex for StateHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::UpperHex::fmt(&self.0, f)
    }
}

/// Deterministic 64-bit FNV-1a state hasher.
///
/// Ensures 100% platform-independent, reproducible hash calculation.
#[derive(Debug, Clone)]
pub struct StateHasher {
    hash: u64,
}

impl StateHasher {
    const FNV_OFFSET_BASIS: u64 = 0xCBF2_9CE4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;

    /// Creates a new state hasher initialized with the standard FNV-1a offset basis.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            hash: Self::FNV_OFFSET_BASIS,
        }
    }

    /// Feeds raw bytes into the hasher.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.hash ^= byte as u64;
            self.hash = self.hash.wrapping_mul(Self::FNV_PRIME);
        }
    }

    /// Feeds a `u64` in little-endian byte order.
    pub fn write_u64(&mut self, val: u64) {
        self.write_bytes(&val.to_le_bytes());
    }

    /// Feeds a `u32` in little-endian byte order.
    pub fn write_u32(&mut self, val: u32) {
        self.write_bytes(&val.to_le_bytes());
    }

    /// Finalizes and returns the computed `StateHash`.
    #[must_use]
    pub const fn finish(&self) -> StateHash {
        StateHash(self.hash)
    }
}

impl Default for StateHasher {
    fn default() -> Self {
        Self::new()
    }
}
