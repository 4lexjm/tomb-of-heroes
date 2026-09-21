//! Corpse entity, lifecycle states, and damage degradation module.
//!
//! Conforms to `SPEC-REQ-NECRO-001`.

use serde::{Deserialize, Serialize};

use crate::config::{CorpseConfig, GameConfig};
use crate::id::LogicId;
use crate::math::BasisPoints;

/// Adventurer and hero archetypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HeroClass {
    /// Frontline martial combatant with balanced resilience.
    Warrior,
    /// Holy caster specialized in divine healing and sanctification.
    Cleric,
    /// Armored holy warrior with high base bravery and defensive auras.
    Paladin,
    /// Arcane caster whose intact remains yield high-tier necromantic spectres.
    Mage,
    /// Agility specialist with high evasion and trap disarming.
    Rogue,
}

/// Physical preservation state of a fallen adventurer or minion.
///
/// Specified in `SPEC-REQ-NECRO-001`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CorpseState {
    /// Flesh and skeleton fully preserved (100% necromantic yield).
    Intact,
    /// Mutilated or crushed body parts (50% necromantic yield).
    Damaged,
    /// Pulverized bone dust, ashes, or fully consumed matter (inert).
    Destroyed,
}

/// Physical remains of a deceased hero or creature in the dungeon.
///
/// Conforms to `SPEC-REQ-NECRO-001`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Corpse {
    /// Stable, deterministic identifier for this corpse entity.
    pub corpse_id: LogicId,
    /// Identifier of the hero from which this corpse originated.
    pub source_hero_id: LogicId,
    /// Archetype class of the fallen hero.
    pub hero_class: HeroClass,
    /// Current integrity state of the corpse.
    pub state: CorpseState,
    /// Remaining structural hit points of the physical corpse.
    pub structural_hp: u32,
    /// Harvestable soul essence yield.
    pub soul_essence_value: u32,
    /// Baseline psychological terror projected onto living observers (BPS).
    pub base_terror_potency: BasisPoints,
    /// Remaining simulation ticks before advanced putrefaction destroys the corpse.
    pub decay_ticks_remaining: u32,
    /// Indicates whether the corpse has been purified by holy rites.
    pub is_sanctified: bool,
}

impl Corpse {
    /// Spawns a new intact corpse from a fallen hero according to [`GameConfig`].
    #[must_use]
    pub fn new(
        corpse_id: LogicId,
        source_hero_id: LogicId,
        hero_class: HeroClass,
        config: &GameConfig,
    ) -> Self {
        Self {
            corpse_id,
            source_hero_id,
            hero_class,
            state: CorpseState::Intact,
            structural_hp: config.corpse.nominal_structural_hp,
            soul_essence_value: config.corpse.default_soul_essence,
            base_terror_potency: config.corpse.default_terror_potency,
            decay_ticks_remaining: config.corpse.default_decay_ticks,
            is_sanctified: false,
        }
    }

    /// Applies environmental or trap damage to the corpse's structural integrity.
    ///
    /// - If `structural_hp <= damaged_threshold_hp` (nominal 25 HP) -> transitions to `Damaged`.
    /// - If `structural_hp == 0` -> transitions to `Destroyed`.
    pub fn apply_damage(&mut self, damage: u32, config: &CorpseConfig) {
        if self.state == CorpseState::Destroyed {
            self.structural_hp = 0;
            return;
        }

        self.structural_hp = self.structural_hp.saturating_sub(damage);
        if self.structural_hp == 0 {
            self.state = CorpseState::Destroyed;
        } else if self.structural_hp <= config.damaged_threshold_hp {
            self.state = CorpseState::Damaged;
        }
    }

    /// Advances corpse natural decay by one simulation tick.
    ///
    /// When `decay_ticks_remaining` reaches zero, the corpse decomposes into `Destroyed`.
    pub fn tick_decay(&mut self) {
        if self.state == CorpseState::Destroyed {
            return;
        }
        self.decay_ticks_remaining = self.decay_ticks_remaining.saturating_sub(1);
        if self.decay_ticks_remaining == 0 {
            self.state = CorpseState::Destroyed;
        }
    }

    /// Returns `true` if the corpse is physically viable for necromantic rites.
    #[inline]
    #[must_use]
    pub const fn is_viable(&self) -> bool {
        !self.is_sanctified && !matches!(self.state, CorpseState::Destroyed)
    }
}
