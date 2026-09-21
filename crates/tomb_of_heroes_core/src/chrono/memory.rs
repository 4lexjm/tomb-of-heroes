//! Temporal awareness, residual memory, and paradoxical cognitive adaptation.
//!
//! Conforms to `SPEC-REQ-CHRONO-003`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::id::LogicId;
use crate::math::BasisPoints;
use crate::necro::HeroClass;
use crate::topology::WorldCoord;

/// Residual temporal memory retained across timeline rewinds.
///
/// Specified in `SPEC-REQ-CHRONO-003`.
/// Preserves coordinates of lethal traps triggered or casualties observed in discarded futures.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChronoMemory {
    /// World coordinates of hazards triggered in discarded timelines.
    pub anticipated_hazards: BTreeSet<WorldCoord>,
}

impl ChronoMemory {
    /// Creates an empty residual temporal memory.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            anticipated_hazards: BTreeSet::new(),
        }
    }

    /// Records a new anticipated hazard coordinate into residual memory.
    pub fn record_hazard(&mut self, coord: WorldCoord) {
        self.anticipated_hazards.insert(coord);
    }

    /// Checks whether the given world coordinate is an anticipated hazard.
    #[must_use]
    pub fn is_hazard(&self, coord: &WorldCoord) -> bool {
        self.anticipated_hazards.contains(coord)
    }

    /// Merges another residual memory into this one.
    pub fn merge(&mut self, other: &Self) {
        self.anticipated_hazards
            .extend(other.anticipated_hazards.iter().copied());
    }

    /// Returns `true` if no anticipated hazards are tracked.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.anticipated_hazards.is_empty()
    }

    /// Returns the number of tracked anticipated hazards.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.anticipated_hazards.len()
    }

    /// Clears all recorded anticipated hazards.
    pub fn clear(&mut self) {
        self.anticipated_hazards.clear();
    }
}

/// Adventurer entity state with temporal awareness and residual memory.
///
/// Specified in `SPEC-REQ-CHRONO-003`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChronoHero {
    /// Stable entity identifier.
    pub hero_id: LogicId,
    /// Hero archetype class.
    pub hero_class: HeroClass,
    /// Flag indicating whether the hero possesses temporal rift awareness.
    pub has_chrono_awareness: bool,
    /// Residual memory retained across rewinds.
    pub chrono_memory: ChronoMemory,
    /// Current world position.
    pub position: WorldCoord,
    /// Psychological terror level in basis points (0 to 10_000).
    pub terror_bps: BasisPoints,
    /// Preemptive magical shield activated in response to anticipated hazards.
    pub has_preemptive_shield: bool,
}

impl ChronoHero {
    /// Creates a new hero entity.
    #[must_use]
    pub const fn new(
        hero_id: LogicId,
        hero_class: HeroClass,
        has_chrono_awareness: bool,
        position: WorldCoord,
    ) -> Self {
        Self {
            hero_id,
            hero_class,
            has_chrono_awareness,
            chrono_memory: ChronoMemory::new(),
            position,
            terror_bps: BasisPoints::ZERO,
            has_preemptive_shield: false,
        }
    }

    /// Creates a chrono-aware hero entity.
    #[must_use]
    pub const fn new_aware(hero_id: LogicId, hero_class: HeroClass, position: WorldCoord) -> Self {
        Self::new(hero_id, hero_class, true, position)
    }

    /// Creates an ordinary (non-aware) hero entity.
    #[must_use]
    pub const fn new_ordinary(
        hero_id: LogicId,
        hero_class: HeroClass,
        position: WorldCoord,
    ) -> Self {
        Self::new(hero_id, hero_class, false, position)
    }

    /// Records an anticipated hazard into the hero's residual memory.
    pub fn record_hazard(&mut self, coord: WorldCoord) {
        self.chrono_memory.record_hazard(coord);
    }

    /// Returns `true` if the hero refuses or actively avoids traversing the given tile.
    #[must_use]
    pub fn avoids_tile(&self, coord: &WorldCoord) -> bool {
        self.has_chrono_awareness && self.chrono_memory.is_hazard(coord)
    }

    /// Inflicts paradoxical terror anxiety onto the hero, clamped at 10_000 BPS.
    pub fn apply_paradox_anxiety(&mut self, anxiety: BasisPoints) {
        let current = self.terror_bps.0;
        let sum = current.saturating_add(anxiety.0);
        let clamped = if sum > 10_000 { 10_000 } else { sum };
        self.terror_bps = BasisPoints(clamped);
    }

    /// Activates a preemptive defensive shield.
    pub fn activate_preemptive_shield(&mut self) {
        self.has_preemptive_shield = true;
    }

    /// Checks whether the preemptive shield is currently raised.
    #[must_use]
    pub const fn is_shield_active(&self) -> bool {
        self.has_preemptive_shield
    }
}
