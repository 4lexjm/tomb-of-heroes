//! Discrete 2.5D A* pathfinding algorithm.
//!
//! Enforces zero floating-point arithmetic and strict determinism.

use alloc::collections::{BTreeMap, BinaryHeap};
use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::config::TopologyConfig;
use crate::topology::coordinates::WorldCoord;
use crate::topology::error::NavigationError;
use crate::topology::grid::DungeonGrid;

#[derive(Copy, Clone, Eq, PartialEq)]
struct AStarNode {
    f_score: u64,
    coord: WorldCoord,
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Inverted comparison for min-heap behavior: smallest f_score has highest priority.
        // Deterministic secondary tie-breaker using WorldCoord ordering.
        other
            .f_score
            .cmp(&self.f_score)
            .then_with(|| self.coord.cmp(&other.coord))
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Computes an admissible heuristic estimate in simulation ticks to reach `goal`.
///
/// If `current` and `goal` are on the same floor, Manhattan distance multiplied by orthogonal step
/// cost is guaranteed admissible for 4-directional grid movement.
/// If on different floors, the minimum vertical traversal cost multiplied by floor delta is
/// used, which never overestimates the true traversal cost.
fn admissible_heuristic(current: WorldCoord, goal: WorldCoord, config: &TopologyConfig) -> u64 {
    if current.floor == goal.floor {
        let manhattan = current.coord.manhattan_distance(goal.coord) as u64;
        manhattan.saturating_mul(config.orthogonal_step_cost as u64)
    } else {
        let floor_delta = current.floor.0.abs_diff(goal.floor.0) as u64;
        let min_vert_cost = config
            .pitfall_traversal_ticks
            .min(config.portal_traversal_ticks)
            .min(config.stairs_traversal_ticks)
            .min(config.ladder_traversal_ticks);
        floor_delta.saturating_mul(min_vert_cost)
    }
}

/// Discrete 2.5D A* pathfinding algorithm.
///
/// Finds an optimal sequence of discrete world coordinates connecting `start` to `goal`,
/// traversing both floor tiles orthogonally and vertical connectors (stairs, ladders, etc.).
///
/// # Errors
/// Returns [`NavigationError::OutOfBounds`] if start or goal floors are missing or coordinates
/// are outside floor bounds.
/// Returns [`NavigationError::InvalidStartLocation`] if the start tile is impassable.
/// Returns [`NavigationError::NoPathFound`] if no valid route exists or the goal tile is impassable.
pub fn find_path_astar(
    grid: &DungeonGrid,
    start: WorldCoord,
    goal: WorldCoord,
    config: &TopologyConfig,
) -> Result<Vec<WorldCoord>, NavigationError> {
    // 1. Boundary and initial existence checks
    let start_floor = grid
        .floor(start.floor)
        .ok_or(NavigationError::OutOfBounds)?;
    let goal_floor = grid.floor(goal.floor).ok_or(NavigationError::OutOfBounds)?;

    if !start_floor.in_bounds(start.coord) || !goal_floor.in_bounds(goal.coord) {
        return Err(NavigationError::OutOfBounds);
    }

    if !start_floor.is_passable(start.coord) {
        return Err(NavigationError::InvalidStartLocation);
    }

    if !goal_floor.is_passable(goal.coord) {
        return Err(NavigationError::NoPathFound);
    }

    // 2. Trivial path
    if start == goal {
        return Ok(alloc::vec![start]);
    }

    // 3. A* Search Data Structures
    let mut open_set = BinaryHeap::new();
    let mut g_scores: BTreeMap<WorldCoord, u64> = BTreeMap::new();
    let mut came_from: BTreeMap<WorldCoord, WorldCoord> = BTreeMap::new();

    g_scores.insert(start, 0);
    let initial_h = admissible_heuristic(start, goal, config);
    open_set.push(AStarNode {
        f_score: initial_h,
        coord: start,
    });

    let mut path_found = false;

    while let Some(AStarNode { coord: current, .. }) = open_set.pop() {
        if current == goal {
            path_found = true;
            break;
        }

        let current_g = match g_scores.get(&current) {
            Some(&g) => g,
            None => continue,
        };

        // Collect all valid outgoing transitions (orthogonal neighbors + vertical links)
        let mut successors: Vec<(WorldCoord, u64)> = Vec::new();

        // A. Same-floor orthogonal moves (4 cardinal directions)
        for next_coord in current.coord.neighbors_4() {
            let next_world = WorldCoord::new(current.floor, next_coord);
            if grid.is_passable(next_world) {
                successors.push((next_world, config.orthogonal_step_cost as u64));
            }
        }

        // B. Vertical connectors
        for link in grid.links_at(current) {
            if let Ok(next_world) = link.traverse(current) {
                if grid.is_passable(next_world) {
                    successors.push((next_world, link.kind.traversal_ticks(config)));
                }
            }
        }

        // Process successors
        for (neighbor, step_cost) in successors {
            let tentative_g = current_g.saturating_add(step_cost);
            let is_better = match g_scores.get(&neighbor) {
                Some(&existing_g) => tentative_g < existing_g,
                None => true,
            };

            if is_better {
                g_scores.insert(neighbor, tentative_g);
                came_from.insert(neighbor, current);
                let h = admissible_heuristic(neighbor, goal, config);
                let f = tentative_g.saturating_add(h);
                open_set.push(AStarNode {
                    f_score: f,
                    coord: neighbor,
                });
            }
        }
    }

    if !path_found {
        return Err(NavigationError::NoPathFound);
    }

    // 4. Backtrack path reconstruction
    let mut path = Vec::new();
    let mut current_step = goal;
    path.push(current_step);

    while current_step != start {
        if let Some(&prev) = came_from.get(&current_step) {
            path.push(prev);
            current_step = prev;
        } else {
            return Err(NavigationError::NoPathFound);
        }
    }

    path.reverse();
    Ok(path)
}
