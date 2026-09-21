//! Threat Director, wave compositions, and tri-phase campaign loop.
//!
//! Conforms to `SPEC-REQ-WAVE-001`, `SPEC-REQ-WAVE-002`, and `SPEC-REQ-WAVE-004`
//! (`docs/specs/11_vagues_campagne_victoire.md`).

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::campaign::stats::CampaignStats;
use crate::chrono::memory::{ChronoHero, HeroState};
use crate::id::{LogicId, LogicIdGenerator};
use crate::intel::veterancy::VeteranProfile;
use crate::math::BasisPoints;
use crate::necro::HeroClass;
use crate::topology::coordinates::WorldCoord;

/// Operational phase within an incursion cycle.
///
/// Conforms to `SPEC-REQ-WAVE-001`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WavePhase {
    /// Phase 1: Free or timed preparation. Player fortifies dungeon, arms traps, and raises minions.
    Preparation,
    /// Phase 2: Hostile adventurers incursion exploring and assaulting the dungeon.
    Incursion,
    /// Phase 3: Post-assault assessment, infamy calculation, and guild intel update.
    Debriefing,
    /// Campaign Victory: Guild Master / Wave 5 defeated or Infamy >= 1000.
    Victory,
    /// Campaign Defeat: Dungeon Heart PV <= 0.
    Defeat,
}

/// Deterministic Threat Director governing wave progression, squad generation, and victory/defeat.
///
/// Conforms to `SPEC-REQ-WAVE-001`, `SPEC-REQ-WAVE-002`, `SPEC-REQ-WAVE-004`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreatDirector {
    /// Current wave index (1 to 5).
    pub current_wave: u32,
    /// Maximum campaign wave count (nominal 5).
    pub max_waves: u32,
    /// Current operational phase.
    pub phase: WavePhase,
    /// Accumulated campaign infamy score.
    pub infamy: u32,
    /// Aggregated campaign performance statistics.
    pub stats: CampaignStats,
    /// Survived veterans from past incursions awaiting revenge in future waves.
    pub escaped_veterans: Vec<VeteranProfile>,
    /// Remaining preparation ticks before incursion auto-triggers (if timed, nominal 600 ticks).
    pub preparation_ticks_remaining: Option<u64>,
    /// Number of heroes spawned in the current active incursion.
    pub active_incursion_heroes_count: u32,
}

impl ThreatDirector {
    /// Nominal default campaign wave limit.
    pub const DEFAULT_MAX_WAVES: u32 = 5;
    /// Default preparation duration in ticks (600 ticks = 30 seconds at 20 Hz).
    pub const DEFAULT_PREPARATION_TICKS: u64 = 600;
    /// Infamy threshold for immediate campaign victory (1000 points).
    pub const VICTORY_INFAMY_THRESHOLD: u32 = 1_000;

    /// Creates a new Threat Director initialized at Wave 1 Preparation.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_wave: 1,
            max_waves: Self::DEFAULT_MAX_WAVES,
            phase: WavePhase::Preparation,
            infamy: 0,
            stats: CampaignStats::new(),
            escaped_veterans: Vec::new(),
            preparation_ticks_remaining: Some(Self::DEFAULT_PREPARATION_TICKS),
            active_incursion_heroes_count: 0,
        }
    }

    /// Generates an expedition squad conforming to `SPEC-REQ-WAVE-002`.
    ///
    /// - Wave 1: 3 heroes (Warrior, Rogue, Cleric), Rank 1.
    /// - Wave 2: 4 heroes (Warrior, Rogue, Cleric, Mage), Rank 2.
    /// - Wave 3: 4 heroes (Paladin, Rogue, Cleric, Mage), Rank 3 (integrates escaped veterans).
    /// - Wave 4: 5 heroes (2 Warriors, 1 Rogue, 1 Cleric, 1 Mage), Rank 4.
    /// - Wave 5: 6 heroes (Boss: Grand Master + Elite: 1 Paladin, 1 Mage, 1 Rogue, 1 Cleric, 2 Warriors), Rank 5.
    pub fn generate_wave_squad(
        &mut self,
        id_generator: &mut LogicIdGenerator,
        spawn_coord: WorldCoord,
        target_destination: WorldCoord,
    ) -> Vec<ChronoHero> {
        let wave = self.current_wave;
        let mut squad = Vec::new();

        let class_templates: Vec<HeroClass> = match wave {
            1 => alloc::vec![HeroClass::Warrior, HeroClass::Rogue, HeroClass::Cleric],
            2 => alloc::vec![
                HeroClass::Warrior,
                HeroClass::Rogue,
                HeroClass::Cleric,
                HeroClass::Mage
            ],
            3 => alloc::vec![
                HeroClass::Paladin,
                HeroClass::Rogue,
                HeroClass::Cleric,
                HeroClass::Mage
            ],
            4 => alloc::vec![
                HeroClass::Warrior,
                HeroClass::Warrior,
                HeroClass::Rogue,
                HeroClass::Cleric,
                HeroClass::Mage
            ],
            _ => alloc::vec![
                HeroClass::Paladin, // Grand Master Boss slot
                HeroClass::Warrior,
                HeroClass::Warrior,
                HeroClass::Rogue,
                HeroClass::Cleric,
                HeroClass::Mage
            ],
        };

        let rank: u8 = (wave as u8).clamp(1, 5);

        for (idx, &class) in class_templates.iter().enumerate() {
            // Allocate stable ID or reuse veteran ID
            let is_first = idx == 0;
            let hero_id = if is_first && wave >= 3 && !self.escaped_veterans.is_empty() {
                self.escaped_veterans[0].veteran_id
            } else {
                match id_generator.allocate() {
                    Ok(id) => id,
                    Err(_) => LogicId(1_000_000 + (wave as u64 * 100) + idx as u64),
                }
            };

            let mut hero = ChronoHero::new_ordinary(hero_id, class, spawn_coord);
            hero.target_destination = Some(target_destination);
            hero.state = HeroState::Infiltrating;
            hero.rank = rank;

            // Apply rank progression stat multipliers
            // Rank 2: +15% HP, +500 BPS armor
            // Rank 3: +30% HP, +1000 BPS armor
            // Rank 4: +45% HP, +1500 BPS armor
            // Rank 5: +60% HP, +2000 BPS armor
            let hp_bonus_pct = (rank.saturating_sub(1) as u32) * 15;
            let armor_bonus_bps = (rank.saturating_sub(1) as u32) * 500;
            let scaled_hp = hero
                .max_hp
                .saturating_add((hero.max_hp * hp_bonus_pct) / 100);
            hero.max_hp = scaled_hp;
            hero.current_hp = scaled_hp;
            hero.armor_bps = BasisPoints(hero.armor_bps.0.saturating_add(armor_bonus_bps));

            // Integrate escaped veteran if applicable
            if is_first && wave >= 3 && !self.escaped_veterans.is_empty() {
                let veteran = self.escaped_veterans.remove(0);
                hero.hero_class = veteran.hero_class;
                hero.rank = veteran.rank.max(rank);
                hero.trauma_traits = veteran.trauma_traits;
                hero.is_leader = true;
            } else if is_first {
                hero.is_leader = true;
            }

            // Wave 5 Boss: Grand Master
            if wave >= 5 && is_first {
                hero.is_boss = true;
                hero.max_hp = 250;
                hero.current_hp = 250;
                hero.armor_bps = BasisPoints(5_000);
            }

            squad.push(hero);
        }

        self.active_incursion_heroes_count = squad.len() as u32;
        squad
    }

    /// Evaluates wave infamy delta strictly conforming to `SPEC-REQ-WAVE-004`:
    ///
    /// $$\text{Infamie}_{\text{vague}} = (\text{Héros tués} \times 50) + (\text{Morts par Panique} \times 25) - (\text{Héros Évadés Indemnes} \times 30)$$
    #[must_use]
    pub fn calculate_wave_infamy(killed: u32, panic_deaths: u32, escaped_unhurt: u32) -> i32 {
        let gain = (killed.saturating_mul(50)).saturating_add(panic_deaths.saturating_mul(25));
        let penalty = escaped_unhurt.saturating_mul(30);
        (gain as i32).saturating_sub(penalty as i32)
    }

    /// Updates accumulated infamy with the results of an incursion wave.
    pub fn apply_wave_infamy(
        &mut self,
        killed: u32,
        panic_deaths: u32,
        escaped_unhurt: u32,
    ) -> u32 {
        let delta = Self::calculate_wave_infamy(killed, panic_deaths, escaped_unhurt);
        if delta >= 0 {
            self.infamy = self.infamy.saturating_add(delta as u32);
        } else {
            self.infamy = self.infamy.saturating_sub((-delta) as u32);
        }
        self.infamy
    }

    /// Evaluates whether campaign victory conditions have been met.
    ///
    /// Conforms to `SPEC-REQ-WAVE-004`:
    /// - Wave 5 defeated (all members eliminated or routed).
    /// - OR accumulated Infamy >= 1000 points.
    #[must_use]
    pub fn check_victory_condition(&self) -> bool {
        self.infamy >= Self::VICTORY_INFAMY_THRESHOLD
            || (self.current_wave >= self.max_waves && self.phase == WavePhase::Debriefing)
    }

    /// Transitions from Debriefing to either Victory or the next Wave's Preparation.
    pub fn advance_to_next_wave(&mut self) {
        if self.phase != WavePhase::Debriefing {
            return;
        }

        self.stats.waves_cleared = self.stats.waves_cleared.saturating_add(1);

        if self.check_victory_condition() {
            self.phase = WavePhase::Victory;
        } else if self.current_wave < self.max_waves {
            self.current_wave = self.current_wave.saturating_add(1);
            self.phase = WavePhase::Preparation;
            self.preparation_ticks_remaining = Some(Self::DEFAULT_PREPARATION_TICKS);
            self.active_incursion_heroes_count = 0;
        } else {
            self.phase = WavePhase::Victory;
        }
    }
}

impl Default for ThreatDirector {
    fn default() -> Self {
        Self::new()
    }
}
