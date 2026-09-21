//! Discrete line-of-sight calculation using recursive shadowcasting with rational slopes.
//!
//! Specified in `SPEC-REQ-TOPO-003`.

use alloc::collections::BTreeSet;
use serde::{Deserialize, Serialize};

use crate::config::TopologyConfig;
use crate::topology::coordinates::{FloorId, GridCoord};
use crate::topology::grid::{DungeonGrid, FloorGrid};

/// Discrete angular slope represented as dy / dx with dx > 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RationalSlope {
    /// Vertical difference (dy).
    pub num: i32,
    /// Horizontal difference (dx, strictly positive).
    pub den: i32,
}

impl RationalSlope {
    /// Creates a new rational slope ensuring den > 0.
    pub const fn new(num: i32, den: i32) -> Self {
        if den < 0 {
            Self {
                num: -num,
                den: -den,
            }
        } else {
            Self { num, den }
        }
    }
}

impl PartialOrd for RationalSlope {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RationalSlope {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let lhs = (self.num as i64) * (other.den as i64);
        let rhs = (other.num as i64) * (self.den as i64);
        lhs.cmp(&rhs)
    }
}

/// Vision occlusion properties of an individual dungeon tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TileOpacity {
    /// Freely permits light and projectiles.
    Transparent,
    /// Completely blocks line of sight and projection.
    Opaque,
    /// Partially attenuates light, reducing remaining vision range by configured penalty.
    SemiOpaque,
}

/// Maps an octant index and relative coordinates (row, col) to absolute dungeon grid coordinates.
fn octant_to_grid(origin: GridCoord, octant: u8, row: i32, col: i32) -> GridCoord {
    match octant {
        0 => GridCoord::new(origin.x.saturating_add(row), origin.y.saturating_add(col)),
        1 => GridCoord::new(origin.x.saturating_add(col), origin.y.saturating_add(row)),
        2 => GridCoord::new(origin.x.saturating_sub(col), origin.y.saturating_add(row)),
        3 => GridCoord::new(origin.x.saturating_sub(row), origin.y.saturating_add(col)),
        4 => GridCoord::new(origin.x.saturating_sub(row), origin.y.saturating_sub(col)),
        5 => GridCoord::new(origin.x.saturating_sub(col), origin.y.saturating_sub(row)),
        6 => GridCoord::new(origin.x.saturating_add(col), origin.y.saturating_sub(row)),
        7 => GridCoord::new(origin.x.saturating_add(row), origin.y.saturating_sub(col)),
        _ => origin,
    }
}

/// Read-only contextual parameters for recursive octant scanning.
struct OctantContext<'a> {
    octant: u8,
    origin: GridCoord,
    floor: &'a FloorGrid,
    config: &'a TopologyConfig,
}

/// Recursively scans an angular sector within an octant.
fn scan_octant(
    ctx: &OctantContext<'_>,
    row: u32,
    start_slope: RationalSlope,
    end_slope: RationalSlope,
    max_row: u32,
    visible: &mut BTreeSet<GridCoord>,
) {
    if row > max_row || start_slope >= end_slope {
        return;
    }

    let mut current_beam: Option<(RationalSlope, u32)> = None;

    for col in 0..=row {
        let tile_lower = RationalSlope::new((2 * col as i32 - 1).max(0), 2 * row as i32);
        let tile_upper =
            RationalSlope::new((2 * col as i32 + 1).min(2 * row as i32), 2 * row as i32);

        if tile_upper < start_slope {
            continue;
        }
        if tile_lower > end_slope {
            break;
        }

        let coord = octant_to_grid(ctx.origin, ctx.octant, row as i32, col as i32);
        if ctx.floor.in_bounds(coord) {
            visible.insert(coord);
        }

        let opacity = if ctx.floor.in_bounds(coord) {
            ctx.floor.opacity(coord)
        } else {
            TileOpacity::Opaque
        };

        let trans = match opacity {
            TileOpacity::Opaque => None,
            TileOpacity::Transparent => Some(max_row),
            TileOpacity::SemiOpaque => {
                let reduced = max_row.saturating_sub(ctx.config.semi_opaque_range_penalty);
                if row < reduced {
                    Some(reduced)
                } else {
                    None
                }
            }
        };

        match current_beam {
            Some((beam_start, beam_max)) => {
                if trans == Some(beam_max) {
                    // Beam continues across column
                } else {
                    // End previous beam at tile_lower boundary
                    let beam_end = tile_lower.min(end_slope);
                    if beam_start < beam_end && row < beam_max {
                        scan_octant(ctx, row + 1, beam_start, beam_end, beam_max, visible);
                    }
                    current_beam = trans.map(|m| (tile_lower.max(start_slope), m));
                }
            }
            None => {
                if let Some(beam_max) = trans {
                    current_beam = Some((tile_lower.max(start_slope), beam_max));
                }
            }
        }
    }

    // Flush any pending beam extending to the end slope
    if let Some((beam_start, beam_max)) = current_beam {
        let beam_end = end_slope;
        if beam_start < beam_end && row < beam_max {
            scan_octant(ctx, row + 1, beam_start, beam_end, beam_max, visible);
        }
    }
}

/// Computes field of view using recursive shadowcasting on floor 0 with default config.
pub fn compute_shadowcasting_fov(
    origin: GridCoord,
    radius: u32,
    grid: &DungeonGrid,
) -> BTreeSet<GridCoord> {
    compute_shadowcasting_fov_with_config(origin, radius, grid, &TopologyConfig::default())
}

/// Computes field of view using recursive shadowcasting with provided topology configuration.
pub fn compute_shadowcasting_fov_with_config(
    origin: GridCoord,
    radius: u32,
    grid: &DungeonGrid,
    config: &TopologyConfig,
) -> BTreeSet<GridCoord> {
    if let Some(floor) = grid.floor(FloorId(0)) {
        compute_shadowcasting_fov_floor_grid(origin, radius, floor, config)
    } else if let Some((_, floor)) = grid.floors().iter().next() {
        compute_shadowcasting_fov_floor_grid(origin, radius, floor, config)
    } else {
        BTreeSet::new()
    }
}

/// Computes field of view on a specific floor.
pub fn compute_shadowcasting_fov_on_floor(
    floor_id: FloorId,
    origin: GridCoord,
    radius: u32,
    grid: &DungeonGrid,
    config: &TopologyConfig,
) -> BTreeSet<GridCoord> {
    if let Some(floor) = grid.floor(floor_id) {
        compute_shadowcasting_fov_floor_grid(origin, radius, floor, config)
    } else {
        BTreeSet::new()
    }
}

/// Computes field of view directly on a floor grid.
pub fn compute_shadowcasting_fov_floor_grid(
    origin: GridCoord,
    radius: u32,
    floor: &FloorGrid,
    config: &TopologyConfig,
) -> BTreeSet<GridCoord> {
    let mut visible = BTreeSet::new();

    if !floor.in_bounds(origin) {
        return visible;
    }

    visible.insert(origin);

    if radius == 0 {
        return visible;
    }

    for octant in 0..8 {
        let ctx = OctantContext {
            octant,
            origin,
            floor,
            config,
        };
        scan_octant(
            &ctx,
            1,
            RationalSlope::new(0, 1),
            RationalSlope::new(1, 1),
            radius,
            &mut visible,
        );
    }

    visible
}
