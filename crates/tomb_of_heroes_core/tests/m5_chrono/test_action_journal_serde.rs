use tomb_of_heroes_core::{
    ActionJournal, CoreCommand, GateState, LogicId, Tick, TimedCommand, TrapType, UndeadKind,
};

#[test]
fn test_action_journal_serde_roundtrip_all_variants() {
    let mut journal = ActionJournal::new();

    let commands = vec![
        TimedCommand {
            tick: Tick(1),
            command_id: LogicId(1),
            payload: CoreCommand::ArmTrap {
                floor: 0,
                x: -5,
                y: 12,
                trap_type: TrapType::Blade,
            },
        },
        TimedCommand {
            tick: Tick(2),
            command_id: LogicId(2),
            payload: CoreCommand::ArmTrap {
                floor: 1,
                x: 100,
                y: -50,
                trap_type: TrapType::Spikes,
            },
        },
        TimedCommand {
            tick: Tick(3),
            command_id: LogicId(3),
            payload: CoreCommand::ArmTrap {
                floor: 2,
                x: 0,
                y: 0,
                trap_type: TrapType::Pitfall,
            },
        },
        TimedCommand {
            tick: Tick(4),
            command_id: LogicId(4),
            payload: CoreCommand::ArmTrap {
                floor: 3,
                x: 3,
                y: 7,
                trap_type: TrapType::Fire,
            },
        },
        TimedCommand {
            tick: Tick(5),
            command_id: LogicId(5),
            payload: CoreCommand::ArmTrap {
                floor: 4,
                x: 8,
                y: 9,
                trap_type: TrapType::PoisonGas,
            },
        },
        TimedCommand {
            tick: Tick(10),
            command_id: LogicId(6),
            payload: CoreCommand::TriggerTrapManual {
                trap_id: LogicId(99),
            },
        },
        TimedCommand {
            tick: Tick(20),
            command_id: LogicId(7),
            payload: CoreCommand::TogglePortcullis {
                gate_id: LogicId(101),
                target_state: GateState::Open,
            },
        },
        TimedCommand {
            tick: Tick(20),
            command_id: LogicId(8),
            payload: CoreCommand::TogglePortcullis {
                gate_id: LogicId(102),
                target_state: GateState::Closed,
            },
        },
        TimedCommand {
            tick: Tick(20),
            command_id: LogicId(9),
            payload: CoreCommand::TogglePortcullis {
                gate_id: LogicId(103),
                target_state: GateState::Locked,
            },
        },
        TimedCommand {
            tick: Tick(30),
            command_id: LogicId(10),
            payload: CoreCommand::RaiseCorpse {
                corpse_id: LogicId(201),
                target_undead: UndeadKind::SkeletonGuardian,
            },
        },
        TimedCommand {
            tick: Tick(30),
            command_id: LogicId(11),
            payload: CoreCommand::RaiseCorpse {
                corpse_id: LogicId(202),
                target_undead: UndeadKind::FleshWallZombie,
            },
        },
        TimedCommand {
            tick: Tick(30),
            command_id: LogicId(12),
            payload: CoreCommand::RaiseCorpse {
                corpse_id: LogicId(203),
                target_undead: UndeadKind::FallenSoulSpectre,
            },
        },
        TimedCommand {
            tick: Tick(100),
            command_id: LogicId(13),
            payload: CoreCommand::ChannelTemporalRewind {
                target_tick: Tick(50),
            },
        },
    ];

    for cmd in commands {
        assert!(journal.record_command(cmd).is_ok());
    }

    assert_eq!(journal.len(), 13);

    // Serialize to JSON
    let serialized = serde_json::to_string(&journal);
    assert!(serialized.is_ok());
    let json = serialized.unwrap_or_default();
    assert!(!json.is_empty());

    // Deserialize from JSON
    let deserialized: Result<ActionJournal, _> = serde_json::from_str(&json);
    assert!(deserialized.is_ok());
    let mut restored = deserialized.unwrap_or_default();

    // Verify strict equality
    assert_eq!(journal, restored);
    assert_eq!(journal.len(), restored.len());
    assert_eq!(journal.as_slice(), restored.as_slice());

    // Verify invariants are maintained on restored journal:
    // 1. Cannot insert duplicate ID
    let dup_res = restored.record_command(TimedCommand {
        tick: Tick(101),
        command_id: LogicId(13),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(1),
        },
    });
    assert!(dup_res.is_err());

    // 2. Cannot insert anti-chronological tick
    let unord_res = restored.record_command(TimedCommand {
        tick: Tick(99),
        command_id: LogicId(100),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(1),
        },
    });
    assert!(unord_res.is_err());

    // 3. Can insert valid subsequent command
    let valid_res = restored.record_command(TimedCommand {
        tick: Tick(101),
        command_id: LogicId(100),
        payload: CoreCommand::TriggerTrapManual {
            trap_id: LogicId(1),
        },
    });
    assert!(valid_res.is_ok());
    assert_eq!(restored.len(), 14);
}
