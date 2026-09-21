use tomb_of_heroes_core::{
    DungeonGrid, FloorGrid, FloorId, GridCoord, HeroKnowledgeMap, TileOpacity, TileVisibility,
};

#[test]
fn test_door_state_discrepancy_invalidates_planned_path_on_rediscovery() {
    let mut dungeon = DungeonGrid::new();
    let mut floor = FloorGrid::new(FloorId(0), 12, 12);

    let door = GridCoord::new(5, 5);
    // Initially door is open / passable
    assert!(floor.set_passable(door, true).is_ok());
    assert!(floor.set_opacity(door, TileOpacity::Transparent).is_ok());

    dungeon.add_floor(floor);

    let mut knowledge = HeroKnowledgeMap::new();

    // Squad previously explored the corridor and remembers the door as open
    let route = vec![
        GridCoord::new(3, 5),
        GridCoord::new(4, 5),
        door,
        GridCoord::new(6, 5),
    ];
    for step in &route {
        knowledge.set_visibility(*step, TileVisibility::Explored);
        knowledge.set_remembered_passable(*step, true);
    }
    knowledge.set_planned_path(route.clone());
    assert!(knowledge.is_path_valid(&route));

    // Player/Dungeon Master closes the reinforced door while it is in the fog
    if let Some(f) = dungeon.floor_mut(FloorId(0)) {
        assert!(f.set_passable(door, false).is_ok());
        assert!(f.set_opacity(door, TileOpacity::Opaque).is_ok());
    }

    // While squad is far away, the door remains in fog and remembered as passable
    assert!(knowledge.is_remembered_passable(door));

    // Squad arrives at (4, 5) with line of sight: the door enters InSight
    let fov_result = knowledge.update_fov(GridCoord::new(4, 5), 5, &dungeon);

    // Invalidation must trigger immediately
    assert!(fov_result.discrepancies.contains(&door));
    assert!(fov_result.path_invalidated);
    assert!(knowledge.planned_path().is_none());
    assert!(!knowledge.is_remembered_passable(door));
    assert!(!knowledge.is_path_valid(&route));
}
