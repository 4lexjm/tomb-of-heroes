//! Domain error types for corpses, terror, and necromancy.
//!
//! Enforces exhaustive error handling without unwrap/expect/panic.

use serde::{Deserialize, Serialize};

/// Errors originating from corpse lifecycle, terror mechanics, and necromantic rituals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NecroError {
    /// Corpse has been sanctified by holy rites and cannot be raised or desecrated.
    CorpseSanctified,
    /// Insufficient Dark Mana to execute the necromantic ritual.
    InsufficientMana,
    /// Corpse state is incompatible with the requested ritual.
    InvalidCorpseState,
    /// Corpse hero class does not satisfy the ritual requirement (e.g., Spectre requires Mage).
    HeroClassMismatch,
    /// Tile and all adjacent candidate tiles have reached maximum corpse capacity.
    TileCapacityExceeded,
    /// Requested corpse identifier was not found in the registry.
    CorpseNotFound,
}

impl std::fmt::Display for NecroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CorpseSanctified => {
                write!(f, "corpse has been sanctified and cannot be reanimated")
            }
            Self::InsufficientMana => {
                write!(f, "insufficient dark mana for necromantic ritual")
            }
            Self::InvalidCorpseState => {
                write!(f, "corpse state is invalid for the requested ritual")
            }
            Self::HeroClassMismatch => {
                write!(f, "corpse hero class does not match ritual requirement")
            }
            Self::TileCapacityExceeded => {
                write!(f, "all candidate tiles exceeded maximum corpse capacity")
            }
            Self::CorpseNotFound => {
                write!(f, "corpse not found in world registry")
            }
        }
    }
}

impl std::error::Error for NecroError {}
