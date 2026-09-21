//! Oriented vertical connectors for 2.5D multi-floor movement.
//!
//! Specified in `SPEC-REQ-TOPO-002`.

use serde::{Deserialize, Serialize};

use crate::config::TopologyConfig;
use crate::math::{apply_bps, BasisPoints};
use crate::topology::coordinates::WorldCoord;
use crate::topology::error::NavigationError;

/// Typology of vertical transitions between floors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerticalLinkKind {
    /// Bidirectional staircase (nominal 40 ticks).
    Stairs,
    /// Bidirectional slow ladder (nominal 80 ticks, vulnerability increased by 2500 BPS).
    Ladder,
    /// Strictly descending unidirectional trap/pit (5 ticks).
    Pitfall,
    /// Unidirectional magical portal (10 ticks).
    OneWayPortal,
}

impl VerticalLinkKind {
    /// Returns true if this vertical link allows bidirectional travel.
    pub fn is_bidirectional(&self) -> bool {
        match self {
            Self::Stairs | Self::Ladder => true,
            Self::Pitfall | Self::OneWayPortal => false,
        }
    }

    /// Returns the logical traversal duration in ticks according to configuration.
    pub fn traversal_ticks(&self, config: &TopologyConfig) -> u64 {
        match self {
            Self::Stairs => config.stairs_traversal_ticks,
            Self::Ladder => config.ladder_traversal_ticks,
            Self::Pitfall => config.pitfall_traversal_ticks,
            Self::OneWayPortal => config.portal_traversal_ticks,
        }
    }
}

/// Oriented vertical connection between two discrete 2.5D world coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VerticalLink {
    /// Transition type and directionality rules.
    pub kind: VerticalLinkKind,
    /// Source coordinate (origin of unidirectional traversal or one end of bidirectional link).
    pub source: WorldCoord,
    /// Destination coordinate (target of unidirectional traversal or other end of bidirectional link).
    pub destination: WorldCoord,
}

impl VerticalLink {
    /// Creates a new vertical link between two coordinates.
    pub const fn new(kind: VerticalLinkKind, source: WorldCoord, destination: WorldCoord) -> Self {
        Self {
            kind,
            source,
            destination,
        }
    }

    /// Traverses the link starting from `from`.
    ///
    /// Validates directionality invariants:
    /// - For bidirectional links (`Stairs`, `Ladder`), traversing from either endpoint is allowed.
    /// - For unidirectional links (`Pitfall`, `OneWayPortal`), ascending traversal from `destination`
    ///   is strictly rejected with `Err(NavigationError::PassageUnidirectional)`.
    pub fn traverse(&self, from: WorldCoord) -> Result<WorldCoord, NavigationError> {
        if from == self.source {
            Ok(self.destination)
        } else if from == self.destination {
            if self.kind.is_bidirectional() {
                Ok(self.source)
            } else {
                Err(NavigationError::PassageUnidirectional)
            }
        } else {
            Err(NavigationError::LinkNotFound)
        }
    }

    /// Validates that direct traversal from `from` to `to` via this link is allowed.
    pub fn can_traverse(&self, from: WorldCoord, to: WorldCoord) -> Result<(), NavigationError> {
        let dest = self.traverse(from)?;
        if dest == to {
            Ok(())
        } else {
            Err(NavigationError::PassageUnidirectional)
        }
    }

    /// Returns the floor difference between source and destination floors.
    pub fn delta_floor(&self) -> u8 {
        self.destination.floor.0.abs_diff(self.source.floor.0)
    }

    /// Computes fall damage for pitfall transitions based on acrobatics rating and config.
    ///
    /// Formula from `SPEC-REQ-TOPO-002`:
    /// `fall_damage = base_fall_dmg * delta_floor * (10_000 - acrobatics_bps) / 10_000`
    pub fn compute_fall_damage(&self, acrobatics_bps: BasisPoints, config: &TopologyConfig) -> u32 {
        if self.kind != VerticalLinkKind::Pitfall {
            return 0;
        }
        let raw_damage = config
            .base_fall_damage
            .saturating_mul(self.delta_floor() as u32);
        let reduction = apply_bps(raw_damage, acrobatics_bps);
        raw_damage.saturating_sub(reduction)
    }
}
