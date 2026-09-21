use tomb_of_heroes_core::{
    compute_shadowcasting_fov, DungeonGrid, FloorGrid, FloorId, GameConfig, GridCoord, TileOpacity,
};

#[test]
fn test_semi_opaque_attenuation_reduces_range_by_penalty() {
    let config = GameConfig::default();
    let penalty = config.topology.semi_opaque_range_penalty;
    assert_eq!(penalty, 3);

    let mut dungeon = DungeonGrid::new();
    let mut floor = FloorGrid::new(FloorId(0), 12, 12);

    // Semi-opaque mist tile placed at distance 2 along the horizontal axis
    let mist_coord = GridCoord::new(2, 0);
    assert!(floor
        .set_opacity(mist_coord, TileOpacity::SemiOpaque)
        .is_ok());

    dungeon.add_floor(floor);

    let observer = GridCoord::new(0, 0);
    let vision_radius = 8;
    let visible = compute_shadowcasting_fov(observer, vision_radius, &dungeon);

    // Observer and mist tile are visible
    assert!(visible.contains(&observer));
    assert!(visible.contains(&mist_coord));

    // Distance 3, 4, 5 tiles must remain visible (nominal 8 - 3 penalty = max reach 5)
    let reach_3 = GridCoord::new(3, 0);
    let reach_4 = GridCoord::new(4, 0);
    let reach_5 = GridCoord::new(5, 0);
    assert!(visible.contains(&reach_3));
    assert!(visible.contains(&reach_4));
    assert!(visible.contains(&reach_5));

    // Distance 6, 7, 8 must be obscured due to the 3-tile penalty
    let obscured_6 = GridCoord::new(6, 0);
    let obscured_7 = GridCoord::new(7, 0);
    let obscured_8 = GridCoord::new(8, 0);
    assert!(!visible.contains(&obscured_6));
    assert!(!visible.contains(&obscured_7));
    assert!(!visible.contains(&obscured_8));

    // Contrast with an unobstructed direction (e.g. vertical axis): tile at (0, 8) must be visible
    let unobstructed_8 = GridCoord::new(0, 8);
    assert!(visible.contains(&unobstructed_8));
}
