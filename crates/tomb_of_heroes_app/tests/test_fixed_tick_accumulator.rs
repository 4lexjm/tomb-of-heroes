use std::time::Duration;
use tomb_of_heroes_app::simulation::FixedTickAccumulator;
use tomb_of_heroes_core::GameConfig;

#[test]
fn test_fixed_tick_accumulator_nominal_and_spiral_of_death() {
    let config = GameConfig::default();
    let mut accumulator = FixedTickAccumulator::from_config(&config);

    assert_eq!(accumulator.tick_duration(), Duration::from_millis(50));
    assert_eq!(accumulator.max_catchup_ticks(), 5);
    assert_eq!(accumulator.accumulated(), Duration::ZERO);

    // 1. 49 ms -> 0 tick, remains 49 ms
    let ticks_49 = accumulator.step_accumulator(Duration::from_millis(49));
    assert_eq!(ticks_49, 0);
    assert_eq!(accumulator.accumulated(), Duration::from_millis(49));

    // 2. +51 ms (total accumulated 100 ms) -> 2 ticks, 0 ms remaining
    let ticks_51 = accumulator.step_accumulator(Duration::from_millis(51));
    assert_eq!(ticks_51, 2);
    assert_eq!(accumulator.accumulated(), Duration::ZERO);

    // 3. 500 ms (10 potential ticks) -> strictly capped at 5 ticks
    // Residue must be strictly below 50 ms
    let ticks_500 = accumulator.step_accumulator(Duration::from_millis(500));
    assert_eq!(ticks_500, 5);
    assert!(accumulator.accumulated() < Duration::from_millis(50));
    assert_eq!(accumulator.accumulated(), Duration::ZERO);

    // 4. Test with non-multiple overflow (e.g. 523 ms from 0 ms)
    let ticks_523 = accumulator.step_accumulator(Duration::from_millis(523));
    assert_eq!(ticks_523, 5);
    assert_eq!(accumulator.accumulated(), Duration::from_millis(23));
    assert!(accumulator.accumulated() < Duration::from_millis(50));
}
