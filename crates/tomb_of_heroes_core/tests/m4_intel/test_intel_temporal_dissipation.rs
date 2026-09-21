use tomb_of_heroes_core::{
    merge_intel_on_escape, GameConfig, GuildIntelRegister, HeroKnowledgeMap, IntelConfidence, Tick,
    TrapType, WorldCoord,
};

#[test]
fn test_intel_temporal_dissipation_linear_decay_5_days() {
    let config = GameConfig::default();
    let mut guild = GuildIntelRegister::new();
    let mut hero_map = HeroKnowledgeMap::new();

    let trap_coord = WorldCoord::from_raw(0, 3, 3);
    hero_map.identify_trap(trap_coord, TrapType::Spikes);

    let escape_tick = Tick(0);
    merge_intel_on_escape(&mut guild, &hero_map, escape_tick);

    // Initial confidence is 10_000 BPS
    let initial_conf = guild.trap_confidence_at(trap_coord, escape_tick, &config.intel);
    assert_eq!(initial_conf, Some(IntelConfidence(10_000)));

    // Advance 5 days:
    // 5 days * 24_000 ticks/day = 120_000 ticks
    let ticks_5_days = 5 * config.intel.ticks_per_day;
    let current_tick = Tick(ticks_5_days);

    // Exact formula: 10_000 - (120_000 * 100 / 24_000) = 10_000 - 500 = 9_500 BPS
    let conf_5_days = guild.trap_confidence_at(trap_coord, current_tick, &config.intel);
    assert_eq!(conf_5_days, Some(IntelConfidence(9_500)));

    // Advance 100 days (2_400_000 ticks):
    // 10_000 - 10_000 = 0 BPS
    let ticks_100_days = 100 * config.intel.ticks_per_day;
    let conf_100_days = guild.trap_confidence_at(trap_coord, Tick(ticks_100_days), &config.intel);
    assert_eq!(conf_100_days, Some(IntelConfidence::ZERO));

    // Advance 150 days: clamped strictly at 0 BPS
    let ticks_150_days = 150 * config.intel.ticks_per_day;
    let conf_150_days = guild.trap_confidence_at(trap_coord, Tick(ticks_150_days), &config.intel);
    assert_eq!(conf_150_days, Some(IntelConfidence::ZERO));
}
