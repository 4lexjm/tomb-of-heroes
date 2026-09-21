//! Decoupled deterministic PRNG streams module.
//!
//! Enforces `SPEC-REQ-ARCH-004` PRNG stream isolation using `rand_xoshiro::Xoshiro256PlusPlus`.

use rand_xoshiro::rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use serde::{Deserialize, Serialize};

/// Master dungeon seed initializing all independent PRNG streams.
///
/// Specified in `SPEC-REQ-ARCH-004`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DungeonMasterSeed(pub u64);

impl DungeonMasterSeed {
    /// Returns the inner raw seed as `u64`.
    #[inline]
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for DungeonMasterSeed {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<DungeonMasterSeed> for u64 {
    fn from(seed: DungeonMasterSeed) -> Self {
        seed.0
    }
}

impl std::fmt::Display for DungeonMasterSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DungeonMasterSeed({})", self.0)
    }
}

/// Decoupled PRNG stream categories preventing cross-system desynchronization.
///
/// Specified in `SPEC-REQ-ARCH-004`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RngStreamKind {
    /// Floor generation, room layouts, and trap placements.
    DungeonGen,
    /// Hit rolls, damage variance, terror checks.
    Combat,
    /// Exploration choices on equal-cost paths.
    AiDecisions,
    /// Loot tables and corpse degradation.
    LootAndDecay,
}

impl RngStreamKind {
    /// All 4 distinct stream kinds.
    pub const ALL: [Self; 4] = [
        Self::DungeonGen,
        Self::Combat,
        Self::AiDecisions,
        Self::LootAndDecay,
    ];

    /// Deterministic domain tag for stream seed derivation.
    #[inline]
    #[must_use]
    pub const fn domain_tag(self) -> u64 {
        match self {
            Self::DungeonGen => 0x5F44_554E_4745_4F4E,   // "_DUNGEON"
            Self::Combat => 0x5F43_4F4D_4241_545F,       // "_COMBAT_"
            Self::AiDecisions => 0x5F41_4944_4543_4953,  // "_AIDECIS"
            Self::LootAndDecay => 0x5F4C_4F4F_5444_4543, // "_LOOTDEC"
        }
    }
}

/// SplitMix64 deterministic 64-bit mixer.
#[inline]
const fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Derives a stream-specific 64-bit seed from a master seed and stream kind.
#[inline]
#[must_use]
pub const fn derive_stream_seed(master: DungeonMasterSeed, kind: RngStreamKind) -> u64 {
    splitmix64(master.0 ^ kind.domain_tag())
}

/// Bank of decoupled deterministic PRNG streams (`Xoshiro256PlusPlus`).
///
/// Specified in `SPEC-REQ-ARCH-004`.
/// Each channel advances independently without perturbing other simulation subsystems.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicRngBank {
    master_seed: DungeonMasterSeed,
    dungeon_gen: Xoshiro256PlusPlus,
    combat: Xoshiro256PlusPlus,
    ai_decisions: Xoshiro256PlusPlus,
    loot_and_decay: Xoshiro256PlusPlus,
}

impl DeterministicRngBank {
    /// Initializes all 4 decoupled streams from a single `DungeonMasterSeed`.
    #[must_use]
    pub fn new(master_seed: DungeonMasterSeed) -> Self {
        let dungeon_gen = Self::create_stream(master_seed, RngStreamKind::DungeonGen);
        let combat = Self::create_stream(master_seed, RngStreamKind::Combat);
        let ai_decisions = Self::create_stream(master_seed, RngStreamKind::AiDecisions);
        let loot_and_decay = Self::create_stream(master_seed, RngStreamKind::LootAndDecay);

        Self {
            master_seed,
            dungeon_gen,
            combat,
            ai_decisions,
            loot_and_decay,
        }
    }

    /// Creates a standalone virgin stream for the given kind and master seed.
    #[must_use]
    pub fn create_stream(
        master_seed: DungeonMasterSeed,
        kind: RngStreamKind,
    ) -> Xoshiro256PlusPlus {
        let stream_seed = derive_stream_seed(master_seed, kind);
        Xoshiro256PlusPlus::seed_from_u64(stream_seed)
    }

    /// Returns the master seed used to initialize this bank.
    #[inline]
    #[must_use]
    pub const fn master_seed(&self) -> DungeonMasterSeed {
        self.master_seed
    }

    /// Returns a reference to the given stream.
    #[inline]
    #[must_use]
    pub fn stream(&self, kind: RngStreamKind) -> &Xoshiro256PlusPlus {
        match kind {
            RngStreamKind::DungeonGen => &self.dungeon_gen,
            RngStreamKind::Combat => &self.combat,
            RngStreamKind::AiDecisions => &self.ai_decisions,
            RngStreamKind::LootAndDecay => &self.loot_and_decay,
        }
    }

    /// Returns a mutable reference to the given stream.
    #[inline]
    pub fn stream_mut(&mut self, kind: RngStreamKind) -> &mut Xoshiro256PlusPlus {
        match kind {
            RngStreamKind::DungeonGen => &mut self.dungeon_gen,
            RngStreamKind::Combat => &mut self.combat,
            RngStreamKind::AiDecisions => &mut self.ai_decisions,
            RngStreamKind::LootAndDecay => &mut self.loot_and_decay,
        }
    }

    /// Generates a next `u64` from the specified stream.
    #[inline]
    pub fn next_u64(&mut self, kind: RngStreamKind) -> u64 {
        self.stream_mut(kind).next_u64()
    }
}
