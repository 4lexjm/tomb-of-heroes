//! Adventurer veterancy, trauma traits, and guild roster eviction policy.
//!
//! Conforms to `SPEC-REQ-INTEL-004`.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::chrono::command::TrapType;
use crate::config::IntelConfig;
use crate::id::LogicId;
use crate::math::{div_bps, BasisPoints};
use crate::necro::HeroClass;
use crate::time::Tick;
use crate::topology::coordinates::WorldCoord;

/// Psychological trauma traits acquired by surviving adventurers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TraumaTrait {
    /// Terror doubled against incendiary traps.
    Pyrophobia,
    /// Trap detection speed increased by 4_000 BPS (+40%).
    TrapParanoia,
    /// +2_500 BPS (+25%) damage against skeletons and zombies.
    UndeadSlayer,
    /// +3_000 BPS (+30%) retreat threshold resistance.
    VengefulTenacity,
}

/// Statistics from an adventurer's incursion used for trait evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HeroRunStats {
    /// Lowest hit points reached during run, in basis points of max HP.
    pub min_hp_bps: BasisPoints,
    /// Whether the hero suffered fire/flame damage.
    pub suffered_fire_damage: bool,
    /// Count of mechanical traps disarmed or survived.
    pub mechanical_traps_survived: u32,
    /// Count of undead slain during the incursion.
    pub undead_killed: u32,
    /// Consecutive successful incursions survived.
    pub consecutive_survivals: u32,
}

/// Generates psychological trauma traits based on run statistics.
///
/// Conforms to `SPEC-REQ-INTEL-004`:
/// - `Pyrophobia`: suffered fire damage and HP dropped < 1_000 BPS (10.00%).
/// - `TrapParanoia`: survived > 3 mechanical traps.
/// - `UndeadSlayer`: killed at least one undead creature.
/// - `VengefulTenacity`: survived at least 2 consecutive incursions.
#[must_use]
pub fn generate_trauma_traits(stats: &HeroRunStats, config: &IntelConfig) -> Vec<TraumaTrait> {
    let mut traits = Vec::new();

    if stats.suffered_fire_damage && stats.min_hp_bps < config.pyrophobia_hp_threshold_bps {
        traits.push(TraumaTrait::Pyrophobia);
    }
    if stats.mechanical_traps_survived > config.trap_paranoia_threshold {
        traits.push(TraumaTrait::TrapParanoia);
    }
    if stats.undead_killed > 0 {
        traits.push(TraumaTrait::UndeadSlayer);
    }
    if stats.consecutive_survivals >= 2 {
        traits.push(TraumaTrait::VengefulTenacity);
    }

    traits
}

/// Persistent veteran profile of an escaped hero.
///
/// Conforms to `SPEC-REQ-INTEL-004`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VeteranProfile {
    /// Stable veteran identifier.
    pub veteran_id: LogicId,
    /// Hash of hero's name.
    pub name_hash: u64,
    /// Hero archetype class.
    pub hero_class: HeroClass,
    /// Progression rank (1 to 5).
    pub rank: u8,
    /// Total incursions survived.
    pub survived_incursions_count: u32,
    /// Known traps cataloged from dungeon runs.
    pub known_keep_traps: Vec<TrapType>,
    /// Acquired psychological trauma traits.
    pub trauma_traits: Vec<TraumaTrait>,
    /// Specific vengeance target location in the dungeon.
    pub vengeance_target_zone: Option<WorldCoord>,
    /// Total incursions attempted (for survival ratio calculation).
    pub total_incursions_count: u32,
    /// Simulation tick of the last incursion.
    pub last_incursion_tick: Tick,
}

impl VeteranProfile {
    /// Creates a new veteran profile.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        veteran_id: LogicId,
        name_hash: u64,
        hero_class: HeroClass,
        rank: u8,
        survived_incursions_count: u32,
        known_keep_traps: Vec<TrapType>,
        trauma_traits: Vec<TraumaTrait>,
        vengeance_target_zone: Option<WorldCoord>,
    ) -> Self {
        Self {
            veteran_id,
            name_hash,
            hero_class,
            rank,
            survived_incursions_count,
            known_keep_traps,
            trauma_traits,
            vengeance_target_zone,
            total_incursions_count: survived_incursions_count,
            last_incursion_tick: Tick::ZERO,
        }
    }

    /// Sets extended incursion statistics for survival ratio and eviction evaluation.
    #[must_use]
    pub fn with_stats(mut self, total_incursions: u32, last_incursion_tick: Tick) -> Self {
        self.total_incursions_count = total_incursions;
        self.last_incursion_tick = last_incursion_tick;
        self
    }

    /// Computes the survival ratio in basis points.
    #[must_use]
    pub fn survival_ratio_bps(&self) -> BasisPoints {
        if self.total_incursions_count == 0 {
            return BasisPoints::ZERO;
        }
        match div_bps(self.survived_incursions_count, self.total_incursions_count) {
            Ok(bps) => bps,
            Err(_) => BasisPoints::ZERO,
        }
    }
}

/// Result of attempting to register a veteran into the guild roster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RosterOutcome {
    /// Successfully registered within capacity.
    Registered,
    /// Registered, evicting an existing veteran due to capacity overflow.
    RegisteredWithEviction {
        /// Identifier of the evicted veteran.
        evicted_id: LogicId,
    },
    /// Veteran reached Rank 5 and retired honorably to the royal court.
    HonorablyRetired,
}

/// Guild veteran roster with bounded capacity and eviction policy.
///
/// Conforms to `SPEC-REQ-INTEL-004`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuildRoster {
    /// Registered veteran profiles.
    pub profiles: Vec<VeteranProfile>,
    /// Maximum roster capacity (nominal 32).
    pub max_capacity: usize,
}

impl GuildRoster {
    /// Creates an empty guild roster with maximum capacity.
    #[must_use]
    pub fn new(max_capacity: usize) -> Self {
        Self {
            profiles: Vec::new(),
            max_capacity,
        }
    }

    /// Creates an empty guild roster configured via [`IntelConfig`].
    #[must_use]
    pub fn with_config(config: &IntelConfig) -> Self {
        Self::new(config.max_roster_capacity)
    }

    /// Returns the number of veterans in the roster.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    /// Returns `true` if the roster has no veterans.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    /// Registers a veteran profile according to rank and eviction rules:
    /// 1. If rank >= `config.honorable_retirement_rank` (Rank 5), retires honorably.
    /// 2. If below capacity, registers directly.
    /// 3. If at capacity, evicts the veteran with lowest survival ratio and oldest run.
    pub fn register(&mut self, profile: VeteranProfile, config: &IntelConfig) -> RosterOutcome {
        // Rule 1: Honorable retirement at Rank 5
        if profile.rank >= config.honorable_retirement_rank {
            return RosterOutcome::HonorablyRetired;
        }

        // Rule 2: Capacity available
        if self.profiles.len() < self.max_capacity {
            self.profiles.push(profile);
            return RosterOutcome::Registered;
        }

        // Rule 3: Overflow eviction
        // Find index of worst candidate (lowest survival ratio, then oldest incursion tick, then lowest ID)
        let mut evict_idx = 0;
        let mut lowest_ratio = self.profiles[0].survival_ratio_bps();
        let mut oldest_tick = self.profiles[0].last_incursion_tick;
        let mut lowest_id = self.profiles[0].veteran_id;

        for (idx, candidate) in self.profiles.iter().enumerate().skip(1) {
            let candidate_ratio = candidate.survival_ratio_bps();
            let candidate_tick = candidate.last_incursion_tick;
            let candidate_id = candidate.veteran_id;

            let is_worse = if candidate_ratio < lowest_ratio {
                true
            } else if candidate_ratio == lowest_ratio {
                if candidate_tick < oldest_tick {
                    true
                } else if candidate_tick == oldest_tick {
                    candidate_id < lowest_id
                } else {
                    false
                }
            } else {
                false
            };

            if is_worse {
                evict_idx = idx;
                lowest_ratio = candidate_ratio;
                oldest_tick = candidate_tick;
                lowest_id = candidate_id;
            }
        }

        let evicted_veteran = self.profiles.remove(evict_idx);
        self.profiles.push(profile);

        RosterOutcome::RegisteredWithEviction {
            evicted_id: evicted_veteran.veteran_id,
        }
    }
}
