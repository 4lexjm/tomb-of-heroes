//! Core simulation world state module.
//!
//! Composes discrete fixed-tick clock, stable ID generator, decoupled PRNG bank,
//! engine gameplay configuration, and cryptographic state hashing.

use serde::{Deserialize, Serialize};

use crate::config::GameConfig;
use crate::hash::{StateHash, StateHasher};
use crate::id::{LogicId, LogicIdError, LogicIdGenerator};
use crate::necro::CorpseRegistry;
use crate::rng::{DeterministicRngBank, DungeonMasterSeed};
use crate::time::Tick;

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
}
