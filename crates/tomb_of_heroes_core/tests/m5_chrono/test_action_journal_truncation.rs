use tomb_of_heroes_core::{ActionJournal, CoreCommand, LogicId, Tick, TimedCommand, TrapType};

#[test]
fn test_truncate_after_nominal() {
    let mut journal = ActionJournal::new();

    // Insert commands from tick 50 up to tick 200
    for tick_val in [50, 100, 150, 160, 175, 200] {
        let cmd = TimedCommand {
            tick: Tick(tick_val),
            command_id: LogicId(tick_val),
            payload: CoreCommand::TriggerTrapManual {
                trap_id: LogicId(tick_val),
            },
        };
        assert!(journal.record_command(cmd).is_ok());
    }

    assert_eq!(journal.len(), 6);

    // Truncate all commands after Tick(150)
    journal.truncate_after(Tick(150));

    // Verify only commands <= 150 remain
    assert_eq!(journal.len(), 3);
    let remaining = journal.as_slice();
    assert_eq!(remaining[0].tick, Tick(50));
    assert_eq!(remaining[1].tick, Tick(100));
    assert_eq!(remaining[2].tick, Tick(150));

    // Verify that subsequent recorded command at tick 150 with higher id works
    let cmd_rebranch = TimedCommand {
        tick: Tick(150),
        command_id: LogicId(999),
        payload: CoreCommand::ArmTrap {
            floor: 0,
            x: 0,
            y: 0,
            trap_type: TrapType::Fire,
        },
    };
    assert!(journal.record_command(cmd_rebranch).is_ok());
    assert_eq!(journal.len(), 4);

    // Verify that previously truncated command_id (e.g. 160) can be re-recorded in the new timeline
    let cmd_future = TimedCommand {
        tick: Tick(155),
        command_id: LogicId(160),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(160),
        },
    };
    assert!(journal.record_command(cmd_future).is_ok());
    assert_eq!(journal.len(), 5);
}

#[test]
fn test_truncate_after_edge_cases() {
    let mut journal = ActionJournal::new();

    let cmd1 = TimedCommand {
        tick: Tick(10),
        command_id: LogicId(1),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(1),
        },
    };
    let cmd2 = TimedCommand {
        tick: Tick(20),
        command_id: LogicId(2),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(2),
        },
    };

    assert!(journal.record_command(cmd1).is_ok());
    assert!(journal.record_command(cmd2).is_ok());
    assert_eq!(journal.len(), 2);

    // Truncate with tick > max tick -> no-op
    journal.truncate_after(Tick(50));
    assert_eq!(journal.len(), 2);

    // Truncate with tick < min tick -> all cleared
    journal.truncate_after(Tick(5));
    assert_eq!(journal.len(), 0);
    assert!(journal.is_empty());

    // Can record new commands starting from tick 1
    let cmd3 = TimedCommand {
        tick: Tick(6),
        command_id: LogicId(1),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(3),
        },
    };
    assert!(journal.record_command(cmd3).is_ok());
    assert_eq!(journal.len(), 1);
}
