//! Floor and multi-floor dungeon grid representations.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::config::TopologyConfig;
use crate::topology::astar::find_path_astar;
use crate::topology::coordinates::{FloorId, GridCoord, WorldCoord};
use crate::topology::error::NavigationError;
use crate::topology::links::VerticalLink;
use crate::topology::shadowcasting::TileOpacity;

/// Passability and geometry for an individual floor in the dungeon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloorGrid {
    floor: FloorId,
    min_coord: GridCoord,
    width: u32,
    height: u32,
    passable: Vec<bool>,
    opacity: Vec<TileOpacity>,
    terror_miasma: Vec<u32>,
    room_area: Vec<u32>,
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
        let default_opacity = if default_passable {
            TileOpacity::Transparent
        } else {
            TileOpacity::Opaque
        };
        Self {
            floor,
            min_coord,
            width,
            height,
            passable: alloc::vec![default_passable; total_cells],
            opacity: alloc::vec![default_opacity; total_cells],
            terror_miasma: alloc::vec![0; total_cells],
            room_area: alloc::vec![0; total_cells],
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
            if let Some(op) = self.opacity.get_mut(idx) {
                if !passable && *op == TileOpacity::Transparent {
                    *op = TileOpacity::Opaque;
                } else if passable && *op == TileOpacity::Opaque {
                    *op = TileOpacity::Transparent;
                }
            }
            Ok(())
        } else {
            Err(NavigationError::OutOfBounds)
        }
    }

    /// Returns the vision opacity of the specified coordinate.
    pub fn opacity(&self, coord: GridCoord) -> TileOpacity {
        match self.coord_to_index(coord) {
            Ok(idx) => self
                .opacity
                .get(idx)
                .copied()
                .unwrap_or(TileOpacity::Opaque),
            Err(_) => TileOpacity::Opaque,
        }
    }

    /// Sets the vision opacity of a tile on this floor.
    pub fn set_opacity(
        &mut self,
        coord: GridCoord,
        opacity: TileOpacity,
    ) -> Result<(), NavigationError> {
        let idx = self.coord_to_index(coord)?;
        if let Some(cell) = self.opacity.get_mut(idx) {
            *cell = opacity;
            Ok(())
        } else {
            Err(NavigationError::OutOfBounds)
        }
    }

    /// Returns terror miasma intensity at the given coordinate.
    pub fn terror_miasma(&self, coord: GridCoord) -> u32 {
        match self.coord_to_index(coord) {
            Ok(idx) => self.terror_miasma.get(idx).copied().unwrap_or(0),
            Err(_) => 0,
        }
    }

    /// Sets terror miasma intensity at the given coordinate.
    pub fn set_terror_miasma(
        &mut self,
        coord: GridCoord,
        miasma: u32,
    ) -> Result<(), NavigationError> {
        let idx = self.coord_to_index(coord)?;
        if let Some(cell) = self.terror_miasma.get_mut(idx) {
            *cell = miasma;
            Ok(())
        } else {
            Err(NavigationError::OutOfBounds)
        }
    }

    /// Returns room area heuristic value at the given coordinate.
    pub fn room_area(&self, coord: GridCoord) -> u32 {
        match self.coord_to_index(coord) {
            Ok(idx) => self.room_area.get(idx).copied().unwrap_or(0),
            Err(_) => 0,
        }
    }

    /// Sets room area heuristic value at the given coordinate.
    pub fn set_room_area(&mut self, coord: GridCoord, area: u32) -> Result<(), NavigationError> {
        let idx = self.coord_to_index(coord)?;
        if let Some(cell) = self.room_area.get_mut(idx) {
            *cell = area;
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

    /// Returns the vision opacity of a world coordinate.
    pub fn opacity(&self, coord: WorldCoord) -> TileOpacity {
        if let Some(floor_grid) = self.floors.get(&coord.floor) {
            floor_grid.opacity(coord.coord)
        } else {
            TileOpacity::Opaque
        }
    }

    /// Sets the vision opacity of a world coordinate.
    pub fn set_opacity(
        &mut self,
        coord: WorldCoord,
        opacity: TileOpacity,
    ) -> Result<(), NavigationError> {
        if let Some(floor_grid) = self.floors.get_mut(&coord.floor) {
            floor_grid.set_opacity(coord.coord, opacity)
        } else {
            Err(NavigationError::FloorNotFound)
        }
    }

    /// Returns the terror miasma at a world coordinate.
    pub fn terror_miasma(&self, coord: WorldCoord) -> u32 {
        if let Some(floor_grid) = self.floors.get(&coord.floor) {
            floor_grid.terror_miasma(coord.coord)
        } else {
            0
        }
    }

    /// Sets the terror miasma at a world coordinate.
    pub fn set_terror_miasma(
        &mut self,
        coord: WorldCoord,
        miasma: u32,
    ) -> Result<(), NavigationError> {
        if let Some(floor_grid) = self.floors.get_mut(&coord.floor) {
            floor_grid.set_terror_miasma(coord.coord, miasma)
        } else {
            Err(NavigationError::FloorNotFound)
        }
    }

    /// Returns the room area heuristic at a world coordinate.
    pub fn room_area(&self, coord: WorldCoord) -> u32 {
        if let Some(floor_grid) = self.floors.get(&coord.floor) {
            floor_grid.room_area(coord.coord)
        } else {
            0
        }
    }

    /// Sets the room area heuristic at a world coordinate.
    pub fn set_room_area(&mut self, coord: WorldCoord, area: u32) -> Result<(), NavigationError> {
        if let Some(floor_grid) = self.floors.get_mut(&coord.floor) {
            floor_grid.set_room_area(coord.coord, area)
        } else {
            Err(NavigationError::FloorNotFound)
        }
    }

    /// Returns reference to all registered floor grids.
    pub fn floors(&self) -> &BTreeMap<FloorId, FloorGrid> {
        &self.floors
    }
}

impl Default for DungeonGrid {
    fn default() -> Self {
        Self::new()
    }
}
