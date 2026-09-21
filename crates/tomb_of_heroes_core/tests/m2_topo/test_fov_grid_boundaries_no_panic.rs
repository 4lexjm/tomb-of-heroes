use tomb_of_heroes_core::{compute_shadowcasting_fov, DungeonGrid, FloorGrid, FloorId, GridCoord};

#[test]
fn test_fov_grid_boundaries_no_panic() {
    let mut dungeon = DungeonGrid::new();
    let min_coord = GridCoord::new(-128, -128);
    let floor = FloorGrid::new_with_bounds(FloorId(0), min_coord, 16, 16, true);
    dungeon.add_floor(floor);

    let observer = GridCoord::new(-128, -128);
    let vision_radius = 8;

    // Must compute FOV right at the negative boundary without integer underflow or index panic
    let visible = compute_shadowcasting_fov(observer, vision_radius, &dungeon);

    // Observer is visible
    assert!(visible.contains(&observer));

    // Valid in-bounds neighbors are visible
    let in_bounds_neighbor = GridCoord::new(-127, -128);
    assert!(visible.contains(&in_bounds_neighbor));

    // Out-of-bounds coordinates must not be included
    let out_of_bounds = GridCoord::new(-129, -128);
    assert!(!visible.contains(&out_of_bounds));
}
