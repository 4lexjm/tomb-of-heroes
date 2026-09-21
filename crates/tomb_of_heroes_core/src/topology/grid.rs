//! Floor and multi-floor dungeon grid representations.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::config::TopologyConfig;
use crate::topology::astar::find_path_astar;
use crate::topology::coordinates::{FloorId, GridCoord, WorldCoord};
use crate::topology::error::NavigationError;
use crate::topology::links::VerticalLink;

/// Passability and geometry for an individual floor in the dungeon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloorGrid {
    floor: FloorId,
    min_coord: GridCoord,
    width: u32,
    height: u32,
    passable: Vec<bool>,
}

impl FloorGrid {
    /// Creates a new floor grid with bounds starting at (0, 0) and all tiles passable.
    pub fn new(floor: FloorId, width: u32, height: u32) -> Self {
        Self::new_with_bounds(floor, GridCoord::new(0, 0), width, height, true)
    }

    /// Creates a new floor grid with specified bounds and initial passability.
    pub fn new_with_bounds(
        floor: FloorId,
        min_coord: GridCoord,
        width: u32,
        height: u32,
        default_passable: bool,
    ) -> Self {
        let total_cells = (width as usize).saturating_mul(height as usize);
        Self {
            floor,
            min_coord,
            width,
            height,
            passable: alloc::vec![default_passable; total_cells],
        }
    }

    /// Returns the floor identifier.
    pub fn floor(&self) -> FloorId {
        self.floor
    }

    /// Returns the width in tiles.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height in tiles.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the minimum (origin) coordinate.
    pub fn min_coord(&self) -> GridCoord {
        self.min_coord
    }

    /// Returns true if the coordinate is within grid bounds.
    pub fn in_bounds(&self, coord: GridCoord) -> bool {
        if coord.x < self.min_coord.x || coord.y < self.min_coord.y {
            return false;
        }
        let rel_x = (coord.x - self.min_coord.x) as u32;
        let rel_y = (coord.y - self.min_coord.y) as u32;
        rel_x < self.width && rel_y < self.height
    }

    fn coord_to_index(&self, coord: GridCoord) -> Result<usize, NavigationError> {
        if !self.in_bounds(coord) {
            return Err(NavigationError::OutOfBounds);
        }
        let rel_x = (coord.x - self.min_coord.x) as usize;
        let rel_y = (coord.y - self.min_coord.y) as usize;
        Ok(rel_y * (self.width as usize) + rel_x)
    }

    /// Checks if a tile is passable. Returns false if out of bounds.
    pub fn is_passable(&self, coord: GridCoord) -> bool {
        match self.coord_to_index(coord) {
            Ok(idx) => match self.passable.get(idx) {
                Some(&p) => p,
                None => false,
            },
            Err(_) => false,
        }
    }

    /// Sets the passability of a coordinate on this floor.
    pub fn set_passable(
        &mut self,
        coord: GridCoord,
        passable: bool,
    ) -> Result<(), NavigationError> {
        let idx = self.coord_to_index(coord)?;
        if let Some(cell) = self.passable.get_mut(idx) {
            *cell = passable;
            Ok(())
        } else {
            Err(NavigationError::OutOfBounds)
        }
    }
}

/// Multi-floor discrete dungeon topology containing all floor grids and vertical links.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DungeonGrid {
    floors: BTreeMap<FloorId, FloorGrid>,
    links: Vec<VerticalLink>,
}

impl DungeonGrid {
    /// Creates an empty multi-floor dungeon grid.
    pub fn new() -> Self {
        Self {
            floors: BTreeMap::new(),
            links: Vec::new(),
        }
    }

    /// Adds or replaces a floor grid.
    pub fn add_floor(&mut self, floor: FloorGrid) {
        self.floors.insert(floor.floor(), floor);
    }

    /// Adds a vertical transition link between floors.
    pub fn add_link(&mut self, link: VerticalLink) {
        self.links.push(link);
    }

    /// Returns a reference to the specified floor, if present.
    pub fn floor(&self, floor: FloorId) -> Option<&FloorGrid> {
        self.floors.get(&floor)
    }

    /// Returns a mutable reference to the specified floor, if present.
    pub fn floor_mut(&mut self, floor: FloorId) -> Option<&mut FloorGrid> {
        self.floors.get_mut(&floor)
    }

    /// Returns a slice of all registered vertical links.
    pub fn links(&self) -> &[VerticalLink] {
        &self.links
    }

    /// Returns true if the coordinate is within floor bounds and marked passable.
    pub fn is_passable(&self, coord: WorldCoord) -> bool {
        if let Some(floor_grid) = self.floors.get(&coord.floor) {
            floor_grid.is_passable(coord.coord)
        } else {
            false
        }
    }

    /// Returns all vertical links attached to the specified world coordinate.
    pub fn links_at(&self, coord: WorldCoord) -> Vec<VerticalLink> {
        self.links
            .iter()
            .filter(|link| link.source == coord || link.destination == coord)
            .copied()
            .collect()
    }

    /// Navigates using discrete 2.5D A* pathfinding.
    pub fn find_path(
        &self,
        start: WorldCoord,
        goal: WorldCoord,
        config: &TopologyConfig,
    ) -> Result<Vec<WorldCoord>, NavigationError> {
        find_path_astar(self, start, goal, config)
    }
}

impl Default for DungeonGrid {
    fn default() -> Self {
        Self::new()
    }
}
