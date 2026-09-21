use tomb_of_heroes_core::{
    execute_rewind, ActionJournal, ChronoError, DungeonMasterSeed, GameConfig, LogicWorld,
    SnapshotRingBuffer, Tick,
};

#[test]
fn test_rewind_past_oldest_snapshot_returns_snapshot_expired(
) -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(444);
    let mut world = LogicWorld::new(config.clone(), seed);
    let mut journal = ActionJournal::new();
    let mut snapshots = SnapshotRingBuffer::new(config.chrono.ring_buffer_capacity);

    // Run 350 ticks: snapshots at 100, 200, 300
    for _ in 1..=350 {
        world.step();
        snapshots.capture_if_due(&world, &config.chrono);
    }
    assert_eq!(snapshots.oldest_tick(), Some(Tick(100)));

    let initial_hash = world.state_hash();
    let mut available_mana: u32 = 100;

    // Attempt to rewind to tick 50 (older than oldest snapshot at 100)
    let result = execute_rewind(
        &mut world,
        &mut journal,
        &snapshots,
        Tick(50),
        &mut available_mana,
    );

    assert_eq!(result, Err(ChronoError::SnapshotExpired));
    // World state and mana must remain untouched
    assert_eq!(world.current_tick(), Tick(350));
    assert_eq!(world.state_hash(), initial_hash);
    assert_eq!(available_mana, 100);

    Ok(())
}
