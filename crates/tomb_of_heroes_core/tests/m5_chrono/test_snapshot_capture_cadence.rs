use tomb_of_heroes_core::{DungeonMasterSeed, GameConfig, LogicWorld, SnapshotRingBuffer, Tick};

#[test]
fn test_snapshot_capture_cadence_at_100_tick_intervals() -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(42);
    let mut world = LogicWorld::new(config.clone(), seed);
    let mut snapshots = SnapshotRingBuffer::new(config.chrono.ring_buffer_capacity);

    // Initial state at Tick(0): no snapshots
    assert_eq!(snapshots.len(), 0);
    assert!(snapshots.is_empty());

    // Execute 350 ticks: snapshots captured every snapshot_interval_ticks (100 ticks)
    for _ in 1..=350 {
        world.step();
        snapshots.capture_if_due(&world, &config.chrono);
    }

    assert_eq!(world.current_tick(), Tick(350));
    assert_eq!(snapshots.len(), 3);

    let recorded = snapshots.as_slice();
    assert_eq!(recorded.len(), 3);
    assert_eq!(recorded[0].tick, Tick(100));
    assert_eq!(recorded[1].tick, Tick(200));
    assert_eq!(recorded[2].tick, Tick(300));

    assert_eq!(snapshots.oldest_tick(), Some(Tick(100)));
    assert_eq!(snapshots.newest_tick(), Some(Tick(300)));

    Ok(())
}

#[test]
fn test_snapshot_ring_buffer_capacity_eviction() -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(99);
    let mut world = LogicWorld::new(config.clone(), seed);
    let mut snapshots = SnapshotRingBuffer::new(config.chrono.ring_buffer_capacity);

    // Capture 14 snapshots (interval = 100 ticks -> 1400 ticks)
    for _ in 1..=1400 {
        world.step();
        snapshots.capture_if_due(&world, &config.chrono);
    }

    // Capacity is 12: ring buffer must retain only the last 12 snapshots
    assert_eq!(snapshots.len(), 12);
    assert_eq!(snapshots.oldest_tick(), Some(Tick(300)));
    assert_eq!(snapshots.newest_tick(), Some(Tick(1400)));

    Ok(())
}
