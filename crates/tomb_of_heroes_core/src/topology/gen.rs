//! Deterministic procedural multi-floor dungeon generator.
//!
//! Enforces `SPEC-REQ-GEN-001` through `SPEC-REQ-GEN-004` (`docs/specs/10_generation_procedurale_donjon.md`).

use alloc::vec::Vec;
use rand_xoshiro::rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::config::TopologyConfig;
use crate::rng::DungeonMasterSeed;
use crate::topology::coordinates::{FloorId, GridCoord, WorldCoord};
use crate::topology::error::NavigationError;
use crate::topology::grid::{DungeonGrid, FloorGrid};
use crate::topology::links::{VerticalLink, VerticalLinkKind};
use crate::topology::shadowcasting::TileOpacity;

/// Configuration parameters for procedural dungeon generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DungeonGeneratorConfig {
    /// Width of each floor in grid tiles.
    pub width: u32,
    /// Height of each floor in grid tiles.
    pub height: u32,
    /// Total number of vertical floors (nominal 3).
    pub floors: u32,
    /// Minimum inner dimension for rectangular rooms (nominal 4).
    pub min_room_dim: u32,
    /// Maximum inner dimension for rectangular rooms (nominal 8).
    pub max_room_dim: u32,
    /// BSP recursive split depth (nominal 3 for 8 leaves).
    pub bsp_depth: u32,
    /// Ratio of rejected MST edges added back as tactical loop corridors in BPS (nominal 1500 = 15%).
    pub tactical_loop_ratio_bps: u32,
}

impl Default for DungeonGeneratorConfig {
    fn default() -> Self {
        Self {
            width: 24,
            height: 24,
            floors: 3,
            min_room_dim: 4,
            max_room_dim: 8,
            bsp_depth: 3,
            tactical_loop_ratio_bps: 1500,
        }
    }
}

/// Axis-aligned bounding box for a room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomRect {
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
}

impl RoomRect {
    pub const fn new(min_x: u32, min_y: u32, max_x: u32, max_y: u32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub const fn center(&self) -> GridCoord {
        GridCoord::new(
            ((self.min_x + self.max_x) / 2) as i32,
            ((self.min_y + self.max_y) / 2) as i32,
        )
    }

    pub const fn contains(&self, coord: GridCoord) -> bool {
        coord.x >= self.min_x as i32
            && coord.x <= self.max_x as i32
            && coord.y >= self.min_y as i32
            && coord.y <= self.max_y as i32
    }
}

/// Fully constructed and validated procedural dungeon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedDungeon {
    /// Multi-floor discrete topology.
    pub grid: DungeonGrid,
    /// Player/heroes surface entrance coordinate (Floor 0).
    pub spawn_point: WorldCoord,
    /// Objective Dungeon Heart coordinate (Floor 2).
    pub dungeon_heart: WorldCoord,
    /// All stairs and vertical transition links.
    pub vertical_links: Vec<VerticalLink>,
    /// Rooms generated on each floor.
    pub floor_rooms: Vec<(FloorId, Vec<RoomRect>)>,
}

/// Deterministic disjoint set for Kruskal's MST algorithm.
struct DisjointSet {
    parent: Vec<usize>,
}

impl DisjointSet {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
        }
    }

    fn find(&mut self, i: usize) -> usize {
        let root = self.parent[i];
        if root != i {
            let p = self.find(root);
            self.parent[i] = p;
            p
        } else {
            root
        }
    }

    fn union(&mut self, i: usize, j: usize) -> bool {
        let root_i = self.find(i);
        let root_j = self.find(j);
        if root_i != root_j {
            self.parent[root_i] = root_j;
            true
        } else {
            false
        }
    }
}

/// Generates a validated multi-floor dungeon with guaranteed continuous A* connectivity.
///
/// Implements `SPEC-REQ-GEN-001` through `SPEC-REQ-GEN-004`.
pub fn generate_procedural_dungeon(
    master_seed: DungeonMasterSeed,
    config: &DungeonGeneratorConfig,
    topo_config: &TopologyConfig,
) -> Result<GeneratedDungeon, NavigationError> {
    let mut current_seed = master_seed.0;
    const RE_SEED_MASK: u64 = 0x517c_c1b7_2722_0a95;

    for _attempt in 0..5 {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(current_seed);
        if let Ok(dungeon) = try_generate_single_dungeon(&mut rng, config, topo_config) {
            return Ok(dungeon);
        }
        current_seed ^= RE_SEED_MASK;
    }

    Err(NavigationError::NoPathFound)
}

fn try_generate_single_dungeon(
    rng: &mut Xoshiro256PlusPlus,
    config: &DungeonGeneratorConfig,
    topo_config: &TopologyConfig,
) -> Result<GeneratedDungeon, NavigationError> {
    let mut dungeon_grid = DungeonGrid::new();
    let mut floor_rooms = Vec::new();
    let mut all_links = Vec::new();

    // 1. Generate BSP rooms & MST corridors for each floor
    for f in 0..config.floors {
        let floor_id = FloorId(f as u8);
        // Start all tiles as walls (impassable and opaque)
        let mut floor_grid = FloorGrid::new_with_bounds(
            floor_id,
            GridCoord::new(0, 0),
            config.width,
            config.height,
            false,
        );

        // Partition into leaves
        let mut leaves = vec![RoomRect::new(1, 1, config.width - 2, config.height - 2)];
        for _ in 0..config.bsp_depth {
            let mut next_leaves = Vec::new();
            for leaf in leaves {
                let w = leaf.max_x.saturating_sub(leaf.min_x);
                let h = leaf.max_y.saturating_sub(leaf.min_y);
                let split_horizontal = if w > h && w > config.min_room_dim * 2 + 2 {
                    false
                } else if h > config.min_room_dim * 2 + 2 {
                    true
                } else {
                    rng.next_u64() % 2 == 0
                };

                if split_horizontal && h > config.min_room_dim * 2 + 2 {
                    let min_split = leaf.min_y + config.min_room_dim + 1;
                    let max_split = leaf.max_y.saturating_sub(config.min_room_dim + 1);
                    if min_split < max_split {
                        let split =
                            min_split + (rng.next_u64() % (max_split - min_split) as u64) as u32;
                        next_leaves.push(RoomRect::new(leaf.min_x, leaf.min_y, leaf.max_x, split));
                        next_leaves.push(RoomRect::new(
                            leaf.min_x,
                            split + 1,
                            leaf.max_x,
                            leaf.max_y,
                        ));
                        continue;
                    }
                } else if !split_horizontal && w > config.min_room_dim * 2 + 2 {
                    let min_split = leaf.min_x + config.min_room_dim + 1;
                    let max_split = leaf.max_x.saturating_sub(config.min_room_dim + 1);
                    if min_split < max_split {
                        let split =
                            min_split + (rng.next_u64() % (max_split - min_split) as u64) as u32;
                        next_leaves.push(RoomRect::new(leaf.min_x, leaf.min_y, split, leaf.max_y));
                        next_leaves.push(RoomRect::new(
                            split + 1,
                            leaf.min_y,
                            leaf.max_x,
                            leaf.max_y,
                        ));
                        continue;
                    }
                }
                next_leaves.push(leaf);
            }
            leaves = next_leaves;
        }

        // Place rooms inside leaves
        let mut rooms = Vec::new();
        for leaf in &leaves {
            let lw = leaf.max_x.saturating_sub(leaf.min_x);
            let lh = leaf.max_y.saturating_sub(leaf.min_y);
            if lw < config.min_room_dim || lh < config.min_room_dim {
                continue;
            }

            let rw = config.min_room_dim
                + (rng.next_u64() % (config.max_room_dim - config.min_room_dim + 1).max(1) as u64)
                    as u32;
            let rh = config.min_room_dim
                + (rng.next_u64() % (config.max_room_dim - config.min_room_dim + 1).max(1) as u64)
                    as u32;

            let rw = rw.min(lw.saturating_sub(1)).max(config.min_room_dim);
            let rh = rh.min(lh.saturating_sub(1)).max(config.min_room_dim);

            let max_offset_x = lw.saturating_sub(rw);
            let max_offset_y = lh.saturating_sub(rh);
            let offset_x = if max_offset_x > 0 {
                (rng.next_u64() % max_offset_x as u64) as u32
            } else {
                0
            };
            let offset_y = if max_offset_y > 0 {
                (rng.next_u64() % max_offset_y as u64) as u32
            } else {
                0
            };

            let rx = leaf.min_x + offset_x;
            let ry = leaf.min_y + offset_y;
            let room = RoomRect::new(rx, ry, rx + rw, ry + rh);

            // Carve room
            for x in room.min_x..=room.max_x {
                for y in room.min_y..=room.max_y {
                    let coord = GridCoord::new(x as i32, y as i32);
                    let _ = floor_grid.set_passable(coord, true);
                    let _ = floor_grid.set_opacity(coord, TileOpacity::Transparent);
                }
            }
            rooms.push(room);
        }

        if rooms.is_empty() {
            return Err(NavigationError::OutOfBounds);
        }

        // MST Corridors connecting all rooms on this floor
        let mut edges = Vec::new();
        for i in 0..rooms.len() {
            for j in (i + 1)..rooms.len() {
                let ci = rooms[i].center();
                let cj = rooms[j].center();
                let dist = ci.manhattan_distance(cj);
                edges.push((dist, i, j));
            }
        }
        edges.sort_by_key(|e| e.0);

        let mut dsu = DisjointSet::new(rooms.len());
        let mut corridor_edges = Vec::new();
        let mut rejected_edges = Vec::new();

        for (dist, i, j) in edges {
            if dsu.union(i, j) {
                corridor_edges.push((dist, i, j));
            } else {
                rejected_edges.push((dist, i, j));
            }
        }

        // Add 15% tactical loops
        for (_dist, i, j) in rejected_edges {
            if (rng.next_u64() % 10_000) < config.tactical_loop_ratio_bps as u64 {
                corridor_edges.push((0, i, j));
            }
        }

        // Carve corridors
        for (_, i, j) in corridor_edges {
            let start = rooms[i].center();
            let end = rooms[j].center();
            carve_l_corridor(&mut floor_grid, start, end, rng.next_u64() % 2 == 0);
        }

        dungeon_grid.add_floor(floor_grid);
        floor_rooms.push((floor_id, rooms));
    }

    // 2. Connect Floors vertically with Stairs & Pitfalls
    for f in 0..(config.floors - 1) {
        let f_src = FloorId(f as u8);
        let f_dst = FloorId((f + 1) as u8);

        let rooms_src = &floor_rooms[f as usize].1;
        let rooms_dst = &floor_rooms[(f + 1) as usize].1;

        let src_room = rooms_src.last().copied().unwrap_or(rooms_src[0]);
        let stairs_pos = src_room.center();

        // Ensure target floor has passable landing
        if let Some(dst_floor) = dungeon_grid.floor_mut(f_dst) {
            let _ = dst_floor.set_passable(stairs_pos, true);
            let _ = dst_floor.set_opacity(stairs_pos, TileOpacity::Transparent);
            // Connect stairs landing to closest room on dst floor
            let dst_target = rooms_dst[0].center();
            carve_l_corridor(dst_floor, stairs_pos, dst_target, true);
        }

        let link = VerticalLink::new(
            VerticalLinkKind::Stairs,
            WorldCoord::new(f_src, stairs_pos),
            WorldCoord::new(f_dst, stairs_pos),
        );
        dungeon_grid.add_link(link);
        all_links.push(link);

        // Add 1 Pitfall
        let pit_room = &rooms_src[0];
        let pit_pos = GridCoord::new(pit_room.min_x as i32 + 1, pit_room.min_y as i32 + 1);
        if let Some(dst_floor) = dungeon_grid.floor_mut(f_dst) {
            let _ = dst_floor.set_passable(pit_pos, true);
            let _ = dst_floor.set_opacity(pit_pos, TileOpacity::Transparent);
            let dst_target = rooms_dst[0].center();
            carve_l_corridor(dst_floor, pit_pos, dst_target, false);
        }
        let pit_link = VerticalLink::new(
            VerticalLinkKind::Pitfall,
            WorldCoord::new(f_src, pit_pos),
            WorldCoord::new(f_dst, pit_pos),
        );
        dungeon_grid.add_link(pit_link);
        all_links.push(pit_link);
    }

    // 3. Define Spawn and Dungeon Heart
    let spawn_coord = floor_rooms[0].1[0].center();
    let spawn_point = WorldCoord::new(FloorId(0), spawn_coord);

    let last_floor_idx = (config.floors - 1) as usize;
    let heart_coord = floor_rooms[last_floor_idx]
        .1
        .last()
        .map_or(GridCoord::new(12, 12), |r| r.center());
    let dungeon_heart = WorldCoord::new(FloorId(last_floor_idx as u8), heart_coord);

    // 4. Sanity check: verify A* path from spawn_point to dungeon_heart
    let path = dungeon_grid.find_path(spawn_point, dungeon_heart, topo_config)?;
    if path.is_empty() {
        return Err(NavigationError::NoPathFound);
    }

    Ok(GeneratedDungeon {
        grid: dungeon_grid,
        spawn_point,
        dungeon_heart,
        vertical_links: all_links,
        floor_rooms,
    })
}

fn carve_l_corridor(
    floor: &mut FloorGrid,
    start: GridCoord,
    end: GridCoord,
    horizontal_first: bool,
) {
    if horizontal_first {
        let min_x = start.x.min(end.x);
        let max_x = start.x.max(end.x);
        for x in min_x..=max_x {
            let c = GridCoord::new(x, start.y);
            let _ = floor.set_passable(c, true);
            let _ = floor.set_opacity(c, TileOpacity::Transparent);
        }
        let min_y = start.y.min(end.y);
        let max_y = start.y.max(end.y);
        for y in min_y..=max_y {
            let c = GridCoord::new(end.x, y);
            let _ = floor.set_passable(c, true);
            let _ = floor.set_opacity(c, TileOpacity::Transparent);
        }
    } else {
        let min_y = start.y.min(end.y);
        let max_y = start.y.max(end.y);
        for y in min_y..=max_y {
            let c = GridCoord::new(start.x, y);
            let _ = floor.set_passable(c, true);
            let _ = floor.set_opacity(c, TileOpacity::Transparent);
        }
        let min_x = start.x.min(end.x);
        let max_x = start.x.max(end.x);
        for x in min_x..=max_x {
            let c = GridCoord::new(x, end.y);
            let _ = floor.set_passable(c, true);
            let _ = floor.set_opacity(c, TileOpacity::Transparent);
        }
    }
}
