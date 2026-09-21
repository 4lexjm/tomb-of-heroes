use tomb_of_heroes_core::{DungeonMasterSeed, GameConfig, LogicClock, LogicWorld, Tick};

#[test]
fn test_tick_initialization_and_advancement() {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(42);
    let mut world = LogicWorld::new(config, seed);

    // Initial logical clock must start at Tick(0)
    assert_eq!(world.current_tick(), Tick(0));
    assert_eq!(world.current_tick(), Tick::ZERO);

    // Execute 10 simulation steps
    for _ in 0..10 {
        world.step();
    }

    // Strict assertion on tick advancement
    assert_eq!(world.current_tick(), Tick(10));
}

#[test]
fn test_logic_clock_step() {
    let mut clock = LogicClock::new();
    assert_eq!(clock.current_tick(), Tick(0));

    for expected in 1..=10 {
        let tick = clock.step();
        assert_eq!(tick, Tick(expected));
        assert_eq!(clock.current_tick(), Tick(expected));
    }
}

#[test]
fn test_tick_traits_and_arithmetic() {
    let t0 = Tick(0);
    let t5 = Tick(5);
    let t10 = Tick(10);

    assert!(t0 < t5);
    assert!(t5 < t10);
    assert_eq!(t5.as_u64(), 5);
    assert_eq!(t5.next(), Tick(6));

    let mut t_mut = t5;
    t_mut.advance();
    assert_eq!(t_mut, Tick(6));

    assert_eq!(t5 + 5, Tick(10));
    assert_eq!(t10 - t5, 5);

    // Display
    assert_eq!(format!("{t5}"), "Tick(5)");

    // Serde roundtrip
    let serialized = serde_json::to_string(&t10);
    assert!(serialized.is_ok());
    if let Ok(json) = serialized {
        let deserialized: Result<Tick, _> = serde_json::from_str(&json);
        assert_eq!(deserialized.ok(), Some(t10));
    }
}
