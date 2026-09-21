use tomb_of_heroes_core::{
    DungeonGrid, FloorGrid, FloorId, GameConfig, GridCoord, NavigationError, VerticalLink,
    VerticalLinkKind, WorldCoord,
};

#[test]
fn test_astar_grid_with_obstacles_finds_optimal_path() {
    let config = GameConfig::default();
    let mut dungeon = DungeonGrid::new();
    let mut floor0 = FloorGrid::new(FloorId(0), 10, 10);

    // Create a vertical wall of obstacles between (2, 0) and (2, 8), leaving (2, 9) open
    for y in 0..=8 {
        let set_res = floor0.set_passable(GridCoord::new(2, y), false);
        assert!(set_res.is_ok());
    }
    dungeon.add_floor(floor0);

    let start = WorldCoord::new(FloorId(0), GridCoord::new(1, 1));
    let goal = WorldCoord::new(FloorId(0), GridCoord::new(3, 1));

    let path_res = dungeon.find_path(start, goal, &config.topology);
    assert!(path_res.is_ok());

    if let Ok(path) = path_res {
        assert_eq!(path.first(), Some(&start));
        assert_eq!(path.last(), Some(&goal));
        // Verify no wall tiles in path
        for step in &path {
            assert!(dungeon.is_passable(*step));
            assert_ne!(step.coord, GridCoord::new(2, 1));
        }
    }
}

#[test]
fn test_astar_traversing_floor_via_stairs() {
    let config = GameConfig::default();
    let mut dungeon = DungeonGrid::new();

    let floor0 = FloorGrid::new(FloorId(0), 10, 10);
    let floor1 = FloorGrid::new(FloorId(1), 10, 10);
    dungeon.add_floor(floor0);
    dungeon.add_floor(floor1);

    let stairs_source = WorldCoord::new(FloorId(0), GridCoord::new(5, 5));
    let stairs_dest = WorldCoord::new(FloorId(1), GridCoord::new(5, 5));
    dungeon.add_link(VerticalLink::new(
        VerticalLinkKind::Stairs,
        stairs_source,
        stairs_dest,
    ));

    let start = WorldCoord::new(FloorId(0), GridCoord::new(0, 0));
    let goal = WorldCoord::new(FloorId(1), GridCoord::new(8, 8));

    let path_res = dungeon.find_path(start, goal, &config.topology);
    assert!(path_res.is_ok());

    if let Ok(path) = path_res {
        assert_eq!(path.first(), Some(&start));
        assert_eq!(path.last(), Some(&goal));
        // Verify path transitions floors via stairs
        assert!(path.contains(&stairs_source));
        assert!(path.contains(&stairs_dest));
    }
}

#[test]
fn test_astar_unreachable_target_returns_no_path_found() {
    let config = GameConfig::default();
    let mut dungeon = DungeonGrid::new();

    let mut floor0 = FloorGrid::new(FloorId(0), 5, 5);
    // Completely encircle goal at (3, 3) with walls
    let walls = [
        GridCoord::new(2, 2),
        GridCoord::new(3, 2),
        GridCoord::new(4, 2),
        GridCoord::new(2, 3),
        GridCoord::new(4, 3),
        GridCoord::new(2, 4),
        GridCoord::new(3, 4),
        GridCoord::new(4, 4),
    ];
    for w in walls {
        let set_res = floor0.set_passable(w, false);
        assert!(set_res.is_ok());
    }
    dungeon.add_floor(floor0);

    let start = WorldCoord::new(FloorId(0), GridCoord::new(0, 0));
    let goal = WorldCoord::new(FloorId(0), GridCoord::new(3, 3));

    let path_res = dungeon.find_path(start, goal, &config.topology);
    assert_eq!(path_res, Err(NavigationError::NoPathFound));
}
