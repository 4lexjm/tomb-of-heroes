use tomb_of_heroes_core::{
    ActionJournal, CoreCommand, GateState, LogicId, Tick, TimedCommand, TrapType,
};

#[test]
fn test_query_commands_in_interval_nominal() {
    let mut journal = ActionJournal::new();

    let cmd10 = TimedCommand {
        tick: Tick(10),
        command_id: LogicId(1),
        payload: CoreCommand::ArmTrap {
            floor: 0,
            x: 1,
            y: 2,
            trap_type: TrapType::Spikes,
        },
    };
    let cmd20_a = TimedCommand {
        tick: Tick(20),
        command_id: LogicId(2),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(10),
        },
    };
    let cmd20_b = TimedCommand {
        tick: Tick(20),
        command_id: LogicId(3),
        payload: CoreCommand::TogglePortcullis {
            gate_id: LogicId(15),
            target_state: GateState::Closed,
        },
    };
    let cmd30 = TimedCommand {
        tick: Tick(30),
        command_id: LogicId(4),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(11),
        },
    };
    let cmd40 = TimedCommand {
        tick: Tick(40),
        command_id: LogicId(5),
        payload: CoreCommand::ArmTrap {
            floor: 0,
            x: 3,
            y: 4,
            trap_type: TrapType::Pitfall,
        },
    };
    let cmd50 = TimedCommand {
        tick: Tick(50),
        command_id: LogicId(6),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(12),
        },
    };

    assert!(journal.record_command(cmd10.clone()).is_ok());
    assert!(journal.record_command(cmd20_a.clone()).is_ok());
    assert!(journal.record_command(cmd20_b.clone()).is_ok());
    assert!(journal.record_command(cmd30.clone()).is_ok());
    assert!(journal.record_command(cmd40.clone()).is_ok());
    assert!(journal.record_command(cmd50.clone()).is_ok());

    // Query interval [20, 40]
    let result = journal.commands_in_interval(Tick(20), Tick(40));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0], cmd20_a);
    assert_eq!(result[1], cmd20_b);
    assert_eq!(result[2], cmd30);
    assert_eq!(result[3], cmd40);
}

#[test]
fn test_query_commands_exact_tick_single_and_multiple() {
    let mut journal = ActionJournal::new();

    let cmd1 = TimedCommand {
        tick: Tick(100),
        command_id: LogicId(1),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(1),
        },
    };
    let cmd2 = TimedCommand {
        tick: Tick(100),
        command_id: LogicId(2),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(2),
        },
    };
    let cmd3 = TimedCommand {
        tick: Tick(200),
        command_id: LogicId(3),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(3),
        },
    };

    assert!(journal.record_command(cmd1.clone()).is_ok());
    assert!(journal.record_command(cmd2.clone()).is_ok());
    assert!(journal.record_command(cmd3.clone()).is_ok());

    // Single tick query [100, 100] returns both commands at tick 100
    let res_100 = journal.commands_in_interval(Tick(100), Tick(100));
    assert_eq!(res_100.len(), 2);
    assert_eq!(res_100[0], cmd1);
    assert_eq!(res_100[1], cmd2);

    // Single tick query [200, 200] returns 1 command
    let res_200 = journal.commands_in_interval(Tick(200), Tick(200));
    assert_eq!(res_200.len(), 1);
    assert_eq!(res_200[0], cmd3);
}

#[test]
fn test_query_commands_empty_interval() {
    let mut journal = ActionJournal::new();

    let cmd1 = TimedCommand {
        tick: Tick(10),
        command_id: LogicId(1),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(1),
        },
    };
    let cmd2 = TimedCommand {
        tick: Tick(30),
        command_id: LogicId(2),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(2),
        },
    };

    assert!(journal.record_command(cmd1).is_ok());
    assert!(journal.record_command(cmd2).is_ok());

    // Interval [15, 25] has no commands
    let res = journal.commands_in_interval(Tick(15), Tick(25));
    assert!(res.is_empty());

    // Inverted interval [30, 10] where from > to
    let res_inv = journal.commands_in_interval(Tick(30), Tick(10));
    assert!(res_inv.is_empty());

    // Interval before any commands [0, 5]
    let res_before = journal.commands_in_interval(Tick(0), Tick(5));
    assert!(res_before.is_empty());

    // Interval after all commands [50, 100]
    let res_after = journal.commands_in_interval(Tick(50), Tick(100));
    assert!(res_after.is_empty());
}

#[test]
fn test_query_commands_full_span() {
    let mut journal = ActionJournal::new();

    let cmd1 = TimedCommand {
        tick: Tick(5),
        command_id: LogicId(10),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(1),
        },
    };
    let cmd2 = TimedCommand {
        tick: Tick(95),
        command_id: LogicId(20),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(2),
        },
    };

    assert!(journal.record_command(cmd1.clone()).is_ok());
    assert!(journal.record_command(cmd2.clone()).is_ok());

    let res = journal.commands_in_interval(Tick(0), Tick(100));
    assert_eq!(res.len(), 2);
    assert_eq!(res[0], cmd1);
    assert_eq!(res[1], cmd2);
}
