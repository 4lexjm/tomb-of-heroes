use tomb_of_heroes_core::{
    execute_rewind, ActionJournal, ChronoError, DungeonMasterSeed, GameConfig, LogicWorld,
    SnapshotRingBuffer, Tick,
};

#[test]
fn test_rewind_with_insufficient_mana_preserves_state() -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(888);
    let mut world = LogicWorld::new(config.clone(), seed);
    let mut journal = ActionJournal::new();
    let mut snapshots = SnapshotRingBuffer::new(config.chrono.ring_buffer_capacity);

    for _ in 1..=200 {
        world.step();
        snapshots.capture_if_due(&world, &config.chrono);
    }

    let initial_hash = world.state_hash();

    // 1. Zero mana available (required: 24)
    let mut available_mana: u32 = 0;
    let result = execute_rewind(
        &mut world,
        &mut journal,
        &snapshots,
        Tick(120),
        &mut available_mana,
    );
    assert_eq!(result, Err(ChronoError::InsufficientMana));
    assert_eq!(available_mana, 0);
    assert_eq!(world.current_tick(), Tick(200));
    assert_eq!(world.state_hash(), initial_hash);

    // 2. Insufficient mana: 23 available when 24 is required
    let mut borderline_mana: u32 = 23;
    let result2 = execute_rewind(
        &mut world,
        &mut journal,
        &snapshots,
        Tick(120),
        &mut borderline_mana,
    );
    assert_eq!(result2, Err(ChronoError::InsufficientMana));
    assert_eq!(borderline_mana, 23);
    assert_eq!(world.current_tick(), Tick(200));
    assert_eq!(world.state_hash(), initial_hash);

    Ok(())
}
