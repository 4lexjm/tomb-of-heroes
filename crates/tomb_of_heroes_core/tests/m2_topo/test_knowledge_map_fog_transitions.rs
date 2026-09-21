use tomb_of_heroes_core::{
    DungeonGrid, FloorGrid, FloorId, GridCoord, HeroKnowledgeMap, TileVisibility,
};

#[test]
fn test_knowledge_map_fog_transitions() {
    let mut dungeon = DungeonGrid::new();
    let floor = FloorGrid::new(FloorId(0), 15, 15);
    dungeon.add_floor(floor);

    let mut knowledge = HeroKnowledgeMap::new();

    let target_tile = GridCoord::new(2, 2);

    // Initial state: never observed
    assert_eq!(
        knowledge.visibility(target_tile),
        TileVisibility::Unexplored
    );

    // Step 1: Squad at (0, 0) with radius 5 discovers the tile -> InSight
    let fov_result_1 = knowledge.update_fov(GridCoord::new(0, 0), 5, &dungeon);
    assert!(fov_result_1.visible_tiles.contains(&target_tile));
    assert_eq!(knowledge.visibility(target_tile), TileVisibility::InSight);

    // Step 2: Squad moves far away to (10, 10) with radius 2; tile leaves sight -> Explored
    let fov_result_2 = knowledge.update_fov(GridCoord::new(10, 10), 2, &dungeon);
    assert!(!fov_result_2.visible_tiles.contains(&target_tile));
    assert_eq!(knowledge.visibility(target_tile), TileVisibility::Explored);

    // Step 3: Squad returns to (0, 0); tile re-enters sight -> InSight
    let fov_result_3 = knowledge.update_fov(GridCoord::new(0, 0), 5, &dungeon);
    assert!(fov_result_3.visible_tiles.contains(&target_tile));
    assert_eq!(knowledge.visibility(target_tile), TileVisibility::InSight);
}
