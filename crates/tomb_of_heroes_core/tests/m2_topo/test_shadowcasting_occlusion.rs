use tomb_of_heroes_core::{
    compute_shadowcasting_fov, DungeonGrid, FloorGrid, FloorId, GridCoord, TileOpacity,
};

#[test]
fn test_shadowcasting_occlusion_behind_pillar() {
    let mut dungeon = DungeonGrid::new();
    let mut floor = FloorGrid::new(FloorId(0), 10, 10);

    // Opaque pillar placed at (3, 3)
    let pillar = GridCoord::new(3, 3);
    assert!(floor.set_passable(pillar, false).is_ok());
    assert!(floor.set_opacity(pillar, TileOpacity::Opaque).is_ok());

    dungeon.add_floor(floor);

    let observer = GridCoord::new(1, 1);
    let vision_radius = 8;
    let visible = compute_shadowcasting_fov(observer, vision_radius, &dungeon);

    // Observer and the front face of the pillar itself must be visible
    assert!(visible.contains(&observer));
    assert!(visible.contains(&pillar));

    // Lateral tiles flanking the line of sight must be illuminated
    let lateral_east = GridCoord::new(3, 2);
    let lateral_south = GridCoord::new(2, 3);
    assert!(visible.contains(&lateral_east));
    assert!(visible.contains(&lateral_south));

    // Cells strictly behind the opaque pillar along the diagonal line of sight must be shadowed
    let shadowed_1 = GridCoord::new(4, 4);
    let shadowed_2 = GridCoord::new(5, 5);
    assert!(!visible.contains(&shadowed_1));
    assert!(!visible.contains(&shadowed_2));
}
