//! Asymmetric necromancy, undead summoning, holy sanctification, and macabre explosion.
//!
//! Conforms to `SPEC-REQ-NECRO-003` and `SPEC-REQ-NECRO-004`.

use serde::{Deserialize, Serialize};

use crate::config::NecroConfig;
use crate::id::LogicId;
use crate::math::BasisPoints;
use crate::necro::corpse::{Corpse, CorpseState, HeroClass};
use crate::necro::error::NecroError;

/// Typology of reanimated undead minions.
///
/// Specified in `SPEC-REQ-NECRO-003`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UndeadKind {
    /// Fast but fragile frontliner raised from intact or damaged remains (40 Mana).
    SkeletonGuardian,
    /// Massive hit-point meatshield with fear aura, requires intact remains (60 Mana).
    FleshWallZombie,
    /// Intangible wraith capable of passing through obstacles, requires intact Mage remains.
    FallenSoulSpectre,
}

/// Reanimated minion raised through dark arts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndeadMinion {
    /// Deterministic identifier of the reanimated entity.
    pub minion_id: LogicId,
    /// Type of undead minion.
    pub kind: UndeadKind,
    /// Identifier of the original deceased hero.
    pub source_hero_id: LogicId,
    /// Hero archetype class of the consumed corpse.
    pub source_class: HeroClass,
    /// Current hit points.
    pub hp: u32,
    /// Maximum hit points.
    pub max_hp: u32,
}

/// Outcome of a Macabre Explosion ritual.
///
/// Specified in `SPEC-REQ-NECRO-004`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacabreExplosionResult {
    /// Direct physical slashing damage dealt to victims in radius.
    pub damage: u32,
    /// Instant psychological terror potency inflicted in radius (BPS).
    pub terror_bps: BasisPoints,
    /// Blast radius in Chebyshev distance.
    pub radius: u32,
}

/// Reanimates a corpse into an undead minion using dark mana.
///
/// Postconditions:
/// - Corpse is consumed and transitioned to [`CorpseState::Destroyed`].
/// - Mana is deducted from `available_mana`.
///
/// Errors:
/// - [`NecroError::CorpseSanctified`] if the corpse was blessed.
/// - [`NecroError::InvalidCorpseState`] if corpse is Destroyed or not sufficiently preserved.
/// - [`NecroError::HeroClassMismatch`] if ritual requires a specific class (Spectre requires Mage).
/// - [`NecroError::InsufficientMana`] if `*available_mana < ritual_cost`.
pub fn raise_undead(
    corpse: &mut Corpse,
    kind: UndeadKind,
    available_mana: &mut u32,
) -> Result<UndeadMinion, NecroError> {
    let config = NecroConfig::default();
    raise_undead_with_config(corpse, kind, available_mana, &config, corpse.corpse_id)
}

/// Reanimates a corpse with explicit configuration and minion ID.
pub fn raise_undead_with_config(
    corpse: &mut Corpse,
    kind: UndeadKind,
    available_mana: &mut u32,
    config: &NecroConfig,
    minion_id: LogicId,
) -> Result<UndeadMinion, NecroError> {
    // 1. Holy sanctification check
    if corpse.is_sanctified {
        return Err(NecroError::CorpseSanctified);
    }

    // 2. Corpse physical state and class prerequisites
    let (cost, base_hp) = match kind {
        UndeadKind::SkeletonGuardian => match corpse.state {
            CorpseState::Intact | CorpseState::Damaged => (config.skeleton_mana_cost, 40),
            CorpseState::Destroyed => return Err(NecroError::InvalidCorpseState),
        },
        UndeadKind::FleshWallZombie => match corpse.state {
            CorpseState::Intact => (config.zombie_mana_cost, 120),
            CorpseState::Damaged | CorpseState::Destroyed => {
                return Err(NecroError::InvalidCorpseState);
            }
        },
        UndeadKind::FallenSoulSpectre => match corpse.state {
            CorpseState::Intact => {
                if corpse.hero_class != HeroClass::Mage {
                    return Err(NecroError::HeroClassMismatch);
                }
                (config.spectre_mana_cost, 60)
            }
            CorpseState::Damaged | CorpseState::Destroyed => {
                return Err(NecroError::InvalidCorpseState);
            }
        },
    };

    // 3. Dark mana sufficiency check
    if *available_mana < cost {
        return Err(NecroError::InsufficientMana);
    }

    // 4. State mutation: deduct mana and consume corpse
    *available_mana = available_mana.saturating_sub(cost);
    corpse.state = CorpseState::Destroyed;

    Ok(UndeadMinion {
        minion_id,
        kind,
        source_hero_id: corpse.source_hero_id,
        source_class: corpse.hero_class,
        hp: base_hp,
        max_hp: base_hp,
    })
}

/// Purifies a corpse through sacred rites of Clerics or Paladins.
///
/// Postconditions:
/// - Corpse transitions to [`CorpseState::Destroyed`].
/// - Corpse is permanently flagged `is_sanctified = true`, forbidding any future reanimation.
///
/// Errors:
/// - [`NecroError::CorpseSanctified`] if corpse was already purified.
pub fn sanctify_corpse(corpse: &mut Corpse) -> Result<(), NecroError> {
    if corpse.is_sanctified {
        return Err(NecroError::CorpseSanctified);
    }

    corpse.state = CorpseState::Destroyed;
    corpse.is_sanctified = true;
    Ok(())
}

/// Sacrifices an intact corpse via Macabre Explosion to deal AoE damage and terror.
///
/// Postconditions:
/// - Corpse is transitioned to [`CorpseState::Destroyed`].
///
/// Errors:
/// - [`NecroError::CorpseSanctified`] if corpse is holy-blessed.
/// - [`NecroError::InvalidCorpseState`] if corpse is not Intact.
pub fn explode_corpse(
    corpse: &mut Corpse,
    config: &NecroConfig,
) -> Result<MacabreExplosionResult, NecroError> {
    if corpse.is_sanctified {
        return Err(NecroError::CorpseSanctified);
    }

    if corpse.state != CorpseState::Intact {
        return Err(NecroError::InvalidCorpseState);
    }

    corpse.state = CorpseState::Destroyed;

    Ok(MacabreExplosionResult {
        damage: config.explosion_damage,
        terror_bps: config.explosion_terror_potency,
        radius: config.explosion_radius,
    })
}
