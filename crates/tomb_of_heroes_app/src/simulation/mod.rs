//! Simulation clock and world integration module.
//!
//! Enforces `SPEC-REQ-ARCH-001` fixed-tick progression.

use std::ops::{Deref, DerefMut};
use std::time::Duration;

use bevy::ecs::prelude::{Res, ResMut, Resource};
use bevy::time::Time;
use tomb_of_heroes_core::{DungeonMasterSeed, GameConfig, LogicWorld};

/// Accumulates elapsed real-time and steps discrete fixed ticks.
///
/// Implements `SPEC-REQ-ARCH-001`.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct FixedTickAccumulator {
    accumulated: Duration,
    tick_duration: Duration,
    max_catchup_ticks: u32,
}

impl FixedTickAccumulator {
    /// Creates a new accumulator with custom tick step duration and anti-death-spiral catch-up cap.
    #[must_use]
    pub fn new(tick_duration: Duration, max_catchup_ticks: u32) -> Self {
        Self {
            accumulated: Duration::ZERO,
            tick_duration,
            max_catchup_ticks,
        }
    }

    /// Constructs an accumulator configured from `GameConfig`.
    #[must_use]
    pub fn from_config(config: &GameConfig) -> Self {
        Self {
            accumulated: Duration::ZERO,
            tick_duration: Duration::from_millis(config.tick.duration_ms as u64),
            max_catchup_ticks: config.tick.max_catchup_ticks_per_frame,
        }
    }

    /// Returns the accumulated residue duration awaiting simulation.
    #[inline]
    #[must_use]
    pub const fn accumulated(&self) -> Duration {
        self.accumulated
    }

    /// Returns the discrete tick interval duration.
    #[inline]
    #[must_use]
    pub const fn tick_duration(&self) -> Duration {
        self.tick_duration
    }

    /// Returns the maximum allowable catch-up ticks per frame.
    #[inline]
    #[must_use]
    pub const fn max_catchup_ticks(&self) -> u32 {
        self.max_catchup_ticks
    }

    /// Resets the accumulator back to zero.
    pub fn reset(&mut self) {
        self.accumulated = Duration::ZERO;
    }

    /// Steps the accumulator by delta duration.
    ///
    /// Returns the number of discrete ticks to execute (0..=max_catchup_ticks)
    /// and strictly retains the remaining residue below `tick_duration`.
    pub fn step_accumulator(&mut self, delta: Duration) -> u32 {
        self.accumulated = self.accumulated.saturating_add(delta);
        let tick_micros = self.tick_duration.as_micros();
        if tick_micros == 0 {
            return 0;
        }
        let accum_micros = self.accumulated.as_micros();
        let potential_ticks = (accum_micros / tick_micros) as u32;
        let ticks = potential_ticks.min(self.max_catchup_ticks);
        let residue_micros = accum_micros % tick_micros;
        self.accumulated = Duration::from_micros(residue_micros as u64);
        ticks
    }
}

impl Default for FixedTickAccumulator {
    fn default() -> Self {
        let config = GameConfig::default();
        Self::from_config(&config)
    }
}

/// Bevy Resource wrapper around the deterministic headless simulation `LogicWorld`.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct WorldSimulation {
    world: LogicWorld,
}

impl WorldSimulation {
    /// Wraps an existing `LogicWorld`.
    #[must_use]
    pub fn new(world: LogicWorld) -> Self {
        Self { world }
    }

    /// Initializes a `LogicWorld` from gameplay configuration and PRNG master seed.
    #[must_use]
    pub fn from_config_and_seed(config: GameConfig, seed: DungeonMasterSeed) -> Self {
        Self {
            world: LogicWorld::new(config, seed),
        }
    }

    /// Returns an immutable reference to the inner `LogicWorld`.
    #[inline]
    #[must_use]
    pub const fn world(&self) -> &LogicWorld {
        &self.world
    }

    /// Returns a mutable reference to the inner `LogicWorld`.
    #[inline]
    pub fn world_mut(&mut self) -> &mut LogicWorld {
        &mut self.world
    }

    /// Steps the inner deterministic simulation by exactly one fixed tick.
    #[inline]
    pub fn step(&mut self) {
        self.world.step();
    }
}

impl Deref for WorldSimulation {
    type Target = LogicWorld;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl DerefMut for WorldSimulation {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world
    }
}

impl Default for WorldSimulation {
    fn default() -> Self {
        Self::from_config_and_seed(GameConfig::default(), DungeonMasterSeed(0))
    }
}

/// Bevy system advancing `WorldSimulation` by consumed ticks from `FixedTickAccumulator`.
pub fn advance_simulation_system(
    time: Res<Time>,
    mut accumulator: ResMut<FixedTickAccumulator>,
    mut simulation: ResMut<WorldSimulation>,
) {
    let delta = time.delta();
    let ticks = accumulator.step_accumulator(delta);
    for _ in 0..ticks {
        simulation.step();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tomb_of_heroes_core::time::Tick;

    #[test]
    fn test_accumulator_reset() {
        let mut acc = FixedTickAccumulator::new(Duration::from_millis(50), 5);
        acc.step_accumulator(Duration::from_millis(40));
        assert_eq!(acc.accumulated(), Duration::from_millis(40));
        acc.reset();
        assert_eq!(acc.accumulated(), Duration::ZERO);
    }

    #[test]
    fn test_world_simulation_deref_and_step() {
        let mut sim = WorldSimulation::default();
        assert_eq!(sim.current_tick(), Tick(0));
        sim.step();
        assert_eq!(sim.current_tick(), Tick(1));
        sim.world_mut().step();
        assert_eq!(sim.world().current_tick(), Tick(2));
    }
}
