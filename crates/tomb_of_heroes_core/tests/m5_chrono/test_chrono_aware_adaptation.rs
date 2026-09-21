use tomb_of_heroes_core::{
    execute_rewind, ActionJournal, ChronoHero, CoreCommand, DungeonMasterSeed, FloorId, GameConfig,
    GridCoord, HeroClass, LogicId, LogicWorld, SnapshotRingBuffer, Tick, TimedCommand, TrapType,
    WorldCoord,
};

#[test]
fn test_chrono_aware_hero_retains_anticipated_hazard() -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(777);
    let mut world = LogicWorld::new(config.clone(), seed);
    let mut journal = ActionJournal::new();
    let mut snapshots = SnapshotRingBuffer::new(config.chrono.ring_buffer_capacity);

    // Register two heroes in the world at tick 0:
    // Hero 1: Temporal Inquisitor with chrono-awareness
    let aware_hero = ChronoHero::new(
        LogicId(101),
        HeroClass::Paladin,
        true, // has_chrono_awareness = true
        WorldCoord::new(FloorId(0), GridCoord::new(2, 2)),
    );
    // Hero 2: Ordinary warrior without chrono-awareness
    let ordinary_hero = ChronoHero::new(
        LogicId(102),
        HeroClass::Warrior,
        false, // has_chrono_awareness = false
        WorldCoord::new(FloorId(0), GridCoord::new(2, 3)),
    );
    world.register_hero(aware_hero);
    world.register_hero(ordinary_hero);

    // Advance to tick 100 and capture snapshot
    for _ in 1..=100 {
        world.step();
        snapshots.capture_if_due(&world, &config.chrono);
    }
    assert_eq!(snapshots.len(), 1);

    // In the future timeline (between tick 101 and 150):
    // A deadly blade trap is armed at WorldCoord(0, 5, 5) at tick 130
    let hazard_coord = WorldCoord::new(FloorId(0), GridCoord::new(5, 5));
    for _ in 101..=150 {
        world.step();
        if world.current_tick() == Tick(130) {
            let cmd = TimedCommand {
                tick: Tick(130),
                command_id: LogicId(1),
                payload: CoreCommand::ArmTrap {
                    floor: 0,
                    x: 5,
                    y: 5,
                    trap_type: TrapType::Blade,
                },
            };
            journal.record_command(cmd.clone())?;
            world.apply_command(&cmd.payload);
        }
    }

    assert_eq!(world.current_tick(), Tick(150));

    // Rewind back to tick 100 (discarding the future from 101 to 150)
    let mut available_mana: u32 = 100;
    execute_rewind(
        &mut world,
        &mut journal,
        &snapshots,
        Tick(100),
        &mut available_mana,
    )?;

    assert_eq!(world.current_tick(), Tick(100));

    // Verify Hero 1 (Aware): retained the erased future trap tile in anticipated_hazards
    let hero1 = world.hero(LogicId(101)).ok_or("Hero 101 not found")?;
    assert!(hero1.has_chrono_awareness);
    assert!(hero1
        .chrono_memory
        .anticipated_hazards
        .contains(&hazard_coord));
    assert!(hero1.avoids_tile(&hazard_coord));

    // Verify Hero 2 (Ordinary): memory was restored to Tick 100; does NOT anticipate hazard
    let hero2 = world.hero(LogicId(102)).ok_or("Hero 102 not found")?;
    assert!(!hero2.has_chrono_awareness);
    assert!(!hero2
        .chrono_memory
        .anticipated_hazards
        .contains(&hazard_coord));
    assert!(!hero2.avoids_tile(&hazard_coord));

    // Verify paradoxical anxiety on non-aware ally (+500 BPS terror)
    assert_eq!(hero2.terror_bps, config.chrono.paradox_terror_bps);

    Ok(())
}
