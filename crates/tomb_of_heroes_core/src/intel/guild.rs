//! Guild intelligence register, map knowledge merging, and temporal dissipation.
//!
//! Conforms to `SPEC-REQ-INTEL-003`.

use alloc::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};

use crate::chrono::command::TrapType;
use crate::config::IntelConfig;
use crate::intel::error::IntelError;
use crate::intel::retreat::SquadMember;
use crate::time::Tick;
use crate::topology::coordinates::WorldCoord;

/// Freshness and reliability confidence score of guild intelligence.
///
/// Invariant: 10_000 BPS = 100.00% initial confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IntelConfidence(pub u32);

impl IntelConfidence {
    /// Nominal full confidence upon escape (10_000 BPS).
    pub const FULL: Self = Self(10_000);
    /// Zero confidence (completely expired or obsolete).
    pub const ZERO: Self = Self(0);

    /// Evaluates dissipated confidence at a given simulation tick.
    ///
    /// Conforms to `SPEC-REQ-INTEL-003`:
    /// $$\text{Confidence}(T) = \max\left(0, 10\,000 - \left\lfloor \frac{(T - T_{\text{escape}}) \times \text{dissipation\_rate\_bps}}{T_{\text{day}}} \right\rfloor\right)$$
    #[must_use]
    pub fn at_tick(&self, escape_tick: Tick, current_tick: Tick, config: &IntelConfig) -> Self {
        let elapsed_ticks = current_tick - escape_tick;
        let day_ticks = config.ticks_per_day;
        if day_ticks == 0 {
            return *self;
        }

        let loss =
            (elapsed_ticks.saturating_mul(config.dissipation_rate_per_day_bps as u64)) / day_ticks;
        let remaining = (self.0 as u64).saturating_sub(loss);
        Self(remaining as u32)
    }
}

/// Mental topological map and trap observations acquired by an adventurer.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HeroKnowledgeMap {
    /// Explored floor and grid tiles.
    pub explored_tiles: BTreeSet<WorldCoord>,
    /// Identified and cataloged traps.
    pub identified_traps: BTreeMap<WorldCoord, TrapType>,
}

impl HeroKnowledgeMap {
    /// Creates an empty mental knowledge map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an explored tile.
    pub fn explore_tile(&mut self, coord: WorldCoord) {
        self.explored_tiles.insert(coord);
    }

    /// Records an identified trap.
    pub fn identify_trap(&mut self, coord: WorldCoord, trap: TrapType) {
        self.identified_traps.insert(coord, trap);
    }
}

/// Intelligence record for an identified dungeon trap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrapIntelRecord {
    /// Recognized trap mechanism type.
    pub trap_type: TrapType,
    /// Simulation tick at which the escaping hero recorded this intel.
    pub escape_tick: Tick,
    /// Initial recorded confidence score.
    pub initial_confidence: IntelConfidence,
}

/// Guild global dungeon intelligence register.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GuildIntelRegister {
    /// Aggregated known dungeon tiles.
    pub known_tiles: BTreeSet<WorldCoord>,
    /// Identified traps mapped by location.
    pub known_traps: BTreeMap<WorldCoord, TrapIntelRecord>,
}

impl GuildIntelRegister {
    /// Creates an empty guild intelligence register.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Queries the dissipated confidence score of a known trap at `coord`.
    #[must_use]
    pub fn trap_confidence_at(
        &self,
        coord: WorldCoord,
        current_tick: Tick,
        config: &IntelConfig,
    ) -> Option<IntelConfidence> {
        self.known_traps.get(&coord).map(|record| {
            record
                .initial_confidence
                .at_tick(record.escape_tick, current_tick, config)
        })
    }

    /// Calculates false confidence trap detection modifier.
    ///
    /// If the guild believes a trap exists with confidence >= 5_000 BPS,
    /// but the actual dungeon tile now has a different trap (or is disarmed/altered),
    /// applies a -3_000 BPS penalty due to false confidence bias.
    #[must_use]
    pub fn calculate_trap_detection_penalty(
        &self,
        coord: WorldCoord,
        actual_trap: Option<TrapType>,
        current_tick: Tick,
        config: &IntelConfig,
    ) -> i32 {
        if let Some(record) = self.known_traps.get(&coord) {
            let conf = record
                .initial_confidence
                .at_tick(record.escape_tick, current_tick, config);

            if conf.0 >= config.false_confidence_threshold_bps {
                let is_altered = match actual_trap {
                    Some(trap) => trap != record.trap_type,
                    None => true,
                };
                if is_altered {
                    return -(config.false_confidence_malus_bps.0 as i32);
                }
            }
        }
        0
    }
}

/// Merges an escaped adventurer's knowledge map into the guild intel register.
///
/// Conforms to `SPEC-REQ-INTEL-003`.
pub fn merge_intel_on_escape(
    guild: &mut GuildIntelRegister,
    hero_map: &HeroKnowledgeMap,
    escape_tick: Tick,
) {
    for tile in &hero_map.explored_tiles {
        guild.known_tiles.insert(*tile);
    }
    for (coord, trap_type) in &hero_map.identified_traps {
        guild.known_traps.insert(
            *coord,
            TrapIntelRecord {
                trap_type: *trap_type,
                escape_tick,
                initial_confidence: IntelConfidence::FULL,
            },
        );
    }
}

/// Attempts to escape an adventurer and merge their intel, validating survival and extraction tile.
///
/// If the hero is dead (`current_hp == 0`), returns `Err(IntelError::HeroDead)` and transfers no intel.
/// If the hero is not on the exit tile, returns `Err(IntelError::NotAtExitTile)`.
pub fn try_escape_and_merge_intel(
    guild: &mut GuildIntelRegister,
    hero: &SquadMember,
    hero_map: &HeroKnowledgeMap,
    hero_coord: WorldCoord,
    exit_coord: WorldCoord,
    escape_tick: Tick,
) -> Result<(), IntelError> {
    if hero.current_hp == 0 {
        return Err(IntelError::HeroDead);
    }
    if hero_coord != exit_coord {
        return Err(IntelError::NotAtExitTile);
    }
    merge_intel_on_escape(guild, hero_map, escape_tick);
    Ok(())
}
