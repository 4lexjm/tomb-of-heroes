use tomb_of_heroes_core::{
    evaluate_exploration_frontiers, DungeonGrid, FloorGrid, FloorId, GridCoord, HeroKnowledgeMap,
    TileVisibility,
};

#[test]
fn test_frontier_selection_prioritizes_closest_without_terror() {
    let mut dungeon = DungeonGrid::new();
    let mut floor = FloorGrid::new(FloorId(0), 15, 15);

    let squad_pos = GridCoord::new(5, 5);

    // Frontier A: at (5, 7) - Manhattan distance 2
    let frontier_a = GridCoord::new(5, 7);

    // Frontier B: at (5, 1) - Manhattan distance 4
    let frontier_b = GridCoord::new(5, 1);

    // Flank the corridor with impassable walls so intermediate tiles have no open unexplored lateral neighbors
    for y in 0..=8 {
        assert!(floor.set_passable(GridCoord::new(4, y), false).is_ok());
        assert!(floor.set_passable(GridCoord::new(6, y), false).is_ok());
    }

    dungeon.add_floor(floor);

    let mut knowledge = HeroKnowledgeMap::new();

    // Mark corridor from y = 1 to y = 7 as Explored
    for y in 1..=7 {
        knowledge.set_visibility(GridCoord::new(5, y), TileVisibility::Explored);
    }

    // Case 1: Zero terror on both candidates.
    // Frontier A (distance 2) must be selected over Frontier B (distance 4).
    let selected_1 = evaluate_exploration_frontiers(&knowledge, squad_pos, &dungeon);
    assert_eq!(selected_1, Some(frontier_a));

    // Case 2: Apply terror miasma to Frontier A (5 terror points = 1_000 BPS penalty).
    // Utility of A drops from 9_700 to 8_700, while B remains at 9_400.
    // Frontier B must now be selected.
    if let Some(f) = dungeon.floor_mut(FloorId(0)) {
        assert!(f.set_terror_miasma(frontier_a, 5).is_ok());
    }

    let selected_2 = evaluate_exploration_frontiers(&knowledge, squad_pos, &dungeon);
    assert_eq!(selected_2, Some(frontier_b));
}
