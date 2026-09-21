use tomb_of_heroes_core::{
    execute_rewind, ActionJournal, CoreCommand, DungeonMasterSeed, GameConfig, LogicId, LogicWorld,
    SnapshotRingBuffer, Tick, TimedCommand, TrapType,
};

#[test]
fn test_future_commands_truncated_after_rewind() -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(333);
    let mut world = LogicWorld::new(config.clone(), seed);
    let mut journal = ActionJournal::new();
    let mut snapshots = SnapshotRingBuffer::new(config.chrono.ring_buffer_capacity);

    for _ in 1..=100 {
        world.step();
        snapshots.capture_if_due(&world, &config.chrono);
    }

    // Commands before and at tick 120
    let cmd1 = TimedCommand {
        tick: Tick(105),
        command_id: LogicId(1),
        payload: CoreCommand::ArmTrap {
            floor: 0,
            x: 1,
            y: 1,
            trap_type: TrapType::Blade,
        },
    };
    let cmd2 = TimedCommand {
        tick: Tick(115),
        command_id: LogicId(2),
        payload: CoreCommand::ArmTrap {
            floor: 0,
            x: 2,
            y: 2,
            trap_type: TrapType::Fire,
        },
    };
    let cmd3 = TimedCommand {
        tick: Tick(120),
        command_id: LogicId(3),
        payload: CoreCommand::ArmTrap {
            floor: 0,
            x: 3,
            y: 3,
            trap_type: TrapType::Spikes,
        },
    };

    // Commands after tick 120 (in the future to be erased)
    let cmd4 = TimedCommand {
        tick: Tick(130),
        command_id: LogicId(4),
        payload: CoreCommand::ArmTrap {
            floor: 0,
            x: 4,
            y: 4,
            trap_type: TrapType::Pitfall,
        },
    };
    let cmd5 = TimedCommand {
        tick: Tick(150),
        command_id: LogicId(5),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(10),
        },
    };
    let cmd6 = TimedCommand {
        tick: Tick(180),
        command_id: LogicId(6),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(11),
        },
    };

    journal.record_command(cmd1.clone())?;
    journal.record_command(cmd2.clone())?;
    journal.record_command(cmd3.clone())?;
    journal.record_command(cmd4.clone())?;
    journal.record_command(cmd5.clone())?;
    journal.record_command(cmd6.clone())?;

    for _ in 101..=200 {
        world.step();
        snapshots.capture_if_due(&world, &config.chrono);
    }

    assert_eq!(journal.len(), 6);
    assert_eq!(journal.last_command().map(|c| c.tick), Some(Tick(180)));

    let mut available_mana: u32 = 100;
    execute_rewind(
        &mut world,
        &mut journal,
        &snapshots,
        Tick(120),
        &mut available_mana,
    )?;

    // Journal must now only contain commands up to Tick(120)
    assert_eq!(journal.len(), 3);
    assert_eq!(journal.last_command().map(|c| c.tick), Some(Tick(120)));

    let future_commands = journal.commands_in_interval(Tick(121), Tick(300));
    assert!(future_commands.is_empty());

    // Preserved commands
    let remaining = journal.as_slice();
    assert_eq!(remaining[0], cmd1);
    assert_eq!(remaining[1], cmd2);
    assert_eq!(remaining[2], cmd3);

    // Truncated command IDs must be reusable in future timelines
    assert!(!journal.contains_command_id(LogicId(4)));
    assert!(!journal.contains_command_id(LogicId(5)));
    assert!(!journal.contains_command_id(LogicId(6)));

    Ok(())
}
