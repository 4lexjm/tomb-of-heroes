//! Core simulation world state module.
//!
//! Composes discrete fixed-tick clock, stable ID generator, decoupled PRNG bank,
//! engine gameplay configuration, and cryptographic state hashing.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::chrono::command::CoreCommand;
use crate::chrono::memory::ChronoHero;
use crate::config::GameConfig;
use crate::hash::{StateHash, StateHasher};
use crate::id::{LogicId, LogicIdError, LogicIdGenerator};
use crate::necro::CorpseRegistry;
use crate::rng::{DeterministicRngBank, DungeonMasterSeed};
use crate::time::Tick;
use crate::topology::{FloorId, GridCoord, WorldCoord};

/// Deterministic, headless simulation state of Tomb of Heroes.
///
/// Specified in `docs/specs/01_architecture_determinisme.md`.
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
    /// Monotonically increments `current_tick`.
    pub fn step(&mut self) {
        self.current_tick.advance();
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
}
