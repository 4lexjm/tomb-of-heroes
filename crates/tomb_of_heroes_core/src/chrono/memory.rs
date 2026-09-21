//! Temporal awareness, residual memory, and paradoxical cognitive adaptation.
//!
//! Conforms to `SPEC-REQ-CHRONO-003`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::id::LogicId;
use crate::intel::veterancy::TraumaTrait;
use crate::math::BasisPoints;
use crate::necro::terror::Bravery;
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

/// Behavioral state of an active adventurer hero in the dungeon.
///
/// Specified in `SPEC-REQ-SIM-002`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HeroState {
    /// Nominal incursion exploring towards objective (stairs or heart).
    Infiltrating,
    /// Heightened vigilance due to detected traps or nearby corpses.
    Alerted,
    /// Active physical combat with a defender minion.
    Engaged,
    /// Uncontrolled or tactical rout retreating towards surface exit.
    Fleeing,
    /// Hero has perished and undergone corpse transformation.
    Dead,
    /// Hero has successfully exited the dungeon with knowledge.
    Escaped,
}

/// Adventurer entity state with temporal awareness and residual memory.
///
/// Specified in `SPEC-REQ-CHRONO-003` and `SPEC-REQ-SIM-002`.
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
    /// Current remaining hit points.
    pub current_hp: u32,
    /// Nominal maximum hit points.
    pub max_hp: u32,
    /// Armor physical damage mitigation in basis points (e.g. 3000 = 30%).
    pub armor_bps: BasisPoints,
    /// Flag indicating whether this hero is the party leader.
    pub is_leader: bool,
    /// Flag indicating active vigilance / alertness (+2000 BPS trap detection).
    pub is_alerted: bool,
    /// Flag indicating active tactical or blind panic retreat towards exit.
    pub is_fleeing: bool,
    /// Current FSM behavioral state.
    pub state: HeroState,
    /// Planned navigation trajectory on the discrete navmesh.
    pub path: Vec<WorldCoord>,
    /// Simulation tick timestamp when the next step may occur.
    pub next_move_tick: u64,
    /// Ticks elapsed since last A* path recalculation.
    pub ticks_since_repath: u32,
    /// Current navigation goal coordinate.
    pub target_destination: Option<WorldCoord>,
    /// Progression rank (1 to 5).
    pub rank: u8,
    /// Acquired psychological trauma traits.
    pub trauma_traits: Vec<TraumaTrait>,
    /// Flag indicating whether this hero is a guild expedition boss.
    pub is_boss: bool,
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
        let (base_hp, base_armor) = match hero_class {
            HeroClass::Warrior => (100, BasisPoints(3000)),
            HeroClass::Cleric => (80, BasisPoints(2000)),
            HeroClass::Paladin => (120, BasisPoints(4000)),
            HeroClass::Mage => (60, BasisPoints(1000)),
            HeroClass::Rogue => (70, BasisPoints(1500)),
        };
        Self {
            hero_id,
            hero_class,
            has_chrono_awareness,
            chrono_memory: ChronoMemory::new(),
            position,
            terror_bps: BasisPoints::ZERO,
            has_preemptive_shield: false,
            current_hp: base_hp,
            max_hp: base_hp,
            armor_bps: base_armor,
            is_leader: false,
            is_alerted: false,
            is_fleeing: false,
            state: HeroState::Infiltrating,
            path: Vec::new(),
            next_move_tick: 0,
            ticks_since_repath: 0,
            target_destination: None,
            rank: 1,
            trauma_traits: Vec::new(),
            is_boss: false,
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

    /// Returns the discrete tick interval duration required between movements.
    ///
    /// Implements `SPEC-REQ-SIM-002`.
    #[must_use]
    pub fn move_cooldown_ticks(&self) -> u64 {
        let base_ticks: u64 = match self.hero_class {
            HeroClass::Rogue => 6,
            HeroClass::Warrior | HeroClass::Cleric => 10,
            HeroClass::Mage | HeroClass::Paladin => 12,
        };
        if self.is_alerted || matches!(self.state, HeroState::Alerted) {
            (base_ticks * 15 + 5) / 10
        } else if self.is_fleeing || matches!(self.state, HeroState::Fleeing) {
            ((base_ticks * 10 + 7) / 15).max(3)
        } else {
            base_ticks
        }
    }

    /// Returns the courage fortitude level of this hero archetype.
    #[must_use]
    pub fn bravery(&self) -> Bravery {
        if self.is_boss {
            return Bravery::ABSOLUTE;
        }
        match self.hero_class {
            HeroClass::Rogue | HeroClass::Mage => Bravery::NOVICE,
            HeroClass::Warrior | HeroClass::Cleric => Bravery::HARDENED,
            HeroClass::Paladin => Bravery::INQUISITOR,
        }
    }
}
