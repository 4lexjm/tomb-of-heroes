//! Adventurer squad spatial knowledge map and fog of war transitions.
//!
//! Specified in `SPEC-REQ-TOPO-004`.

use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap};
use alloc::vec::Vec;
use core::cmp::Ordering;
use serde::{Deserialize, Serialize};

use crate::config::TopologyConfig;
use crate::topology::coordinates::{FloorId, GridCoord, WorldCoord};
use crate::topology::error::NavigationError;
use crate::topology::grid::DungeonGrid;
use crate::topology::shadowcasting::compute_shadowcasting_fov_on_floor;

/// Cognitive visibility state of a tile from the heroes' perspective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TileVisibility {
    /// Tile has never been seen by adventurers; ignored by AI pathfinding.
    Unexplored,
    /// Previously observed tile now outside current field of view.
    Explored,
    /// Currently visible tile within active line of sight.
    InSight,
}

use crate::chrono::command::TrapType;

/// Result of an active field-of-view update against the heroes' mental map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FovUpdateResult {
    /// Set of coordinates currently in sight.
    pub visible_tiles: BTreeSet<GridCoord>,
    /// Coordinates where the real dungeon state conflicts with adventurer memory.
    pub discrepancies: Vec<GridCoord>,
    /// Whether a planned adventurer route was invalidated by newly observed obstacles.
    pub path_invalidated: bool,
}

/// Adventurer collective spatial memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeroKnowledgeMap {
    floor: FloorId,
    visibility: BTreeMap<GridCoord, TileVisibility>,
    remembered_passable: BTreeMap<GridCoord, bool>,
    planned_path: Option<Vec<GridCoord>>,
    /// Explored floor and grid tiles.
    pub explored_tiles: BTreeSet<WorldCoord>,
    /// Identified and cataloged traps.
    pub identified_traps: BTreeMap<WorldCoord, TrapType>,
}

impl HeroKnowledgeMap {
    /// Creates a new empty knowledge map on Floor 0.
    pub fn new() -> Self {
        Self::new_on_floor(FloorId(0))
    }

    /// Creates a new empty knowledge map on the specified floor.
    pub fn new_on_floor(floor: FloorId) -> Self {
        Self {
            floor,
            visibility: BTreeMap::new(),
            remembered_passable: BTreeMap::new(),
            planned_path: None,
            explored_tiles: BTreeSet::new(),
            identified_traps: BTreeMap::new(),
        }
    }

    /// Records an explored tile.
    pub fn explore_tile(&mut self, coord: WorldCoord) {
        self.explored_tiles.insert(coord);
    }

    /// Records an identified trap.
    pub fn identify_trap(&mut self, coord: WorldCoord, trap: TrapType) {
        self.identified_traps.insert(coord, trap);
    }

    /// Checks if a world tile has been explored.
    pub fn is_tile_explored(&self, coord: WorldCoord) -> bool {
        self.explored_tiles.contains(&coord)
    }

    /// Gets an identified trap at a given coordinate, if any.
    pub fn get_identified_trap(&self, coord: WorldCoord) -> Option<TrapType> {
        self.identified_traps.get(&coord).copied()
    }

    /// Returns the floor index this map represents.
    pub fn floor(&self) -> FloorId {
        self.floor
    }

    /// Queries the cognitive visibility state of a grid coordinate.
    pub fn visibility(&self, coord: GridCoord) -> TileVisibility {
        self.visibility
            .get(&coord)
            .copied()
            .unwrap_or(TileVisibility::Unexplored)
    }

    /// Manually sets the visibility state of a tile.
    pub fn set_visibility(&mut self, coord: GridCoord, state: TileVisibility) {
        self.visibility.insert(coord, state);
    }

    /// Checks if a tile is remembered as passable.
    pub fn is_remembered_passable(&self, coord: GridCoord) -> bool {
        self.remembered_passable
            .get(&coord)
            .copied()
            .unwrap_or(false)
    }

    /// Records remembered passability of a tile.
    pub fn set_remembered_passable(&mut self, coord: GridCoord, passable: bool) {
        self.remembered_passable.insert(coord, passable);
    }

    /// Returns current planned path, if any.
    pub fn planned_path(&self) -> Option<&[GridCoord]> {
        self.planned_path.as_deref()
    }

    /// Sets the squad's planned route.
    pub fn set_planned_path(&mut self, path: Vec<GridCoord>) {
        self.planned_path = Some(path);
    }

    /// Invalidates the squad's planned route.
    pub fn invalidate_planned_path(&mut self) {
        self.planned_path = None;
    }

    /// Verifies if a path remains valid according to current knowledge.
    pub fn is_path_valid(&self, path: &[GridCoord]) -> bool {
        for coord in path {
            if let Some(&passable) = self.remembered_passable.get(coord) {
                if !passable {
                    return false;
                }
            }
        }
        true
    }

    /// Updates field of view, transitioning tiles and detecting state discrepancies.
    pub fn update_fov(
        &mut self,
        origin: GridCoord,
        radius: u32,
        grid: &DungeonGrid,
    ) -> FovUpdateResult {
        self.update_fov_with_config(origin, radius, grid, &TopologyConfig::default())
    }

    /// Updates field of view with specified topology configuration.
    pub fn update_fov_with_config(
        &mut self,
        origin: GridCoord,
        radius: u32,
        grid: &DungeonGrid,
        config: &TopologyConfig,
    ) -> FovUpdateResult {
        let visible_tiles =
            compute_shadowcasting_fov_on_floor(self.floor, origin, radius, grid, config);

        // Transition tiles previously InSight to Explored if no longer in FOV
        for (coord, vis) in &mut self.visibility {
            if *vis == TileVisibility::InSight && !visible_tiles.contains(coord) {
                *vis = TileVisibility::Explored;
            }
        }

        let mut discrepancies = Vec::new();
        let mut path_invalidated = false;

        // Transition current FOV tiles to InSight and verify state synchronization
        for &coord in &visible_tiles {
            self.visibility.insert(coord, TileVisibility::InSight);

            let actual_passable = grid.is_passable(WorldCoord::new(self.floor, coord));

            if let Some(&remembered) = self.remembered_passable.get(&coord) {
                if remembered != actual_passable {
                    discrepancies.push(coord);
                    if remembered && !actual_passable {
                        if let Some(path) = &self.planned_path {
                            if path.contains(&coord) {
                                path_invalidated = true;
                            }
                        }
                    }
                }
            }

            self.remembered_passable.insert(coord, actual_passable);
        }

        if path_invalidated {
            self.planned_path = None;
        }

        FovUpdateResult {
            visible_tiles,
            discrepancies,
            path_invalidated,
        }
    }

    /// Finds an optimal path through the explored and visible subgraph.
    pub fn find_path(
        &self,
        start: GridCoord,
        goal: GridCoord,
        _grid: &DungeonGrid,
        config: &TopologyConfig,
    ) -> Result<Vec<GridCoord>, NavigationError> {
        if start == goal {
            return Ok(alloc::vec![start]);
        }

        // Start and goal must be known and remembered passable
        if self.visibility(start) == TileVisibility::Unexplored
            || !self.is_remembered_passable(start)
        {
            return Err(NavigationError::InvalidStartLocation);
        }
        if self.visibility(goal) == TileVisibility::Unexplored || !self.is_remembered_passable(goal)
        {
            return Err(NavigationError::InvalidGoalLocation);
        }

        #[derive(Copy, Clone, Eq, PartialEq)]
        struct Node {
            cost: u32,
            heuristic: u32,
            coord: GridCoord,
        }

        impl Ord for Node {
            fn cmp(&self, other: &Self) -> Ordering {
                let self_total = self.cost.saturating_add(self.heuristic);
                let other_total = other.cost.saturating_add(other.heuristic);
                other_total
                    .cmp(&self_total)
                    .then_with(|| other.cost.cmp(&self.cost))
                    .then_with(|| self.coord.cmp(&other.coord))
            }
        }

        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut open_set = BinaryHeap::new();
        let mut g_scores: BTreeMap<GridCoord, u32> = BTreeMap::new();
        let mut came_from: BTreeMap<GridCoord, GridCoord> = BTreeMap::new();

        g_scores.insert(start, 0);
        open_set.push(Node {
            cost: 0,
            heuristic: start.manhattan_distance(goal),
            coord: start,
        });

        while let Some(current) = open_set.pop() {
            if current.coord == goal {
                let mut path = Vec::new();
                let mut curr = goal;
                path.push(curr);
                while let Some(&prev) = came_from.get(&curr) {
                    path.push(prev);
                    curr = prev;
                }
                path.reverse();
                return Ok(path);
            }

            let best_g = g_scores.get(&current.coord).copied().unwrap_or(u32::MAX);
            if current.cost > best_g {
                continue;
            }

            for neighbor in current.coord.neighbors_4() {
                // Must be known (Explored or InSight) and remembered passable
                if self.visibility(neighbor) == TileVisibility::Unexplored {
                    continue;
                }
                if !self.is_remembered_passable(neighbor) {
                    continue;
                }

                let step_cost = config.orthogonal_step_cost;
                let tentative_g = current.cost.saturating_add(step_cost);
                let current_neighbor_g = g_scores.get(&neighbor).copied().unwrap_or(u32::MAX);

                if tentative_g < current_neighbor_g {
                    g_scores.insert(neighbor, tentative_g);
                    came_from.insert(neighbor, current.coord);
                    open_set.push(Node {
                        cost: tentative_g,
                        heuristic: neighbor.manhattan_distance(goal),
                        coord: neighbor,
                    });
                }
            }
        }

        Err(NavigationError::NoPathFound)
    }
}

impl Default for HeroKnowledgeMap {
    fn default() -> Self {
        Self::new()
    }
}
