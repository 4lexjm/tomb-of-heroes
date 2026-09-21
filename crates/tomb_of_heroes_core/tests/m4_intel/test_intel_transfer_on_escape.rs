use tomb_of_heroes_core::{
    merge_intel_on_escape, GameConfig, GuildIntelRegister, HeroKnowledgeMap, IntelConfidence, Tick,
    TrapType, WorldCoord,
};

#[test]
fn test_intel_transfer_on_escape_nominal() {
    let config = GameConfig::default();
    let mut guild = GuildIntelRegister::new();
    let mut hero_map = HeroKnowledgeMap::new();

    let tile1 = WorldCoord::from_raw(0, 1, 1);
    let tile2 = WorldCoord::from_raw(0, 1, 2);
    let tile3 = WorldCoord::from_raw(0, 2, 2);
    let trap_coord = WorldCoord::from_raw(0, 2, 2);

    hero_map.explore_tile(tile1);
    hero_map.explore_tile(tile2);
    hero_map.explore_tile(tile3);
    hero_map.identify_trap(trap_coord, TrapType::Fire);

    let escape_tick = Tick(500);
    merge_intel_on_escape(&mut guild, &hero_map, escape_tick);

    // Explored tiles transferred
    assert_eq!(guild.known_tiles.len(), 3);
    assert!(guild.known_tiles.contains(&tile1));
    assert!(guild.known_tiles.contains(&tile2));
    assert!(guild.known_tiles.contains(&tile3));

    // Identified trap transferred
    assert_eq!(guild.known_traps.len(), 1);
    assert!(guild.known_traps.contains_key(&trap_coord));
    if let Some(trap_record) = guild.known_traps.get(&trap_coord) {
        assert_eq!(trap_record.trap_type, TrapType::Fire);
        assert_eq!(trap_record.escape_tick, escape_tick);
        assert_eq!(
            trap_record.initial_confidence,
            IntelConfidence(config.intel.initial_confidence_bps)
        );
    }

    // Confidence query at escape tick returns 10_000 BPS
    let confidence = guild.trap_confidence_at(trap_coord, escape_tick, &config.intel);
    assert_eq!(
        confidence,
        Some(IntelConfidence(config.intel.initial_confidence_bps))
    );
}
