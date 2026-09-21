//! Bevy plugin architecture module.

use bevy::app::{App, Plugin, Update};

use crate::simulation::{advance_simulation_system, FixedTickAccumulator, WorldSimulation};
use crate::viewport::ViewportGeometry;

/// Plugin managing fixed-tick simulation integration.
#[derive(Debug, Default, Clone, Copy)]
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<FixedTickAccumulator>() {
            app.init_resource::<FixedTickAccumulator>();
        }
        if !app.world().contains_resource::<WorldSimulation>() {
            app.init_resource::<WorldSimulation>();
        }
        app.add_systems(Update, advance_simulation_system);
    }
}

/// Plugin managing viewport scaling and letterboxing.
#[derive(Debug, Default, Clone, Copy)]
pub struct ViewportPlugin;

impl Plugin for ViewportPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<ViewportGeometry>() {
            app.init_resource::<ViewportGeometry>();
        }
    }
}

/// Root application plugin combining simulation and viewport systems.
#[derive(Debug, Default, Clone, Copy)]
pub struct TombOfHeroesAppPlugin;

impl Plugin for TombOfHeroesAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SimulationPlugin, ViewportPlugin));
    }
}
