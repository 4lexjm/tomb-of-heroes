use tomb_of_heroes_core::{
    ActionJournal, CoreCommand, GateState, JournalError, LogicId, Tick, TimedCommand, TrapType,
    UndeadKind,
};

#[test]
fn test_record_strictly_increasing_ticks() {
    let mut journal = ActionJournal::new();
    assert!(journal.is_empty());
    assert_eq!(journal.len(), 0);

    let cmd1 = TimedCommand {
        tick: Tick(10),
        command_id: LogicId(1),
        payload: CoreCommand::ArmTrap {
            floor: 1,
            x: 5,
            y: 10,
            trap_type: TrapType::Blade,
        },
    };
    let cmd2 = TimedCommand {
        tick: Tick(20),
        command_id: LogicId(2),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(50),
        },
    };
    let cmd3 = TimedCommand {
        tick: Tick(30),
        command_id: LogicId(3),
        payload: CoreCommand::TogglePortcullis {
            gate_id: LogicId(100),
            target_state: GateState::Open,
        },
    };

    assert!(journal.record_command(cmd1.clone()).is_ok());
    assert_eq!(journal.len(), 1);
    assert!(!journal.is_empty());

    assert!(journal.record_command(cmd2.clone()).is_ok());
    assert_eq!(journal.len(), 2);

    assert!(journal.record_command(cmd3.clone()).is_ok());
    assert_eq!(journal.len(), 3);

    let recorded = journal.as_slice();
    assert_eq!(recorded.len(), 3);
    assert_eq!(recorded[0], cmd1);
    assert_eq!(recorded[1], cmd2);
    assert_eq!(recorded[2], cmd3);
}

#[test]
fn test_record_same_tick_increasing_command_id() {
    let mut journal = ActionJournal::new();

    let cmd1 = TimedCommand {
        tick: Tick(50),
        command_id: LogicId(10),
        payload: CoreCommand::RaiseCorpse {
            corpse_id: LogicId(200),
            target_undead: UndeadKind::SkeletonGuardian,
        },
    };
    let cmd2 = TimedCommand {
        tick: Tick(50),
        command_id: LogicId(11),
        payload: CoreCommand::ChannelTemporalRewind {
            target_tick: Tick(25),
        },
    };
    let cmd3 = TimedCommand {
        tick: Tick(50),
        command_id: LogicId(15),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(60),
        },
    };

    assert!(journal.record_command(cmd1.clone()).is_ok());
    assert!(journal.record_command(cmd2.clone()).is_ok());
    assert!(journal.record_command(cmd3.clone()).is_ok());

    assert_eq!(journal.len(), 3);
    let recorded = journal.as_slice();
    assert_eq!(recorded[0].command_id, LogicId(10));
    assert_eq!(recorded[1].command_id, LogicId(11));
    assert_eq!(recorded[2].command_id, LogicId(15));
}

#[test]
fn test_reject_anti_chronological_tick() {
    let mut journal = ActionJournal::new();

    let cmd1 = TimedCommand {
        tick: Tick(100),
        command_id: LogicId(1),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(5),
        },
    };
    assert!(journal.record_command(cmd1).is_ok());

    // Attempt to insert command with tick earlier than last recorded
    let invalid_cmd = TimedCommand {
        tick: Tick(99),
        command_id: LogicId(2),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(6),
        },
    };

    let result = journal.record_command(invalid_cmd);
    assert!(result.is_err());
    assert_eq!(
        result,
        Err(JournalError::UnorderedCommand {
            last_tick: Tick(100),
            last_id: LogicId(1),
            attempted_tick: Tick(99),
            attempted_id: LogicId(2),
        })
    );
    assert_eq!(journal.len(), 1);
}

#[test]
fn test_reject_same_tick_non_increasing_command_id() {
    let mut journal = ActionJournal::new();

    let cmd1 = TimedCommand {
        tick: Tick(100),
        command_id: LogicId(20),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(5),
        },
    };
    assert!(journal.record_command(cmd1).is_ok());

    // Attempt to insert command with same tick but smaller command_id
    let invalid_cmd = TimedCommand {
        tick: Tick(100),
        command_id: LogicId(19),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(6),
        },
    };

    let result = journal.record_command(invalid_cmd);
    assert!(result.is_err());
    assert_eq!(
        result,
        Err(JournalError::UnorderedCommand {
            last_tick: Tick(100),
            last_id: LogicId(20),
            attempted_tick: Tick(100),
            attempted_id: LogicId(19),
        })
    );
    assert_eq!(journal.len(), 1);
}

#[test]
fn test_reject_duplicate_command_id() {
    let mut journal = ActionJournal::new();

    let cmd1 = TimedCommand {
        tick: Tick(50),
        command_id: LogicId(42),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(5),
        },
    };
    assert!(journal.record_command(cmd1).is_ok());

    // Same tick, duplicate command_id
    let dup_same_tick = TimedCommand {
        tick: Tick(50),
        command_id: LogicId(42),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(6),
        },
    };
    let result_same_tick = journal.record_command(dup_same_tick);
    assert_eq!(
        result_same_tick,
        Err(JournalError::DuplicateCommandId { id: LogicId(42) })
    );

    // Later tick, duplicate command_id
    let dup_later_tick = TimedCommand {
        tick: Tick(60),
        command_id: LogicId(42),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(7),
        },
    };
    let result_later_tick = journal.record_command(dup_later_tick);
    assert_eq!(
        result_later_tick,
        Err(JournalError::DuplicateCommandId { id: LogicId(42) })
    );

    assert_eq!(journal.len(), 1);
}
