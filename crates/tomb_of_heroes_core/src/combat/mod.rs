//! Deterministic combat, traps resolution, damage calculation, and death sequence.
//!
//! Enforces `SPEC-DOMAIN-COMBAT` (`docs/specs/09_combat_pieges_degats.md`).

use alloc::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::chrono::memory::ChronoHero;
use crate::config::GameConfig;
use crate::id::LogicId;
use crate::math::{apply_bps, BasisPoints};
use crate::necro::corpse::{Corpse, HeroClass};
use crate::necro::necromancy::UndeadKind;
use crate::necro::spatial::CorpseRegistry;
use crate::necro::terror::PanicLevel;
use crate::topology::coordinates::GridCoord;

pub use crate::chrono::command::TrapType;

impl TrapType {
    /// Mana investment required to construct or arm this trap.
    #[inline]
    #[must_use]
    pub const fn mana_cost(&self) -> u32 {
        match self {
            Self::Spikes => 15,
            Self::Acid => 25,
            Self::Portcullis => 20,
            Self::Darts => 10,
            Self::Blade => 15,
            Self::Pitfall => 20,
            Self::Fire => 30,
            Self::PoisonGas => 25,
        }
    }

    /// Nominal unmitigated base physical damage.
    #[inline]
    #[must_use]
    pub const fn base_damage(&self) -> u32 {
        match self {
            Self::Spikes => 35,
            Self::Acid => 15,
            Self::Portcullis => 80,
            Self::Darts => 20,
            Self::Blade => 30,
            Self::Pitfall => 40,
            Self::Fire => 50,
            Self::PoisonGas => 15,
        }
    }

    /// Resolves damage and side effects when an entity triggers the trap.
    ///
    /// Implements `SPEC-REQ-COMBAT-001`.
    #[must_use]
    pub fn resolve_damage(&self, is_fleeing: bool, target_armor: BasisPoints) -> TrapDamageResult {
        match self {
            Self::Spikes => {
                let base = if is_fleeing {
                    self.base_damage().saturating_mul(2)
                } else {
                    self.base_damage()
                };
                let effective = calculate_effective_damage(base, target_armor);
                TrapDamageResult {
                    damage: effective,
                    armor_shred_bps: BasisPoints::ZERO,
                    poison_ticks: 0,
                    poison_dmg_per_tick: 0,
                }
            }
            Self::Acid => {
                let effective = calculate_effective_damage(self.base_damage(), target_armor);
                TrapDamageResult {
                    damage: effective,
                    armor_shred_bps: BasisPoints(2000), // Permanently reduces armor by 2000 BPS
                    poison_ticks: 3,
                    poison_dmg_per_tick: 5,
                }
            }
            Self::Portcullis => {
                let effective = calculate_effective_damage(self.base_damage(), target_armor);
                TrapDamageResult {
                    damage: effective,
                    armor_shred_bps: BasisPoints::ZERO,
                    poison_ticks: 0,
                    poison_dmg_per_tick: 0,
                }
            }
            Self::Darts => {
                let effective = calculate_effective_damage(self.base_damage(), target_armor);
                TrapDamageResult {
                    damage: effective,
                    armor_shred_bps: BasisPoints::ZERO,
                    poison_ticks: 4,
                    poison_dmg_per_tick: 5,
                }
            }
            Self::Blade | Self::Pitfall | Self::Fire | Self::PoisonGas => {
                let effective = calculate_effective_damage(self.base_damage(), target_armor);
                TrapDamageResult {
                    damage: effective,
                    armor_shred_bps: BasisPoints::ZERO,
                    poison_ticks: 0,
                    poison_dmg_per_tick: 0,
                }
            }
        }
    }
}

/// Resolved consequence of a trap triggering against an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrapDamageResult {
    /// Immediate upfront damage dealt to hit points.
    pub damage: u32,
    /// Permanent reduction applied to target's armor mitigation.
    pub armor_shred_bps: BasisPoints,
    /// Duration in ticks of residual damage over time.
    pub poison_ticks: u32,
    /// Periodic damage applied per tick of residual effect.
    pub poison_dmg_per_tick: u32,
}

/// Checks whether an approaching hero identifies a concealed trap before stepping on it.
///
/// Implements `SPEC-REQ-COMBAT-001`.
#[must_use]
pub fn detect_trap(
    hero_class: HeroClass,
    is_alerted: bool,
    panic_level: PanicLevel,
    roll_bps: BasisPoints,
) -> bool {
    // Blind panic disables trap detection completely
    if matches!(panic_level, PanicLevel::BlindPanic) {
        return false;
    }

    let mut detection_bps: u32 = match hero_class {
        HeroClass::Rogue => 8000,
        _ => 2000,
    };

    if is_alerted {
        detection_bps = detection_bps.saturating_add(2000);
    }

    roll_bps.0 < detection_bps.min(10_000)
}

/// Attempts trap disarming by a qualified hero.
///
/// Implements `SPEC-REQ-COMBAT-001`.
#[must_use]
pub fn disarm_trap(hero_class: HeroClass, roll_bps: BasisPoints) -> bool {
    match hero_class {
        HeroClass::Rogue => roll_bps.0 < 7500,
        _ => false,
    }
}

/// Applies deterministic integer armor mitigation formula with round-half-up.
///
/// Formula: `max(1, ((base_damage * (10_000 - clamp(armor, 8_000)) + 5_000) / 10_000))`
///
/// Specified in `SPEC-REQ-COMBAT-002`.
#[must_use]
pub fn calculate_effective_damage(base_damage: u32, armor_reduction_bps: BasisPoints) -> u32 {
    let clamped_armor = armor_reduction_bps.0.min(8000);
    let mitigation_factor = 10_000u32.saturating_sub(clamped_armor);
    let effective = apply_bps(base_damage, BasisPoints(mitigation_factor));
    effective.max(1)
}

/// Applies +50% critical strike damage multiplier with integer round-half-up.
///
/// Specified in `SPEC-REQ-COMBAT-002`.
#[must_use]
pub fn apply_critical_hit(damage: u32, is_crit: bool) -> u32 {
    if is_crit {
        apply_bps(damage, BasisPoints(15_000))
    } else {
        damage
    }
}

/// Query combat profile of reanimated minions.
///
/// Specified in `SPEC-REQ-COMBAT-003`.
pub trait MinionCombatProfile {
    fn attack_damage(&self) -> u32;
    fn attack_cooldown_ticks(&self) -> u32;
    fn terror_on_hit_bps(&self) -> BasisPoints;
    fn ignores_target_armor(&self) -> bool;
}

impl MinionCombatProfile for UndeadKind {
    fn attack_damage(&self) -> u32 {
        match self {
            Self::SkeletonGuardian => 15,
            Self::FleshWallZombie => 25,
            Self::FallenSoulSpectre => 20,
        }
    }

    fn attack_cooldown_ticks(&self) -> u32 {
        match self {
            Self::SkeletonGuardian => 20,
            Self::FleshWallZombie => 40,
            Self::FallenSoulSpectre => 20,
        }
    }

    fn terror_on_hit_bps(&self) -> BasisPoints {
        match self {
            Self::SkeletonGuardian => BasisPoints(500),
            Self::FleshWallZombie => BasisPoints(1500),
            Self::FallenSoulSpectre => BasisPoints::ZERO,
        }
    }

    fn ignores_target_armor(&self) -> bool {
        matches!(self, Self::FallenSoulSpectre)
    }
}

/// Resulting events and state mutations following a hero's death.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeathSequenceOutcome {
    /// Logic ID assigned to newly spawned corpse.
    pub corpse_id: LogicId,
    /// Final coordinate where the corpse settled (may have slid if tile at capacity).
    pub settled_coord: GridCoord,
    /// Terror inflicted on surviving allies in line of sight.
    pub death_shock_terror_bps: BasisPoints,
    /// Whether the party leader was killed (triggering tactical retreat).
    pub leader_died: bool,
    /// Dark soul essence harvested for dungeon mana.
    pub mana_harvested: u32,
}

/// Errors occurring during combat and death resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatError {
    /// Hero targeted for death sequence was not found in active heroes registry.
    HeroNotFound,
    /// Corpse could not be placed in the spatial corpse registry.
    CorpsePlacementFailed,
}

/// Atomically resolves hero death, corpse spawning, death shock, and resource harvest.
///
/// Implements `SPEC-REQ-COMBAT-004`.
pub fn execute_death_sequence(
    hero_id: LogicId,
    heroes: &mut BTreeMap<LogicId, ChronoHero>,
    corpses: &mut CorpseRegistry,
    corpse_id: LogicId,
    config: &GameConfig,
    allies_in_fov: &[LogicId],
) -> Result<DeathSequenceOutcome, CombatError> {
    let hero = heroes.remove(&hero_id).ok_or(CombatError::HeroNotFound)?;
    let coord = hero.position.coord;
    let corpse = Corpse::new(corpse_id, hero_id, hero.hero_class, config);

    let settled_coord = corpses
        .place_corpse(corpse, coord, &config.corpse)
        .map_err(|_| CombatError::CorpsePlacementFailed)?;

    // Apply Death Shock (+2500 BPS terror) to allies in direct FOV
    const DEATH_SHOCK: BasisPoints = BasisPoints(2500);
    for ally_id in allies_in_fov {
        if let Some(ally) = heroes.get_mut(ally_id) {
            ally.apply_paradox_anxiety(DEATH_SHOCK);
            if hero.is_leader {
                ally.is_fleeing = true;
            }
        }
    }

    const MANA_HARVEST: u32 = 20;

    Ok(DeathSequenceOutcome {
        corpse_id,
        settled_coord,
        death_shock_terror_bps: DEATH_SHOCK,
        leader_died: hero.is_leader,
        mana_harvested: MANA_HARVEST,
    })
}
