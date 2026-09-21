//! # Tomb of Heroes App (`tomb_of_heroes_app`)
//!
//! Bevy 0.15 graphical host and viewport container for *Tomb of Heroes*.
//!
//! Complies with `SPEC-REQ-VIEW-001`, `SPEC-REQ-VIEW-002`, and `SPEC-REQ-ARCH-001`.

use bevy::app::PluginGroup;

pub mod asset_gen;
pub mod camera;
pub mod plugin;
pub mod render;
pub mod simulation;
pub mod ui;
pub mod viewport;

pub use camera::{CameraSettings, PixelCamera, PixelCameraPlugin};
pub use plugin::{SimulationPlugin, TombOfHeroesAppPlugin, ViewportPlugin};
pub use render::{
    grid_to_world, world_to_grid, ActiveFloor, DungeonTopologyResource, EntityRenderPlugin,
    GuildIntelResource, HeroKnowledgeResource, SelectedTile, TilemapRenderPlugin, VisualEntityRef,
    Z_CORPSES, Z_CREATURES, Z_CURSOR, Z_FLOOR, Z_FOG, Z_GROUND_DECORS, Z_OVERLAYS, Z_WALLS,
};
pub use simulation::{advance_simulation_system, FixedTickAccumulator, WorldSimulation};
pub use ui::{touch_btn, HudPlugin, HudState, PlacementTool, MIN_TOUCH_TARGET_SIZE};
pub use viewport::{
    compute_pixel_perfect_scale, SafeZone, ViewportGeometry, LOGICAL_HEIGHT, LOGICAL_WIDTH_MAX,
    LOGICAL_WIDTH_MIN, LOGICAL_WIDTH_REF,
};

/// Builds and configures the Bevy application instance.
///
/// Configures nearest-neighbor texture sampling (`SPEC-REQ-FRONT-001`) and registers
/// tactile `bevy_egui` along with the core application plugins.
#[must_use]
pub fn build_app() -> bevy::app::App {
    let mut app = bevy::app::App::new();
    app.add_plugins(
        bevy::DefaultPlugins
            .set(bevy::render::texture::ImagePlugin::default_nearest())
            .set(bevy::window::WindowPlugin {
                primary_window: Some(bevy::window::Window {
                    title: "Tomb of Heroes".into(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
    );
    app.add_plugins(bevy_egui::EguiPlugin);
    app.add_plugins(TombOfHeroesAppPlugin);
    app
}

/// Mobile entry point (Android NativeActivity and iOS UIKit bootstrapping).
#[bevy::prelude::bevy_main]
#[allow(dead_code)]
fn main() {
    build_app().run();
}
