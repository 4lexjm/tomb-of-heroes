use tomb_of_heroes_core::{
    execute_rewind, ActionJournal, CoreCommand, DungeonMasterSeed, GameConfig, LogicId, LogicWorld,
    SnapshotRingBuffer, Tick, TimedCommand, TrapType,
};

#[test]
fn test_deterministic_rewind_restores_exact_state_hash() -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(12345);
    let mut world = LogicWorld::new(config.clone(), seed);
    let mut journal = ActionJournal::new();
    let mut snapshots = SnapshotRingBuffer::new(config.chrono.ring_buffer_capacity);

    // Simulate up to tick 100 with periodic snapshots
    for _ in 1..=100 {
        world.step();
        snapshots.capture_if_due(&world, &config.chrono);
    }
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots.oldest_tick(), Some(Tick(100)));

    // Advance to tick 120 while recording and applying commands
    for _ in 101..=120 {
        world.step();
        if world.current_tick() == Tick(105) {
            let cmd = TimedCommand {
                tick: Tick(105),
                command_id: LogicId(1),
                payload: CoreCommand::ArmTrap {
                    floor: 0,
                    x: 3,
                    y: 4,
                    trap_type: TrapType::Blade,
                },
            };
            journal.record_command(cmd.clone())?;
            world.apply_command(&cmd.payload);
        }
        if world.current_tick() == Tick(115) {
            let cmd = TimedCommand {
                tick: Tick(115),
                command_id: LogicId(2),
                payload: CoreCommand::TriggerTrapManual {
                    trap_id: LogicId(1),
                },
            };
            journal.record_command(cmd.clone())?;
            world.apply_command(&cmd.payload);
        }
    }

    assert_eq!(world.current_tick(), Tick(120));
    let original_hash_120 = world.state_hash();

    // Advance to tick 200 with further commands and snapshot at 200
    for _ in 121..=200 {
        world.step();
        snapshots.capture_if_due(&world, &config.chrono);

        if world.current_tick() == Tick(135) {
            let cmd = TimedCommand {
                tick: Tick(135),
                command_id: LogicId(3),
                payload: CoreCommand::ArmTrap {
                    floor: 0,
                    x: 7,
                    y: 8,
                    trap_type: TrapType::Fire,
                },
            };
            journal.record_command(cmd.clone())?;
            world.apply_command(&cmd.payload);
        }
    }

    assert_eq!(world.current_tick(), Tick(200));
    assert_ne!(world.state_hash(), original_hash_120);

    // Rewind back to tick 120
    let mut available_mana: u32 = 100;
    execute_rewind(
        &mut world,
        &mut journal,
        &snapshots,
        Tick(120),
        &mut available_mana,
    )?;

    // State at tick 120 must be bit-for-bit identical to the recorded state hash
    assert_eq!(world.current_tick(), Tick(120));
    assert_eq!(world.state_hash(), original_hash_120);

    Ok(())
}
