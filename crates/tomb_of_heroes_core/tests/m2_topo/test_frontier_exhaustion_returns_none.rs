use tomb_of_heroes_core::{
    evaluate_exploration_frontiers, DungeonGrid, FloorGrid, FloorId, GridCoord, HeroKnowledgeMap,
    TileVisibility,
};

#[test]
fn test_frontier_exhaustion_returns_none_when_fully_explored() {
    let mut dungeon = DungeonGrid::new();
    let floor = FloorGrid::new(FloorId(0), 4, 4);
    dungeon.add_floor(floor);

    let mut knowledge = HeroKnowledgeMap::new();

    // Mark all cells in the 4x4 floor as Explored
    for x in 0..4 {
        for y in 0..4 {
            knowledge.set_visibility(GridCoord::new(x, y), TileVisibility::Explored);
        }
    }

    let squad_pos = GridCoord::new(1, 1);
    let frontier = evaluate_exploration_frontiers(&knowledge, squad_pos, &dungeon);

    // No unexplored cells remain; exploration is exhausted
    assert_eq!(frontier, None);
}
