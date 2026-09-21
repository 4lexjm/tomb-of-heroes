//! Spatial positioning, tile stacking, and corpse slide mechanics.
//!
//! Conforms to `SPEC-REQ-TOPO-001` and `SPEC-REQ-NECRO-004`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::CorpseConfig;
use crate::id::LogicId;
use crate::necro::corpse::Corpse;
use crate::necro::error::NecroError;

/// Discrete 2D integer grid coordinates on a dungeon floor.
///
/// Specified in `SPEC-REQ-TOPO-001`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GridCoord {
    /// Horizontal tile coordinate.
    pub x: i32,
    /// Vertical tile coordinate.
    pub y: i32,
}

impl GridCoord {
    /// Creates a new coordinate pair.
    #[inline]
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Chebyshev distance: $\max(|x_1 - x_2|, |y_1 - y_2|)$.
    ///
    /// Used for 8-directional vision, area-of-effect spells, and terror projection.
    #[inline]
    #[must_use]
    pub fn chebyshev_distance(self, other: Self) -> u32 {
        let dx = (self.x - other.x).unsigned_abs();
        let dy = (self.y - other.y).unsigned_abs();
        dx.max(dy)
    }

    /// Manhattan distance: $|x_1 - x_2| + |y_1 - y_2|$.
    ///
    /// Used for 4-directional orthogonal movement.
    #[inline]
    #[must_use]
    pub fn manhattan_distance(self, other: Self) -> u32 {
        let dx = (self.x - other.x).unsigned_abs();
        let dy = (self.y - other.y).unsigned_abs();
        dx.saturating_add(dy)
    }
}

/// Ordered 8-directional neighbor offsets for deterministic sliding resolution.
const SLIDE_OFFSETS: [(i32, i32); 8] = [
    (0, 1),   // North
    (1, 0),   // East
    (0, -1),  // South
    (-1, 0),  // West
    (1, 1),   // North-East
    (1, -1),  // South-East
    (-1, -1), // South-West
    (-1, 1),  // North-West
];

/// Deterministic corpse storage and spatial tile registry.
///
/// Enforces `SPEC-REQ-NECRO-004`: maximum capacity per tile and adjacent sliding.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CorpseRegistry {
    /// Corpse instances indexed by their stable LogicId and current grid position.
    corpses: BTreeMap<LogicId, (GridCoord, Corpse)>,
    /// Corpse identifiers present on each tile.
    tile_corpses: BTreeMap<GridCoord, Vec<LogicId>>,
}

impl CorpseRegistry {
    /// Creates an empty corpse registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            corpses: BTreeMap::new(),
            tile_corpses: BTreeMap::new(),
        }
    }

    /// Returns the number of corpses located at the given grid coordinate.
    #[must_use]
    pub fn corpse_count_at(&self, coord: GridCoord) -> usize {
        self.tile_corpses.get(&coord).map_or(0, Vec::len)
    }

    /// Returns the identifiers of corpses located at the given coordinate.
    #[must_use]
    pub fn corpses_at(&self, coord: GridCoord) -> &[LogicId] {
        self.tile_corpses.get(&coord).map_or(&[], Vec::as_slice)
    }

    /// Places a corpse onto the grid respecting maximum tile capacity.
    ///
    /// If `target` is at capacity (`>= config.max_per_tile`), slides to the first
    /// available adjacent tile in deterministic order.
    pub fn place_corpse(
        &mut self,
        corpse: Corpse,
        target: GridCoord,
        config: &CorpseConfig,
    ) -> Result<GridCoord, NecroError> {
        self.place_corpse_with_passability(corpse, target, config, |_| true)
    }

    /// Places a corpse onto the grid with an external passability predicate.
    pub fn place_corpse_with_passability(
        &mut self,
        corpse: Corpse,
        target: GridCoord,
        config: &CorpseConfig,
        is_passable: impl Fn(GridCoord) -> bool,
    ) -> Result<GridCoord, NecroError> {
        let max_cap = config.max_per_tile as usize;

        // 1. Check if the target tile has available capacity
        let dest = if is_passable(target) && self.corpse_count_at(target) < max_cap {
            target
        } else {
            // 2. Slide to adjacent neighbor (radius 1)
            let mut found = None;
            for &(dx, dy) in &SLIDE_OFFSETS {
                let candidate = GridCoord::new(target.x + dx, target.y + dy);
                if is_passable(candidate) && self.corpse_count_at(candidate) < max_cap {
                    found = Some(candidate);
                    break;
                }
            }

            // 3. If radius 1 is full, search radius 2
            if found.is_none() {
                for r in 2_i32..=3_i32 {
                    for dy in -r..=r {
                        for dx in -r..=r {
                            if dx.abs().max(dy.abs()) == r {
                                let candidate = GridCoord::new(target.x + dx, target.y + dy);
                                if is_passable(candidate)
                                    && self.corpse_count_at(candidate) < max_cap
                                {
                                    found = Some(candidate);
                                    break;
                                }
                            }
                        }
                        if found.is_some() {
                            break;
                        }
                    }
                    if found.is_some() {
                        break;
                    }
                }
            }

            match found {
                Some(coord) => coord,
                None => return Err(NecroError::TileCapacityExceeded),
            }
        };

        // 4. Register corpse at destination coordinate
        let id = corpse.corpse_id;
        self.corpses.insert(id, (dest, corpse));
        self.tile_corpses.entry(dest).or_default().push(id);

        Ok(dest)
    }

    /// Retrieves a reference to a corpse by its stable ID.
    #[must_use]
    pub fn get_corpse(&self, id: LogicId) -> Option<&Corpse> {
        self.corpses.get(&id).map(|(_, c)| c)
    }

    /// Retrieves a mutable reference to a corpse by its stable ID.
    pub fn get_corpse_mut(&mut self, id: LogicId) -> Option<&mut Corpse> {
        self.corpses.get_mut(&id).map(|(_, c)| c)
    }

    /// Returns the current grid position of a registered corpse.
    #[must_use]
    pub fn corpse_position(&self, id: LogicId) -> Option<GridCoord> {
        self.corpses.get(&id).map(|(pos, _)| *pos)
    }

    /// Removes a corpse from the registry and frees its tile slot.
    pub fn remove_corpse(&mut self, id: LogicId) -> Option<Corpse> {
        let (pos, corpse) = self.corpses.remove(&id)?;
        if let Some(list) = self.tile_corpses.get_mut(&pos) {
            list.retain(|&entry_id| entry_id != id);
            if list.is_empty() {
                self.tile_corpses.remove(&pos);
            }
        }
        Some(corpse)
    }

    /// Total number of corpses tracked in the registry.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.corpses.len()
    }

    /// Returns `true` if the registry contains no corpses.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.corpses.is_empty()
    }
}
