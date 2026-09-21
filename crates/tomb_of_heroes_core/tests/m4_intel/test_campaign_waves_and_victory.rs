//! TDD Integration Tests for Waves, Threat Director, Dungeon Heart, and Victory/Defeat.
//!
//! Validates `SPEC-REQ-WAVE-001` through `SPEC-REQ-WAVE-004` (`docs/specs/11_vagues_campagne_victoire.md`).

use tomb_of_heroes_core::campaign::{ThreatDirector, WavePhase};
use tomb_of_heroes_core::chrono::memory::{ChronoHero, HeroState};
use tomb_of_heroes_core::config::GameConfig;
use tomb_of_heroes_core::id::{LogicId, LogicIdGenerator};
use tomb_of_heroes_core::intel::veterancy::{TraumaTrait, VeteranProfile};
use tomb_of_heroes_core::math::BasisPoints;
use tomb_of_heroes_core::necro::terror::Bravery;
use tomb_of_heroes_core::necro::HeroClass;
use tomb_of_heroes_core::rng::DungeonMasterSeed;
use tomb_of_heroes_core::topology::coordinates::{FloorId, GridCoord, WorldCoord};
use tomb_of_heroes_core::topology::grid::FloorGrid;
use tomb_of_heroes_core::world::LogicWorld;

#[test]
fn test_threat_director_wave_compositions_and_veteran_reintegration() {
    let mut director = ThreatDirector::new();
    let mut id_gen = LogicIdGenerator::new();
    let spawn = WorldCoord::new(FloorId(0), GridCoord::new(1, 1));
    let target = WorldCoord::new(FloorId(2), GridCoord::new(5, 5));

    // Wave 1: 3 heroes (Warrior, Rogue, Cleric)
    assert_eq!(director.current_wave, 1);
    let wave1 = director.generate_wave_squad(&mut id_gen, spawn, target);
    assert_eq!(wave1.len(), 3);
    assert_eq!(wave1[0].hero_class, HeroClass::Warrior);
    assert_eq!(wave1[1].hero_class, HeroClass::Rogue);
    assert_eq!(wave1[2].hero_class, HeroClass::Cleric);
    assert_eq!(wave1[0].rank, 1);
    assert!(wave1[0].is_leader);

    // Wave 2: 4 heroes (Warrior, Rogue, Cleric, Mage)
    director.current_wave = 2;
    let wave2 = director.generate_wave_squad(&mut id_gen, spawn, target);
    assert_eq!(wave2.len(), 4);
    assert_eq!(wave2[3].hero_class, HeroClass::Mage);
    assert_eq!(wave2[0].rank, 2);

    // Simulate an escape from Wave 2 with trauma traits
    let veteran_id = LogicId(999);
    let veteran = VeteranProfile::new(
        veteran_id,
        999,
        HeroClass::Rogue,
        2,
        1,
        vec![],
        vec![TraumaTrait::TrapParanoia, TraumaTrait::Pyrophobia],
        None,
    );
    director.escaped_veterans.push(veteran);

    // Wave 3: Should reintegrate the escaped veteran in the primary slot!
    director.current_wave = 3;
    let wave3 = director.generate_wave_squad(&mut id_gen, spawn, target);
    assert_eq!(wave3.len(), 4);
    assert_eq!(wave3[0].hero_id, veteran_id);
    assert_eq!(wave3[0].hero_class, HeroClass::Rogue);
    assert_eq!(wave3[0].rank, 3);
    assert!(wave3[0].trauma_traits.contains(&TraumaTrait::TrapParanoia));
    assert!(wave3[0].trauma_traits.contains(&TraumaTrait::Pyrophobia));
    assert!(wave3[0].is_leader);

    // Wave 5: Grand Master Boss with absolute courage
    director.current_wave = 5;
    let wave5 = director.generate_wave_squad(&mut id_gen, spawn, target);
    assert_eq!(wave5.len(), 6);
    let boss = &wave5[0];
    assert!(boss.is_boss);
    assert!(boss.is_leader);
    assert_eq!(boss.max_hp, 250);
    assert_eq!(boss.armor_bps, BasisPoints(5000));
    assert_eq!(boss.bravery(), Bravery::ABSOLUTE);
}

#[test]
fn test_infamy_calculation_formula() {
    // Formula: (Killed * 50) + (Panic * 25) - (Escaped_Unhurt * 30)
    // Case A: 3 killed (1 panicked), 0 escaped
    // (3 * 50) + (1 * 25) - 0 = 175
    let delta_a = ThreatDirector::calculate_wave_infamy(3, 1, 0);
    assert_eq!(delta_a, 175);

    // Case B: 1 killed (0 panicked), 3 escaped unhurt
    // (1 * 50) + 0 - (3 * 30) = 50 - 90 = -40
    let delta_b = ThreatDirector::calculate_wave_infamy(1, 0, 3);
    assert_eq!(delta_b, -40);

    let mut director = ThreatDirector::new();
    director.infamy = 50;
    director.apply_wave_infamy(1, 0, 3);
    assert_eq!(director.infamy, 10);

    // Test underflow clamp to 0
    director.apply_wave_infamy(0, 0, 2);
    assert_eq!(director.infamy, 0);
}

#[test]
fn test_dungeon_heart_assault_and_immediate_defeat() {
    let mut world = LogicWorld::new(GameConfig::default(), DungeonMasterSeed(42));
    assert_eq!(world.heart().current_hp, 500);

    // Set heart at (0, 3, 3)
    let heart_coord = WorldCoord::new(FloorId(0), GridCoord::new(3, 3));
    world.heart_mut().position = heart_coord;

    // Transition to incursion
    world.campaign_mut().phase = WavePhase::Incursion;
    world.campaign_mut().active_incursion_heroes_count = 1;

    // Place hero right on the Heart tile
    let hero_id = LogicId(10);
    let mut hero = ChronoHero::new_ordinary(hero_id, HeroClass::Warrior, heart_coord);
    hero.state = HeroState::Infiltrating;
    world.register_hero(hero);

    // Step 20 ticks (1 second) -> at 10 dmg/sec, heart should lose exactly 10 HP
    for _ in 0..20 {
        world.step();
    }
    assert_eq!(world.heart().current_hp, 490);
    assert_eq!(world.campaign().phase, WavePhase::Incursion);

    // Direct destruction of heart triggers Defeat
    world.heart_mut().take_damage(490);
    assert_eq!(world.heart().current_hp, 0);
    world.step();
    assert_eq!(world.campaign().phase, WavePhase::Defeat);
}

#[test]
fn test_wave_cycle_preparation_incursion_debriefing_victory() {
    let mut world = LogicWorld::new(GameConfig::default(), DungeonMasterSeed(1234));
    assert_eq!(world.campaign().phase, WavePhase::Preparation);

    // Build floor 0 walkable grid
    let floor_grid = FloorGrid::new_with_bounds(FloorId(0), GridCoord::new(0, 0), 10, 10, true);
    world.dungeon_mut().add_floor(floor_grid);

    // Start Incursion manually (like clicking HUD button)
    world.start_incursion();
    assert_eq!(world.campaign().phase, WavePhase::Incursion);
    assert_eq!(world.campaign().active_incursion_heroes_count, 3);
    assert_eq!(world.heroes().len(), 3);

    // Simulate killing all 3 heroes
    let hero_ids: Vec<LogicId> = world.heroes().keys().copied().collect();
    for id in hero_ids {
        if let Some(h) = world.heroes_mut().get_mut(&id) {
            h.current_hp = 0;
            h.state = HeroState::Dead;
        }
    }

    // Step simulation to trigger incursion resolution
    world.step();

    // Incursion concluded -> Debriefing phase!
    assert_eq!(world.campaign().phase, WavePhase::Debriefing);
    assert!(world.campaign().infamy > 0);

    // Advance to wave 2
    world.campaign_mut().advance_to_next_wave();
    assert_eq!(world.campaign().phase, WavePhase::Preparation);
    assert_eq!(world.campaign().current_wave, 2);
    assert_eq!(world.campaign().stats.waves_cleared, 1);

    // Test Infamy >= 1000 victory
    world.campaign_mut().infamy = 1005;
    assert!(world.campaign().check_victory_condition());

    // Advance in debriefing with 1000+ infamy triggers Campaign Victory
    world.campaign_mut().phase = WavePhase::Debriefing;
    world.campaign_mut().advance_to_next_wave();
    assert_eq!(world.campaign().phase, WavePhase::Victory);
}
