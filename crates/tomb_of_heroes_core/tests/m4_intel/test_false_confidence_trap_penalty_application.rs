use tomb_of_heroes_core::{
    merge_intel_on_escape, GameConfig, GuildIntelRegister, HeroKnowledgeMap, Tick, TrapType,
    WorldCoord,
};

#[test]
fn test_false_confidence_trap_penalty_when_altered() {
    let config = GameConfig::default();
    let mut guild = GuildIntelRegister::new();
    let mut hero_map = HeroKnowledgeMap::new();

    let trap_coord = WorldCoord::from_raw(0, 4, 4);

    // Hero records Spikes trap at (0, 4, 4)
    hero_map.identify_trap(trap_coord, TrapType::Spikes);
    let escape_tick = Tick(0);
    merge_intel_on_escape(&mut guild, &hero_map, escape_tick);

    // 1. Dungeon Master altered the trap to Fire!
    // Guild confidence is 10_000 BPS >= 5_000 BPS (fresh intel)
    let current_tick = Tick(100);
    let penalty = guild.calculate_trap_detection_penalty(
        trap_coord,
        Some(TrapType::Fire),
        current_tick,
        &config.intel,
    );

    // Expect -3_000 BPS malus applied due to false confidence bias
    assert_eq!(penalty, -3_000);

    // 2. DM disarmed the trap (actual_trap = None)
    let penalty_disarmed =
        guild.calculate_trap_detection_penalty(trap_coord, None, current_tick, &config.intel);
    assert_eq!(penalty_disarmed, -3_000);

    // 3. Trap was NOT altered (actual_trap = Some(TrapType::Spikes))
    let penalty_unaltered = guild.calculate_trap_detection_penalty(
        trap_coord,
        Some(TrapType::Spikes),
        current_tick,
        &config.intel,
    );
    assert_eq!(penalty_unaltered, 0);

    // 4. Guild confidence has decayed below 5_000 BPS threshold:
    // 55 days = 55 * 24_000 = 1_320_000 ticks
    // Dissipation: 55 * 100 = 5_500 BPS. Remaining confidence: 4_500 BPS < 5_000 BPS.
    let old_tick = Tick(55 * config.intel.ticks_per_day);
    let penalty_expired = guild.calculate_trap_detection_penalty(
        trap_coord,
        Some(TrapType::Fire),
        old_tick,
        &config.intel,
    );

    // No penalty applied because guild confidence is below threshold (< 5_000 BPS)
    assert_eq!(penalty_expired, 0);
}
