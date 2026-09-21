//! Psychological terror, bravery attributes, and panic level dynamics.
//!
//! Conforms to `SPEC-REQ-NECRO-002`.

use serde::{Deserialize, Serialize};

use crate::config::{GameConfig, TerrorConfig};
use crate::math::{apply_bps, BasisPoints};

/// Bravery attribute representing an adventurer's mental fortitude.
///
/// Invariant: 10_000 BPS (100.00%) = Absolute Bravery / Complete Terror Immunity.
/// Typical values: Novice = 2_000 BPS (20%), Veteran Warrior = 6_500 BPS (65%), Inquisitor = 9_500 BPS (95%).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Bravery(pub BasisPoints);

impl Bravery {
    /// Absolute courage yielding total immunity to psychological dread.
    pub const ABSOLUTE: Self = Self(BasisPoints::STANDARD);

    /// Baseline novice courage (20.00% = 2_000 BPS).
    pub const NOVICE: Self = Self(BasisPoints(2_000));

    /// Hardened warrior courage (65.00% = 6_500 BPS).
    pub const HARDENED: Self = Self(BasisPoints(6_500));

    /// Veteran inquisitor courage (95.00% = 9_500 BPS).
    pub const INQUISITOR: Self = Self(BasisPoints(9_500));
}

/// Cumulative terror points gauge for an adventurer.
///
/// Plafond: strictly clamped at 10_000 BPS (`config.terror.max_points`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TerrorPoints(pub u32);

impl TerrorPoints {
    /// Zero terror (completely calm).
    pub const ZERO: Self = Self(0);

    /// Accumulates terror delta while strictly clamping to [`TerrorConfig::max_points`].
    pub fn accumulate(&mut self, delta: u32, config: &TerrorConfig) {
        self.0 = (self.0.saturating_add(delta)).min(config.max_points);
    }

    /// Evaluates current behavioral panic level against [`TerrorConfig`].
    #[inline]
    #[must_use]
    pub fn panic_level(&self, config: &TerrorConfig) -> PanicLevel {
        PanicLevel::from_points(self.0, config)
    }

    /// Returns `true` if the terror gauge has reached its maximum capacity.
    #[inline]
    #[must_use]
    pub const fn is_max(&self, config: &TerrorConfig) -> bool {
        self.0 >= config.max_points
    }
}

/// Four discrete psychological stages of fear.
///
/// Specified in `SPEC-REQ-NECRO-002`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PanicLevel {
    /// Terror < 3_000: Standard tactical behavior.
    Serene,
    /// 3_000 <= Terror < 6_000: Accuracy/Armor -15%, Speed -10%.
    Shaken,
    /// 6_000 <= Terror < 8_500: Incapable of complex casting, prioritizes retreat.
    Disrupted,
    /// Terror >= 8_500: Complete rout, uncontrollable flee opposite to terror source.
    BlindPanic,
}

impl PanicLevel {
    /// Evaluates psychological panic level for a given terror score.
    #[must_use]
    pub fn from_points(points: u32, config: &TerrorConfig) -> Self {
        if points >= config.blind_panic_threshold {
            Self::BlindPanic
        } else if points >= config.disrupted_threshold {
            Self::Disrupted
        } else if points >= config.shaken_threshold {
            Self::Shaken
        } else {
            Self::Serene
        }
    }

    /// Returns the accuracy and armor malus in basis points associated with this state.
    #[must_use]
    pub const fn penalty_accuracy_and_armor_bps(&self) -> Option<BasisPoints> {
        match self {
            Self::Shaken | Self::Disrupted => Some(BasisPoints(1_500)),
            Self::BlindPanic => Some(BasisPoints(3_000)),
            Self::Serene => None,
        }
    }

    /// Returns the movement speed reduction in basis points associated with this state.
    #[must_use]
    pub const fn penalty_speed_bps(&self) -> Option<BasisPoints> {
        match self {
            Self::Shaken | Self::Disrupted => Some(BasisPoints(1_000)),
            Self::BlindPanic => Some(BasisPoints(2_000)),
            Self::Serene => None,
        }
    }

    /// Returns `true` if the unit is cognitively capable of complex spellcasting.
    #[inline]
    #[must_use]
    pub const fn can_cast_complex_spells(&self) -> bool {
        matches!(self, Self::Serene | Self::Shaken)
    }

    /// Returns `true` if the unit has broken formation and is routing.
    #[inline]
    #[must_use]
    pub const fn is_routing(&self) -> bool {
        matches!(self, Self::BlindPanic)
    }
}

/// Computes per-tick terror accumulation projected onto an observer.
///
/// Conforms to the exact mathematical formula in `SPEC-REQ-NECRO-002`:
///
/// $$\Delta \text{Terror} = \text{apply\_bps}\left(\left\lfloor \frac{\text{base\_terror\_potency} \times (6 - \min(D, 5))}{5 \times 20} \right\rfloor, 10\,000 - \text{Bravery}\right)$$
#[must_use]
pub fn compute_terror_accumulation(distance: u32, potency: BasisPoints, bravery: Bravery) -> u32 {
    let max_dist = 5u32;
    let tick_rate = 20u32;
    let dist_clamped = distance.min(max_dist);
    let dist_factor = (max_dist + 1).saturating_sub(dist_clamped);
    let denom = max_dist * tick_rate;
    let base = (potency.0 * dist_factor) / denom;
    let vuln = 10_000u32.saturating_sub(bravery.0 .0);
    apply_bps(base, BasisPoints(vuln))
}

/// Computes per-tick terror accumulation configured via [`GameConfig`].
#[must_use]
pub fn compute_terror_accumulation_with_config(
    distance: u32,
    potency: BasisPoints,
    bravery: Bravery,
    config: &GameConfig,
) -> u32 {
    let max_dist = config.terror.max_projection_distance;
    let tick_rate = config.tick.rate_hz;
    let dist_clamped = distance.min(max_dist);
    let dist_factor = (max_dist + 1).saturating_sub(dist_clamped);
    let denom = max_dist.saturating_mul(tick_rate);
    if denom == 0 {
        return 0;
    }
    let base = (potency.0 * dist_factor) / denom;
    let vuln = config.standard_bps.0.saturating_sub(bravery.0 .0);
    apply_bps(base, BasisPoints(vuln))
}
