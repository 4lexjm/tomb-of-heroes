use bevy::app::App;
use bevy::time::TimeUpdateStrategy;
use bevy::MinimalPlugins;
use std::time::Duration;
use tomb_of_heroes_app::plugin::SimulationPlugin;
use tomb_of_heroes_app::simulation::WorldSimulation;
use tomb_of_heroes_core::time::Tick;

#[test]
fn test_bevy_app_headless_step() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin);

    // Initial frame initializes resources and schedules.
    // In Bevy, the first frame has delta = 0.
    app.update();

    {
        let sim = app.world().resource::<WorldSimulation>();
        assert_eq!(sim.world().current_tick(), Tick(0));
    }

    // 1. Inject 50 ms (nominal 1 tick at 20 Hz)
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        50,
    )));
    app.update();

    {
        let sim = app.world().resource::<WorldSimulation>();
        assert_eq!(sim.world().current_tick(), Tick(1));
    }

    // 2. Inject 100 ms (2 ticks)
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    app.update();

    {
        let sim = app.world().resource::<WorldSimulation>();
        assert_eq!(sim.world().current_tick(), Tick(3));
    }

    // 3. Inject sub-tick duration (30 ms -> 0 ticks, 30 ms retained in accumulator)
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        30,
    )));
    app.update();

    {
        let sim = app.world().resource::<WorldSimulation>();
        assert_eq!(sim.world().current_tick(), Tick(3));
    }

    // 4. Inject 70 ms (30 ms + 70 ms = 100 ms -> 2 ticks)
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        70,
    )));
    app.update();

    {
        let sim = app.world().resource::<WorldSimulation>();
        assert_eq!(sim.world().current_tick(), Tick(5));
    }
}
