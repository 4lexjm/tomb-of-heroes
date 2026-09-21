use tomb_of_heroes_core::{
    try_escape_and_merge_intel, GuildIntelRegister, HeroKnowledgeMap, IntelError, LogicId,
    SquadMember, TerrorPoints, Tick, TrapType, WorldCoord,
};

#[test]
fn test_hero_killed_at_exit_threshold_cancels_intel_transfer() {
    let mut guild = GuildIntelRegister::new();
    let mut hero_map = HeroKnowledgeMap::new();

    let exit_coord = WorldCoord::from_raw(0, 0, 0);
    let explored_tile = WorldCoord::from_raw(0, 5, 5);
    let trap_coord = WorldCoord::from_raw(0, 6, 6);

    hero_map.explore_tile(explored_tile);
    hero_map.identify_trap(trap_coord, TrapType::Blade);

    // Hero reaches exit tile, but has 0 HP (killed on the threshold)
    let dead_hero = SquadMember::new(
        LogicId(10),
        0, // Dead
        100,
        TerrorPoints::ZERO,
        false,
    );

    let result = try_escape_and_merge_intel(
        &mut guild,
        &dead_hero,
        &hero_map,
        exit_coord,
        exit_coord,
        Tick(200),
    );

    // Escape must fail with HeroDead
    assert_eq!(result, Err(IntelError::HeroDead));

    // Guild intelligence remains strictly empty
    assert!(guild.known_tiles.is_empty());
    assert!(guild.known_traps.is_empty());
}

#[test]
fn test_hero_not_at_exit_tile_cancels_intel_transfer() {
    let mut guild = GuildIntelRegister::new();
    let hero_map = HeroKnowledgeMap::new();

    let exit_coord = WorldCoord::from_raw(0, 0, 0);
    let other_coord = WorldCoord::from_raw(0, 3, 3);

    let living_hero = SquadMember::new(LogicId(11), 50, 100, TerrorPoints::ZERO, false);

    let result = try_escape_and_merge_intel(
        &mut guild,
        &living_hero,
        &hero_map,
        other_coord,
        exit_coord,
        Tick(200),
    );

    // Hero is not at extraction point
    assert_eq!(result, Err(IntelError::NotAtExitTile));
    assert!(guild.known_tiles.is_empty());
}
