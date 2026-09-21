//! TDD Integration Tests for Simulation Step Orchestration & Hero AI.
//!
//! Validates `SPEC-REQ-SIM-001` through `SPEC-REQ-SIM-004` (`docs/specs/08_simulation_ia_heros.md`).

use tomb_of_heroes_core::chrono::memory::{ChronoHero, HeroState};
use tomb_of_heroes_core::config::GameConfig;
use tomb_of_heroes_core::id::LogicId;
use tomb_of_heroes_core::necro::corpse::{Corpse, HeroClass};
use tomb_of_heroes_core::rng::DungeonMasterSeed;
use tomb_of_heroes_core::topology::coordinates::{FloorId, GridCoord, WorldCoord};
use tomb_of_heroes_core::topology::grid::FloorGrid;
use tomb_of_heroes_core::world::LogicWorld;

#[test]
fn test_simulation_step_clock_and_mana_regeneration() {
    let mut world = LogicWorld::new(GameConfig::default(), DungeonMasterSeed(42));
    assert_eq!(world.current_tick().as_u64(), 0);
    let initial_mana = world.mana();

    // Step 10 ticks
    for _ in 0..10 {
        world.step();
    }

    assert_eq!(world.current_tick().as_u64(), 10);
    assert_eq!(world.mana(), initial_mana + 1);
}

#[test]
fn test_hero_terror_accumulation_and_fsm_transition() {
    let mut world = LogicWorld::new(GameConfig::default(), DungeonMasterSeed(123));

    // Place an intact corpse at (5, 5)
    let corpse_id = LogicId(100);
    let source_id = LogicId(101);
    let corpse = Corpse::new(corpse_id, source_id, HeroClass::Warrior, world.config());
    let corpse_cfg = world.config().corpse;
    let _ = world
        .corpses_mut()
        .place_corpse(corpse, GridCoord::new(5, 5), &corpse_cfg);

    // Register a novice hero at (5, 6) adjacent to the corpse
    let hero_id = LogicId(1);
    let hero = ChronoHero::new_ordinary(
        hero_id,
        HeroClass::Rogue,
        WorldCoord::new(FloorId(0), GridCoord::new(5, 6)),
    );
    world.register_hero(hero);

    assert_eq!(
        world.hero(hero_id).map(|h| h.state),
        Some(HeroState::Infiltrating)
    );

    // Step simulation 20 ticks (1 second)
    for _ in 0..20 {
        world.step();
    }

    let hero_after = world.hero(hero_id);
    assert!(hero_after.is_some());
    let Some(h) = hero_after else {
        return;
    };

    // Terror accumulated and state transitioned to Alerted or Fleeing
    assert!(h.terror_bps.0 > 0);
    assert!(matches!(h.state, HeroState::Alerted | HeroState::Fleeing));
}

#[test]
fn test_hero_movement_speed_and_locomotion() {
    let rogue = ChronoHero::new_ordinary(
        LogicId(1),
        HeroClass::Rogue,
        WorldCoord::new(FloorId(0), GridCoord::new(1, 1)),
    );
    let warrior = ChronoHero::new_ordinary(
        LogicId(2),
        HeroClass::Warrior,
        WorldCoord::new(FloorId(0), GridCoord::new(1, 1)),
    );
    let mage = ChronoHero::new_ordinary(
        LogicId(3),
        HeroClass::Mage,
        WorldCoord::new(FloorId(0), GridCoord::new(1, 1)),
    );

    // Base cooldown: Rogue (6 ticks) < Warrior (10 ticks) < Mage (12 ticks)
    assert_eq!(rogue.move_cooldown_ticks(), 6);
    assert_eq!(warrior.move_cooldown_ticks(), 10);
    assert_eq!(mage.move_cooldown_ticks(), 12);
}

#[test]
fn test_trap_trigger_and_death_conversion_during_step() {
    let mut world = LogicWorld::new(GameConfig::default(), DungeonMasterSeed(777));

    // Setup floor 0 grid with walkable tiles from (1, 1) to (1, 3)
    let floor_grid = FloorGrid::new_with_bounds(FloorId(0), GridCoord::new(0, 0), 10, 10, true);
    world.dungeon_mut().add_floor(floor_grid);

    // Arm a hazard/trap at (1, 2)
    let trap_coord = WorldCoord::new(FloorId(0), GridCoord::new(1, 2));
    world.record_hazard(trap_coord);

    // Spawn a fragile hero at (1, 1) with low HP (15 HP)
    let hero_id = LogicId(1);
    let mut hero = ChronoHero::new_ordinary(
        hero_id,
        HeroClass::Warrior,
        WorldCoord::new(FloorId(0), GridCoord::new(1, 1)),
    );
    hero.current_hp = 10; // Lethal against 35 base spike damage
    hero.target_destination = Some(WorldCoord::new(FloorId(0), GridCoord::new(1, 5)));
    world.register_hero(hero);

    let initial_mana = world.mana();

    // Step simulation enough ticks for warrior to step onto (1, 2)
    for _ in 0..25 {
        world.step();
    }

    // Hero died from trap -> removed from living heroes and converted to corpse
    assert!(world.hero(hero_id).is_none());
    assert!(!world.corpses().is_empty());
    // Mana harvested (+20) + natural regeneration (+2 across 25 ticks)
    assert_eq!(world.mana(), initial_mana + 20 + 2);
}
