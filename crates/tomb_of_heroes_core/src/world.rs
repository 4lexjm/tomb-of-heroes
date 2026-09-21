//! Core simulation world state module.
//!
//! Composes discrete fixed-tick clock, stable ID generator, decoupled PRNG bank,
//! engine gameplay configuration, and cryptographic state hashing.

use std::collections::{BTreeMap, BTreeSet};

use rand_xoshiro::rand_core::RngCore;
use serde::{Deserialize, Serialize};

use crate::campaign::{DungeonHeart, ThreatDirector, WavePhase};
use crate::chrono::command::CoreCommand;
use crate::chrono::memory::{ChronoHero, HeroState};
use crate::combat::{detect_trap, disarm_trap, execute_death_sequence, TrapType};
use crate::config::GameConfig;
use crate::hash::{StateHash, StateHasher};
use crate::id::{LogicId, LogicIdError, LogicIdGenerator};
use crate::intel::veterancy::VeteranProfile;
use crate::math::BasisPoints;
use crate::necro::corpse::CorpseState;
use crate::necro::necromancy::UndeadMinion;
use crate::necro::terror::{compute_terror_accumulation_with_config, PanicLevel};
use crate::necro::CorpseRegistry;
use crate::rng::{DeterministicRngBank, DungeonMasterSeed, RngStreamKind};
use crate::time::Tick;
use crate::topology::{DungeonGrid, FloorId, GridCoord, WorldCoord};

/// Deterministic, headless simulation state of Tomb of Heroes.
///
/// Specified in `docs/specs/01_architecture_determinisme.md` and `docs/specs/08_simulation_ia_heros.md`.
/// Orchestrates discrete ticks, stable entity IDs, decoupled PRNG streams,
/// gameplay configuration, and cryptographic state hashing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicWorld {
    current_tick: Tick,
    id_generator: LogicIdGenerator,
    rng_bank: DeterministicRngBank,
    config: GameConfig,
    corpses: CorpseRegistry,
    heroes: BTreeMap<LogicId, ChronoHero>,
    hazards: BTreeSet<WorldCoord>,
    dungeon: DungeonGrid,
    minions: BTreeMap<LogicId, UndeadMinion>,
    mana: u32,
    max_mana: u32,
    heart: DungeonHeart,
    campaign: ThreatDirector,
}

impl LogicWorld {
    /// Creates a new `LogicWorld` at `Tick(0)` with the given configuration and master seed.
    #[must_use]
    pub fn new(config: GameConfig, seed: DungeonMasterSeed) -> Self {
        Self {
            current_tick: Tick::ZERO,
            id_generator: LogicIdGenerator::new(),
            rng_bank: DeterministicRngBank::new(seed),
            config,
            corpses: CorpseRegistry::new(),
            heroes: BTreeMap::new(),
            hazards: BTreeSet::new(),
            dungeon: DungeonGrid::new(),
            minions: BTreeMap::new(),
            mana: 100,
            max_mana: 200,
            heart: DungeonHeart::default_sanctuary(),
            campaign: ThreatDirector::new(),
        }
    }

    /// Returns the current simulation tick.
    #[inline]
    #[must_use]
    pub const fn current_tick(&self) -> Tick {
        self.current_tick
    }

    /// Advances the simulation by exactly one fixed discrete tick.
    ///
    /// Implements `SPEC-REQ-SIM-001` through `SPEC-REQ-SIM-004`.
    pub fn step(&mut self) {
        self.current_tick.advance();
        let current_tick_val = self.current_tick.as_u64();

        // Natural mana regeneration (+1 every 10 ticks)
        if current_tick_val.is_multiple_of(10) {
            self.mana = self.mana.saturating_add(1).min(self.max_mana);
        }

        // Advance corpse decay
        self.corpses.tick_decay_all();

        // 1. Perception, Terror accumulation, and FSM update for all heroes
        let hero_ids: Vec<LogicId> = self.heroes.keys().copied().collect();
        let mut heroes_died: Vec<LogicId> = Vec::new();

        for &id in &hero_ids {
            if let Some(hero) = self.heroes.get_mut(&id) {
                if hero.current_hp == 0
                    || matches!(hero.state, HeroState::Dead | HeroState::Escaped)
                {
                    continue;
                }

                // Terror from nearby corpses
                let mut terror_delta: u32 = 0;
                for (_cid, coord, corpse) in self.corpses.iter() {
                    let dist = hero.position.coord.chebyshev_distance(coord);
                    if dist <= 6 && corpse.state != CorpseState::Destroyed {
                        let bravery = hero.bravery();
                        terror_delta =
                            terror_delta.saturating_add(compute_terror_accumulation_with_config(
                                dist,
                                corpse.base_terror_potency,
                                bravery,
                                &self.config,
                            ));
                    }
                }
                if terror_delta > 0 {
                    hero.apply_paradox_anxiety(BasisPoints(terror_delta.min(10_000)));
                }

                // FSM transitions based on terror
                if hero.terror_bps.0 >= self.config.terror.blind_panic_threshold {
                    hero.state = HeroState::Fleeing;
                    hero.is_fleeing = true;
                } else if hero.terror_bps.0 >= self.config.terror.shaken_threshold
                    && hero.state == HeroState::Infiltrating
                {
                    hero.state = HeroState::Alerted;
                    hero.is_alerted = true;
                }
            }
        }

        // 2. Locomotion along A* navmesh
        let mut occupied_tiles: BTreeMap<WorldCoord, LogicId> = BTreeMap::new();
        for (&id, hero) in &self.heroes {
            if hero.current_hp > 0 && !matches!(hero.state, HeroState::Dead | HeroState::Escaped) {
                occupied_tiles.insert(hero.position, id);
            }
        }

        for &id in &hero_ids {
            let (can_move, next_coord, is_escaped) = {
                let hero = match self.heroes.get_mut(&id) {
                    Some(h) => h,
                    None => continue,
                };
                if hero.current_hp == 0
                    || matches!(
                        hero.state,
                        HeroState::Dead | HeroState::Escaped | HeroState::Engaged
                    )
                {
                    continue;
                }

                hero.ticks_since_repath = hero.ticks_since_repath.saturating_add(1);

                if current_tick_val < hero.next_move_tick {
                    continue;
                }

                let target = if hero.is_fleeing {
                    WorldCoord::new(FloorId(0), GridCoord::new(1, 1))
                } else {
                    hero.target_destination
                        .unwrap_or_else(|| WorldCoord::new(FloorId(0), GridCoord::new(10, 10)))
                };

                if hero.path.is_empty() || hero.ticks_since_repath >= 60 {
                    if let Ok(new_path) =
                        self.dungeon
                            .find_path(hero.position, target, &self.config.topology)
                    {
                        if new_path.len() > 1 {
                            hero.path = new_path[1..].to_vec();
                            hero.ticks_since_repath = 0;
                        }
                    }
                }

                if let Some(&next) = hero.path.first() {
                    let is_blocked = match occupied_tiles.get(&next) {
                        Some(&other_id) => other_id < id,
                        None => false,
                    };

                    if is_blocked {
                        continue;
                    }

                    let escaped = hero.is_fleeing
                        && hero.position.floor == FloorId(0)
                        && hero.position.coord == GridCoord::new(1, 1);

                    (true, next, escaped)
                } else {
                    let escaped = hero.is_fleeing
                        && hero.position.floor == FloorId(0)
                        && hero.position.coord == GridCoord::new(1, 1);
                    (false, hero.position, escaped)
                }
            };

            if is_escaped {
                if let Some(h) = self.heroes.get_mut(&id) {
                    h.state = HeroState::Escaped;
                    let profile = VeteranProfile::new(
                        h.hero_id,
                        h.hero_id.0,
                        h.hero_class,
                        h.rank,
                        1,
                        Vec::new(),
                        h.trauma_traits.clone(),
                        None,
                    );
                    self.campaign.escaped_veterans.push(profile);
                    self.campaign.stats.record_escape(h.current_hp == h.max_hp);
                }
                continue;
            }

            if can_move {
                let hero = match self.heroes.get_mut(&id) {
                    Some(h) => h,
                    None => continue,
                };
                let old_pos = hero.position;
                hero.position = next_coord;
                if !hero.path.is_empty() {
                    hero.path.remove(0);
                }
                hero.next_move_tick = current_tick_val.saturating_add(hero.move_cooldown_ticks());

                occupied_tiles.remove(&old_pos);
                occupied_tiles.insert(next_coord, id);

                // 3. Traps & Hazards
                if self.hazards.contains(&next_coord) {
                    let is_fleeing = hero.is_fleeing;
                    let panic = if hero.terror_bps.0 >= self.config.terror.blind_panic_threshold {
                        PanicLevel::BlindPanic
                    } else {
                        PanicLevel::Serene
                    };

                    let roll = BasisPoints(
                        (self.rng_bank.stream_mut(RngStreamKind::Combat).next_u64() % 10_000)
                            as u32,
                    );
                    let detected = detect_trap(hero.hero_class, hero.is_alerted, panic, roll);
                    let disarmed = if detected {
                        let disarm_roll = BasisPoints(
                            (self.rng_bank.stream_mut(RngStreamKind::Combat).next_u64() % 10_000)
                                as u32,
                        );
                        disarm_trap(hero.hero_class, disarm_roll)
                    } else {
                        false
                    };

                    if !disarmed {
                        let trap_res = TrapType::Spikes.resolve_damage(is_fleeing, hero.armor_bps);
                        hero.current_hp = hero.current_hp.saturating_sub(trap_res.damage);
                        if hero.current_hp == 0 {
                            hero.state = HeroState::Dead;
                            heroes_died.push(id);
                        }
                    }
                }
            }
        }

        // 4. Death sequence execution
        for dead_id in heroes_died {
            if let Some(h) = self.heroes.get(&dead_id) {
                let panicked = h.terror_bps.0 >= self.config.terror.blind_panic_threshold;
                self.campaign.stats.record_kill(h.hero_class, panicked);
            }
            let allies: Vec<LogicId> = self
                .heroes
                .keys()
                .copied()
                .filter(|&k| k != dead_id)
                .collect();
            if let Ok(corpse_id) = self.id_generator.allocate() {
                if let Ok(outcome) = execute_death_sequence(
                    dead_id,
                    &mut self.heroes,
                    &mut self.corpses,
                    corpse_id,
                    &self.config,
                    &allies,
                ) {
                    self.mana = self
                        .mana
                        .saturating_add(outcome.mana_harvested)
                        .min(self.max_mana);
                    self.campaign
                        .stats
                        .record_mana_harvested(outcome.mana_harvested);
                }
            }
        }

        // 5. Campaign & Incursion progression
        self.resolve_campaign_tick(current_tick_val);
    }

    /// Computes the deterministic 64-bit state hash of the world.
    ///
    /// Specified in `SPEC-REQ-ARCH-005`.
    #[must_use]
    pub fn state_hash(&self) -> StateHash {
        let mut hasher = StateHasher::new();
        match serde_json::to_vec(self) {
            Ok(bytes) => {
                hasher.write_bytes(&bytes);
                hasher.finish()
            }
            Err(_) => StateHash::ZERO,
        }
    }

    /// Returns an immutable reference to the identifier generator.
    #[inline]
    #[must_use]
    pub const fn id_generator(&self) -> &LogicIdGenerator {
        &self.id_generator
    }

    /// Returns a mutable reference to the identifier generator.
    #[inline]
    pub fn id_generator_mut(&mut self) -> &mut LogicIdGenerator {
        &mut self.id_generator
    }

    /// Allocates the next stable `LogicId`.
    #[inline]
    pub fn allocate_id(&mut self) -> Result<LogicId, LogicIdError> {
        self.id_generator.allocate()
    }

    /// Returns an immutable reference to the deterministic PRNG bank.
    #[inline]
    #[must_use]
    pub const fn rng_bank(&self) -> &DeterministicRngBank {
        &self.rng_bank
    }

    /// Returns a mutable reference to the deterministic PRNG bank.
    #[inline]
    pub fn rng_bank_mut(&mut self) -> &mut DeterministicRngBank {
        &mut self.rng_bank
    }

    /// Returns an immutable reference to the engine and gameplay configuration.
    #[inline]
    #[must_use]
    pub const fn config(&self) -> &GameConfig {
        &self.config
    }

    /// Returns an immutable reference to the corpse registry.
    #[inline]
    #[must_use]
    pub const fn corpses(&self) -> &CorpseRegistry {
        &self.corpses
    }

    /// Returns a mutable reference to the corpse registry.
    #[inline]
    pub fn corpses_mut(&mut self) -> &mut CorpseRegistry {
        &mut self.corpses
    }

    /// Registers a hero into the simulation world.
    pub fn register_hero(&mut self, hero: ChronoHero) {
        self.heroes.insert(hero.hero_id, hero);
    }

    /// Returns a reference to a hero by identifier, if present.
    #[must_use]
    pub fn hero(&self, id: LogicId) -> Option<&ChronoHero> {
        self.heroes.get(&id)
    }

    /// Returns a mutable reference to a hero by identifier, if present.
    pub fn hero_mut(&mut self, id: LogicId) -> Option<&mut ChronoHero> {
        self.heroes.get_mut(&id)
    }

    /// Returns an immutable reference to the heroes registry.
    #[inline]
    #[must_use]
    pub const fn heroes(&self) -> &BTreeMap<LogicId, ChronoHero> {
        &self.heroes
    }

    /// Returns a mutable reference to the heroes registry.
    #[inline]
    pub fn heroes_mut(&mut self) -> &mut BTreeMap<LogicId, ChronoHero> {
        &mut self.heroes
    }

    /// Records an environmental hazard at the specified world coordinate.
    pub fn record_hazard(&mut self, coord: WorldCoord) {
        self.hazards.insert(coord);
    }

    /// Checks whether an environmental hazard exists at the coordinate.
    #[must_use]
    pub fn is_hazard(&self, coord: &WorldCoord) -> bool {
        self.hazards.contains(coord)
    }

    /// Returns an immutable reference to all tracked hazards.
    #[inline]
    #[must_use]
    pub const fn hazards(&self) -> &BTreeSet<WorldCoord> {
        &self.hazards
    }

    /// Applies a discrete gameplay command mutation to the world.
    pub fn apply_command(&mut self, command: &CoreCommand) {
        match command {
            CoreCommand::ArmTrap { floor, x, y, .. } => {
                let coord = WorldCoord::new(FloorId(*floor), GridCoord::new(*x, *y));
                self.hazards.insert(coord);
            }
            CoreCommand::TriggerTrapManual { .. } => {
                // Trap mechanism trigger
            }
            CoreCommand::TogglePortcullis { .. } => {
                // Portcullis state transition
            }
            CoreCommand::RaiseCorpse { corpse_id, .. } => {
                if let Some(corpse) = self.corpses.get_corpse_mut(*corpse_id) {
                    corpse.state = crate::necro::CorpseState::Destroyed;
                }
            }
            CoreCommand::ChannelTemporalRewind { .. } => {
                // High-level command, executed via execute_rewind
            }
        }
    }

    /// Returns an immutable reference to the dungeon topology.
    #[inline]
    #[must_use]
    pub const fn dungeon(&self) -> &DungeonGrid {
        &self.dungeon
    }

    /// Returns a mutable reference to the dungeon topology.
    #[inline]
    pub fn dungeon_mut(&mut self) -> &mut DungeonGrid {
        &mut self.dungeon
    }

    /// Returns an immutable reference to active minions.
    #[inline]
    #[must_use]
    pub const fn minions(&self) -> &BTreeMap<LogicId, UndeadMinion> {
        &self.minions
    }

    /// Returns a mutable reference to active minions.
    #[inline]
    pub fn minions_mut(&mut self) -> &mut BTreeMap<LogicId, UndeadMinion> {
        &mut self.minions
    }

    /// Returns current available dungeon mana.
    #[inline]
    #[must_use]
    pub const fn mana(&self) -> u32 {
        self.mana
    }

    /// Returns maximum available dungeon mana.
    #[inline]
    #[must_use]
    pub const fn max_mana(&self) -> u32 {
        self.max_mana
    }

    /// Sets available dungeon mana.
    #[inline]
    pub fn set_mana(&mut self, mana: u32) {
        self.mana = mana.min(self.max_mana);
    }

    /// Returns an immutable reference to the Dungeon Heart entity.
    #[inline]
    #[must_use]
    pub const fn heart(&self) -> &DungeonHeart {
        &self.heart
    }

    /// Returns a mutable reference to the Dungeon Heart entity.
    #[inline]
    pub fn heart_mut(&mut self) -> &mut DungeonHeart {
        &mut self.heart
    }

    /// Returns an immutable reference to the Campaign Threat Director.
    #[inline]
    #[must_use]
    pub const fn campaign(&self) -> &ThreatDirector {
        &self.campaign
    }

    /// Returns a mutable reference to the Campaign Threat Director.
    #[inline]
    pub fn campaign_mut(&mut self) -> &mut ThreatDirector {
        &mut self.campaign
    }

    /// Initiates an expedition incursion for the current campaign wave.
    pub fn start_incursion(&mut self) {
        if self.campaign.phase != WavePhase::Preparation {
            return;
        }
        let spawn_coord = WorldCoord::new(FloorId(0), GridCoord::new(1, 1));
        let target_dest = self.heart.position;
        let squad =
            self.campaign
                .generate_wave_squad(&mut self.id_generator, spawn_coord, target_dest);
        for hero in squad {
            self.heroes.insert(hero.hero_id, hero);
        }
        self.campaign.phase = WavePhase::Incursion;
        self.campaign.preparation_ticks_remaining = None;
    }

    /// Processes campaign wave progression, heart assaults, and victory/defeat evaluations.
    fn resolve_campaign_tick(&mut self, current_tick: u64) {
        match self.campaign.phase {
            WavePhase::Preparation => {
                if let Some(ref mut remaining) = self.campaign.preparation_ticks_remaining {
                    if *remaining > 0 {
                        *remaining = remaining.saturating_sub(1);
                    }
                    if *remaining == 0 {
                        self.start_incursion();
                    }
                }
            }
            WavePhase::Incursion => {
                // Check heroes assaulting the Dungeon Heart (SPEC-REQ-WAVE-003)
                let heart_pos = self.heart.position;
                let mut heart_damage: u32 = 0;
                for hero in self.heroes.values() {
                    if hero.position == heart_pos
                        && hero.current_hp > 0
                        && matches!(
                            hero.state,
                            HeroState::Infiltrating | HeroState::Alerted | HeroState::Engaged
                        )
                    {
                        // 10 damage / second = 1 damage every 2 ticks at 20 Hz
                        if current_tick.is_multiple_of(2) {
                            heart_damage = heart_damage.saturating_add(1);
                        }
                    }
                }
                if heart_damage > 0 {
                    self.heart.take_damage(heart_damage);
                }

                // Defeat condition check (SPEC-REQ-WAVE-003)
                if self.heart.is_destroyed() {
                    self.campaign.phase = WavePhase::Defeat;
                    return;
                }

                // Check resolution of incursion (SPEC-REQ-WAVE-001 & SPEC-REQ-WAVE-004)
                if self.campaign.active_incursion_heroes_count > 0 {
                    let living_count = self
                        .heroes
                        .values()
                        .filter(|h| {
                            h.current_hp > 0
                                && matches!(
                                    h.state,
                                    HeroState::Infiltrating
                                        | HeroState::Alerted
                                        | HeroState::Engaged
                                        | HeroState::Fleeing
                                )
                        })
                        .count();

                    if living_count == 0 {
                        let killed = self
                            .heroes
                            .values()
                            .filter(|h| matches!(h.state, HeroState::Dead))
                            .count() as u32;
                        let panic_deaths = self
                            .heroes
                            .values()
                            .filter(|h| {
                                matches!(h.state, HeroState::Dead)
                                    && h.terror_bps.0 >= self.config.terror.blind_panic_threshold
                            })
                            .count() as u32;
                        let escaped_unhurt = self
                            .heroes
                            .values()
                            .filter(|h| {
                                matches!(h.state, HeroState::Escaped) && h.current_hp == h.max_hp
                            })
                            .count() as u32;

                        self.campaign.phase = WavePhase::Debriefing;
                        self.campaign
                            .apply_wave_infamy(killed, panic_deaths, escaped_unhurt);

                        if self.campaign.check_victory_condition() {
                            self.campaign.phase = WavePhase::Victory;
                        }
                    }
                }
            }
            WavePhase::Debriefing | WavePhase::Victory | WavePhase::Defeat => {}
        }
    }
}
