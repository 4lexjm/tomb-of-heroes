//! # Tomb of Heroes App (`tomb_of_heroes_app`)
//!
//! Bevy 0.15 graphical host and viewport container for *Tomb of Heroes*.
//!
//! Complies with `SPEC-REQ-VIEW-001`, `SPEC-REQ-VIEW-002`, and `SPEC-REQ-ARCH-001`.

pub mod plugin;
pub mod simulation;
pub mod viewport;

pub use plugin::{SimulationPlugin, TombOfHeroesAppPlugin, ViewportPlugin};
pub use simulation::{advance_simulation_system, FixedTickAccumulator, WorldSimulation};
pub use viewport::{
    compute_pixel_perfect_scale, SafeZone, ViewportGeometry, LOGICAL_HEIGHT, LOGICAL_WIDTH_MAX,
    LOGICAL_WIDTH_MIN, LOGICAL_WIDTH_REF,
};

/// Builds and configures the default Bevy application instance.
#[must_use]
pub fn build_app() -> bevy::app::App {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_plugins(TombOfHeroesAppPlugin);
    app
}
