//! Deterministic integer and basis points arithmetic module.
//!
//! Conforms strictly to SPEC-REQ-ARCH-002: Zero-Float Policy.
//! Eliminates floating-point discrepancies across platforms (x86_64, ARM64, WASM)
//! by using integer arithmetic with basis points representation.

use serde::{Deserialize, Serialize};

/// Basis points representation where 10_000 BPS = 100.00%.
///
/// Invariant: 1 BPS = 0.01%, 10_000 BPS = 100.00%.
/// Over-surcharges (> 100%) are supported up to `u32::MAX`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BasisPoints(pub u32);

impl BasisPoints {
    /// Zero basis points (0.00%).
    pub const ZERO: Self = Self(0);

    /// Standard 100.00% basis points representation (10_000 BPS).
    pub const STANDARD: Self = Self(10_000);

    /// Full (100.00%) basis points representation (alias for STANDARD).
    pub const FULL: Self = Self(10_000);

    /// Creates a new `BasisPoints` instance from a raw `u32` value.
    #[inline]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw integer value of basis points.
    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl Default for BasisPoints {
    #[inline]
    fn default() -> Self {
        Self::STANDARD
    }
}

impl From<u32> for BasisPoints {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<BasisPoints> for u32 {
    #[inline]
    fn from(bps: BasisPoints) -> Self {
        bps.0
    }
}

/// Errors originating from deterministic arithmetic operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathError {
    /// Denominator was zero in a division operation.
    DivisionByZero,
    /// Resulting value exceeded capacity of the target integer type.
    Overflow,
}

impl std::fmt::Display for MathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DivisionByZero => write!(f, "division by zero in deterministic math"),
            Self::Overflow => write!(f, "integer overflow in deterministic math"),
        }
    }
}

impl std::error::Error for MathError {}

/// Standard basis points factor denominator: 10_000 = 100.00%.
const BPS_DENOMINATOR: u64 = 10_000;

/// Half of basis points factor denominator for round-half-up: 5_000.
const BPS_HALF_ROUNDING: u64 = 5_000;

/// Multiplies an unsigned 32-bit integer `val` by a factor in basis points `bps`,
/// with round-half-up nearest integer rounding:
///
/// $$\text{apply\_bps}(V, B) = \left\lfloor \frac{V \times B + 5\,000}{10\,000} \right\rfloor$$
///
/// Promotes intermediate calculation to `u64` to prevent overflow on `u32` multiplicands.
/// Saturates at `u32::MAX` in case extreme `bps` multipliers exceed `u32` capacity.
#[inline]
pub fn apply_bps(val: u32, bps: BasisPoints) -> u32 {
    let product = (val as u64) * (bps.0 as u64);
    let rounded = (product + BPS_HALF_ROUNDING) / BPS_DENOMINATOR;
    if rounded > u32::MAX as u64 {
        u32::MAX
    } else {
        rounded as u32
    }
}

/// Multiplies a signed 32-bit integer `val` by a factor in basis points `bps`,
/// with round-half-away-from-zero symmetric rounding.
///
/// Promotes intermediate calculation to `u64` to prevent overflow on `i32` multiplicands,
/// including `i32::MIN`. Clamps at `i32::MIN`..=`i32::MAX`.
#[inline]
pub fn apply_bps_i32(val: i32, bps: BasisPoints) -> i32 {
    let sign: i64 = if val < 0 { -1 } else { 1 };
    let abs_val = (val as i64).unsigned_abs();
    let product = abs_val * (bps.0 as u64);
    let rounded = (product + BPS_HALF_ROUNDING) / BPS_DENOMINATOR;
    let signed_val = sign * (rounded as i64);
    if signed_val > i32::MAX as i64 {
        i32::MAX
    } else if signed_val < i32::MIN as i64 {
        i32::MIN
    } else {
        signed_val as i32
    }
}

/// Calculates the ratio of `numerator` over `denominator` expressed in basis points,
/// with round-half-up rounding:
///
/// $$\text{div\_bps}(N, D) = \left\lfloor \frac{N \times 10\,000 + (D / 2)}{D} \right\rfloor$$
///
/// Returns `Err(MathError::DivisionByZero)` if `denominator == 0`.
/// Returns `Err(MathError::Overflow)` if the result exceeds `u32::MAX`.
#[inline]
pub fn div_bps(numerator: u32, denominator: u32) -> Result<BasisPoints, MathError> {
    if denominator == 0 {
        return Err(MathError::DivisionByZero);
    }
    let num = numerator as u64;
    let den = denominator as u64;
    let raw = (num * BPS_DENOMINATOR + (den / 2)) / den;
    if raw > u32::MAX as u64 {
        return Err(MathError::Overflow);
    }
    Ok(BasisPoints(raw as u32))
}
