use tomb_of_heroes_core::{
    DimensionalAnchor, DungeonGrid, FloorGrid, FloorId, GameConfig, GateState, GridCoord, LogicId,
    NavigationError, Portcullis, WorldCoord,
};

#[test]
fn test_portcullis_blocking_and_forcing_delay() {
    let config = GameConfig::default();
    let gate_coord = WorldCoord::from_raw(0, 3, 3);
    let mut portcullis = Portcullis::new(LogicId(100), gate_coord, GateState::Closed);

    // Closed portcullis blocks passage
    assert!(portcullis.is_blocking());

    // Forcing requires 100 ticks (from config.retreat.portcullis_force_ticks)
    // Advance forcing by half the required duration (50 ticks)
    let half_ticks = config.retreat.portcullis_force_ticks / 2;
    let opened_prematurely = portcullis.tick_forcing(half_ticks, &config.retreat);
    assert!(!opened_prematurely);
    assert!(portcullis.is_blocking());
    assert_eq!(portcullis.forcing_progress_ticks, half_ticks);
    assert_eq!(portcullis.state, GateState::Closed);

    // Advance forcing by remaining ticks
    let remaining_ticks = config.retreat.portcullis_force_ticks - half_ticks;
    let opened = portcullis.tick_forcing(remaining_ticks, &config.retreat);
    assert!(opened);
    assert!(!portcullis.is_blocking());
    assert_eq!(portcullis.state, GateState::Open);
}

#[test]
fn test_portcullis_blocks_path_and_forces_detour() {
    let config = GameConfig::default();
    let mut grid = DungeonGrid::new();

    // 5x5 floor grid
    // Row 2 has a wall except at (2, 2) where a portcullis sits, and a detour passage at (4, 2)
    let mut floor0 = FloorGrid::new(FloorId(0), 5, 5);

    // Barricade row y = 2 except at (2, 2) and (4, 2)
    for x in 0..5 {
        if x != 2 && x != 4 {
            let res = floor0.set_passable(GridCoord::new(x, 2), false);
            assert!(res.is_ok());
        }
    }
    grid.add_floor(floor0);

    let start = WorldCoord::from_raw(0, 2, 0);
    let goal = WorldCoord::from_raw(0, 2, 4);

    // Initial path with portcullis open (straight through x = 2)
    let path_direct = grid.find_path(start, goal, &config.topology);
    assert!(path_direct.is_ok());
    if let Ok(path) = path_direct {
        assert!(path.contains(&WorldCoord::from_raw(0, 2, 2)));
    }

    // Now lower portcullis: tile (2, 2) becomes impassable
    if let Some(floor) = grid.floor_mut(FloorId(0)) {
        let res = floor.set_passable(GridCoord::new(2, 2), false);
        assert!(res.is_ok());
    }

    // Pathfinding must take the detour via (4, 2)
    let path_detour = grid.find_path(start, goal, &config.topology);
    assert!(path_detour.is_ok());
    if let Ok(path) = path_detour {
        assert!(!path.contains(&WorldCoord::from_raw(0, 2, 2)));
        assert!(path.contains(&WorldCoord::from_raw(0, 4, 2)));
    }

    // If detour is also closed, pathfinding fails with NoPathFound
    if let Some(floor) = grid.floor_mut(FloorId(0)) {
        let res = floor.set_passable(GridCoord::new(4, 2), false);
        assert!(res.is_ok());
    }
    let path_blocked = grid.find_path(start, goal, &config.topology);
    assert_eq!(path_blocked, Err(NavigationError::NoPathFound));
}

#[test]
fn test_dimensional_anchor_inhibits_teleport_within_radius_6() {
    let config = GameConfig::default();
    let anchor_coord = WorldCoord::from_raw(0, 10, 10);
    let anchor = DimensionalAnchor::new(LogicId(200), anchor_coord, &config.retreat);

    // Within radius 6 (Chebyshev distance <= 6) -> inhibited
    let inside_coord = WorldCoord::from_raw(0, 14, 16); // max(4, 6) = 6
    assert!(anchor.inhibits_teleport(inside_coord));

    let close_coord = WorldCoord::from_raw(0, 10, 12); // distance = 2
    assert!(anchor.inhibits_teleport(close_coord));

    // Outside radius 6 (Chebyshev distance > 6) -> not inhibited
    let outside_coord = WorldCoord::from_raw(0, 17, 10); // distance = 7
    assert!(!anchor.inhibits_teleport(outside_coord));

    // Different floor -> not inhibited
    let diff_floor_coord = WorldCoord::from_raw(1, 10, 10);
    assert!(!anchor.inhibits_teleport(diff_floor_coord));
}
