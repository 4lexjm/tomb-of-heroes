use tomb_of_heroes_core::{
    compute_rewind_cost, execute_rewind, ActionJournal, DungeonMasterSeed, GameConfig, LogicWorld,
    SnapshotRingBuffer, Tick,
};

#[test]
fn test_rewind_mana_cost_80_ticks_deducts_24_mana() -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(555);
    let mut world = LogicWorld::new(config.clone(), seed);
    let mut journal = ActionJournal::new();
    let mut snapshots = SnapshotRingBuffer::new(config.chrono.ring_buffer_capacity);

    // Advance to tick 200, capturing snapshots at 100 and 200
    for _ in 1..=200 {
        world.step();
        snapshots.capture_if_due(&world, &config.chrono);
    }
    assert_eq!(world.current_tick(), Tick(200));

    // Standalone cost calculation check:
    // BaseCost = 20, TickCostBps = 500. Delta = 200 - 120 = 80 ticks.
    // 20 + floor(80 * 500 / 10_000) = 20 + 4 = 24 Mana.
    let cost = compute_rewind_cost(Tick(200), Tick(120), &config.chrono)?;
    assert_eq!(cost, 24);

    let mut available_mana: u32 = 100;
    execute_rewind(
        &mut world,
        &mut journal,
        &snapshots,
        Tick(120),
        &mut available_mana,
    )?;

    // 100 - 24 = 76 mana remaining
    assert_eq!(available_mana, 76);

    Ok(())
}

#[test]
fn test_rewind_zero_ticks_noop_cost() -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(555);
    let mut world = LogicWorld::new(config.clone(), seed);
    let mut journal = ActionJournal::new();
    let mut snapshots = SnapshotRingBuffer::new(config.chrono.ring_buffer_capacity);

    for _ in 1..=100 {
        world.step();
        snapshots.capture_if_due(&world, &config.chrono);
    }

    let mut available_mana: u32 = 50;
    execute_rewind(
        &mut world,
        &mut journal,
        &snapshots,
        Tick(100),
        &mut available_mana,
    )?;

    // No-op rewind: mana intact
    assert_eq!(available_mana, 50);

    Ok(())
}
