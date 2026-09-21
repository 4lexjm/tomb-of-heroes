//! Frontier exploration algorithm and discrete utility scoring.
//!
//! Specified in `SPEC-REQ-TOPO-005`.

use crate::config::TopologyConfig;
use crate::topology::coordinates::GridCoord;
use crate::topology::grid::DungeonGrid;
use crate::topology::knowledge::{HeroKnowledgeMap, TileVisibility};

/// Evaluates and selects the next exploration frontier cell based on discrete utility scoring.
///
/// Uses default [`TopologyConfig`]. Returns `None` when all accessible areas are explored.
pub fn evaluate_exploration_frontiers(
    knowledge: &HeroKnowledgeMap,
    squad_pos: GridCoord,
    grid: &DungeonGrid,
) -> Option<GridCoord> {
    evaluate_exploration_frontiers_with_config(
        knowledge,
        squad_pos,
        grid,
        &TopologyConfig::default(),
    )
}

/// Evaluates and selects the next exploration frontier cell using specified [`TopologyConfig`].
pub fn evaluate_exploration_frontiers_with_config(
    knowledge: &HeroKnowledgeMap,
    squad_pos: GridCoord,
    grid: &DungeonGrid,
    config: &TopologyConfig,
) -> Option<GridCoord> {
    let floor = grid.floor(knowledge.floor())?;

    let mut best_candidate: Option<(i64, u32, GridCoord)> = None;

    let min_coord = floor.min_coord();
    let width = floor.width();
    let height = floor.height();

    for y in 0..height {
        for x in 0..width {
            let coord = GridCoord::new(
                min_coord.x.saturating_add(x as i32),
                min_coord.y.saturating_add(y as i32),
            );

            // 1. Must be passable
            if !floor.is_passable(coord) {
                continue;
            }

            // 2. Must be Explored or InSight
            let vis = knowledge.visibility(coord);
            if vis == TileVisibility::Unexplored {
                continue;
            }

            // 3. Must have at least one neighbor that is Unexplored and passable
            let mut has_unexplored_passable_neighbor = false;
            let mut unexplored_passable_count = 0u32;

            for neighbor in coord.neighbors_4() {
                if floor.in_bounds(neighbor)
                    && floor.is_passable(neighbor)
                    && knowledge.visibility(neighbor) == TileVisibility::Unexplored
                {
                    has_unexplored_passable_neighbor = true;
                    unexplored_passable_count = unexplored_passable_count.saturating_add(1);
                }
            }

            if !has_unexplored_passable_neighbor {
                continue;
            }

            // Calculate discrete utility score S(F):
            // S(F) = U_base - (D_Manhattan * W_dist) - (TerrorMiasma * W_terror) + (RoomAreaHeuristic * W_room)
            let dist = squad_pos.manhattan_distance(coord);
            let dist_i64 = dist as i64;
            let terror_i64 = floor.terror_miasma(coord) as i64;
            let room_i64 = if floor.room_area(coord) > 0 {
                floor.room_area(coord) as i64
            } else {
                unexplored_passable_count as i64
            };

            let u_base = config.frontier_base_utility_bps.0 as i64;
            let w_dist = config.frontier_distance_weight_bps.0 as i64;
            let w_terror = config.frontier_terror_weight_bps.0 as i64;
            let w_room = config.frontier_room_weight_bps.0 as i64;

            let score =
                u_base - (dist_i64 * w_dist) - (terror_i64 * w_terror) + (room_i64 * w_room);

            let is_better = match best_candidate {
                None => true,
                Some((best_score, best_dist, best_coord)) => {
                    if score > best_score {
                        true
                    } else if score == best_score {
                        // Tie breaker: prefer smaller distance, then smaller y, then smaller x
                        if dist < best_dist {
                            true
                        } else if dist == best_dist {
                            if coord.y < best_coord.y {
                                true
                            } else if coord.y == best_coord.y {
                                coord.x < best_coord.x
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            };

            if is_better {
                best_candidate = Some((score, dist, coord));
            }
        }
    }

    best_candidate.map(|(_, _, coord)| coord)
}
