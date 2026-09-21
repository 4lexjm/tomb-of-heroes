//! Sacred Dungeon Heart sanctuary entity and defeat conditions.
//!
//! Conforms to `SPEC-REQ-WAVE-003` (`docs/specs/11_vagues_campagne_victoire.md`).

use serde::{Deserialize, Serialize};

use crate::math::BasisPoints;
use crate::topology::coordinates::{FloorId, GridCoord, WorldCoord};

/// Sacred core entity of the dungeon.
///
/// Specified in `SPEC-REQ-WAVE-003`:
/// - Situated in the deepest sanctuary room (Floor 2).
/// - Initial Hit Points: 500 PV.
/// - Armor Mitigation: 0 BPS.
/// - Immediate campaign defeat if PV <= 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DungeonHeart {
    /// Current health points (initially 500).
    pub current_hp: u32,
    /// Maximum health points (nominal 500).
    pub max_hp: u32,
    /// Armor mitigation in basis points (nominal 0 BPS).
    pub armor_bps: BasisPoints,
    /// Spatial location of the heart sanctuary.
    pub position: WorldCoord,
}

impl DungeonHeart {
    /// Nominal maximum health points of the Dungeon Heart.
    pub const DEFAULT_MAX_HP: u32 = 500;

    /// Creates a new Dungeon Heart entity at the specified world coordinate.
    #[must_use]
    pub const fn new(position: WorldCoord) -> Self {
        Self {
            current_hp: Self::DEFAULT_MAX_HP,
            max_hp: Self::DEFAULT_MAX_HP,
            armor_bps: BasisPoints::ZERO,
            position,
        }
    }

    /// Creates a default sanctuary Heart located on floor 2 at grid (5, 5).
    #[must_use]
    pub const fn default_sanctuary() -> Self {
        Self::new(WorldCoord::new(FloorId(2), GridCoord::new(5, 5)))
    }

    /// Inflicts unmitigated direct damage to the Dungeon Heart.
    pub fn take_damage(&mut self, amount: u32) {
        self.current_hp = self.current_hp.saturating_sub(amount);
    }

    /// Returns `true` if the Dungeon Heart has been completely destroyed.
    #[inline]
    #[must_use]
    pub const fn is_destroyed(&self) -> bool {
        self.current_hp == 0
    }
}

impl Default for DungeonHeart {
    fn default() -> Self {
        Self::default_sanctuary()
    }
}
