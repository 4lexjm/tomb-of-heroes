//! Bevy plugin architecture module.

use bevy::app::{App, Plugin, Update};

use crate::camera::PixelCameraPlugin;
use crate::render::{EntityRenderPlugin, TilemapRenderPlugin};
use crate::simulation::{advance_simulation_system, FixedTickAccumulator, WorldSimulation};
use crate::ui::HudPlugin;
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
            let mut sim = WorldSimulation::default();
            populate_default_scenario(&mut sim);
            app.insert_resource(sim);
        }
        app.add_systems(Update, advance_simulation_system);
    }
}

/// Populates default heroes and fallen adventurers for visual demonstration and inspection.
fn populate_default_scenario(sim: &mut WorldSimulation) {
    use tomb_of_heroes_core::chrono::memory::ChronoHero;
    use tomb_of_heroes_core::necro::{Corpse, CorpseState, HeroClass};
    use tomb_of_heroes_core::topology::{FloorId, GridCoord, WorldCoord};

    // Hero 1: Frontline Warrior
    if let Ok(id) = sim.allocate_id() {
        let hero = ChronoHero::new_ordinary(
            id,
            HeroClass::Warrior,
            WorldCoord::new(FloorId(0), GridCoord::new(0, -1)),
        );
        sim.register_hero(hero);
    }
    // Hero 2: Chrono-aware Mage
    if let Ok(id) = sim.allocate_id() {
        let hero = ChronoHero::new_aware(
            id,
            HeroClass::Mage,
            WorldCoord::new(FloorId(0), GridCoord::new(1, 0)),
        );
        sim.register_hero(hero);
    }

    let corpse_cfg = sim.config().corpse;

    // Corpse 1: Intact Rogue corpse
    if let Ok(id) = sim.allocate_id() {
        if let Ok(src) = sim.allocate_id() {
            let mut corpse = Corpse::new(id, src, HeroClass::Rogue, sim.config());
            corpse.state = CorpseState::Intact;
            let _ = sim
                .corpses_mut()
                .place_corpse(corpse, GridCoord::new(-2, 2), &corpse_cfg);
        }
    }
    // Corpse 2: Damaged Cleric corpse
    if let Ok(id) = sim.allocate_id() {
        if let Ok(src) = sim.allocate_id() {
            let mut corpse = Corpse::new(id, src, HeroClass::Cleric, sim.config());
            corpse.state = CorpseState::Damaged;
            corpse.structural_hp = 20;
            let _ = sim
                .corpses_mut()
                .place_corpse(corpse, GridCoord::new(2, 2), &corpse_cfg);
        }
    }
    // Corpse 3: Bones / Destroyed
    if let Ok(id) = sim.allocate_id() {
        if let Ok(src) = sim.allocate_id() {
            let mut corpse = Corpse::new(id, src, HeroClass::Warrior, sim.config());
            corpse.state = CorpseState::Destroyed;
            corpse.structural_hp = 0;
            let _ = sim
                .corpses_mut()
                .place_corpse(corpse, GridCoord::new(5, -1), &corpse_cfg);
        }
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

/// Root application plugin combining camera, rendering, HUD, and simulation systems.
#[derive(Debug, Default, Clone, Copy)]
pub struct TombOfHeroesAppPlugin;

impl Plugin for TombOfHeroesAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SimulationPlugin,
            ViewportPlugin,
            PixelCameraPlugin,
            TilemapRenderPlugin,
            EntityRenderPlugin,
            HudPlugin,
        ));
    }
}
