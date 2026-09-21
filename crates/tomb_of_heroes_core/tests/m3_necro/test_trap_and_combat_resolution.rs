use std::collections::BTreeMap;
use tomb_of_heroes_core::chrono::memory::ChronoHero;
use tomb_of_heroes_core::combat::{
    apply_critical_hit, calculate_effective_damage, detect_trap, disarm_trap,
    execute_death_sequence, MinionCombatProfile, TrapType,
};
use tomb_of_heroes_core::config::GameConfig;
use tomb_of_heroes_core::id::LogicId;
use tomb_of_heroes_core::math::BasisPoints;
use tomb_of_heroes_core::necro::corpse::{CorpseState, HeroClass};
use tomb_of_heroes_core::necro::necromancy::UndeadKind;
use tomb_of_heroes_core::necro::spatial::CorpseRegistry;
use tomb_of_heroes_core::necro::terror::PanicLevel;
use tomb_of_heroes_core::topology::coordinates::{FloorId, GridCoord, WorldCoord};

#[test]
fn test_trap_damage_calculation_and_fleeing_multiplier() {
    let spikes = TrapType::Spikes;
    assert_eq!(spikes.mana_cost(), 15);
    assert_eq!(spikes.base_damage(), 35);

    // Normal standing target with zero armor
    let res_normal = spikes.resolve_damage(false, BasisPoints::ZERO);
    assert_eq!(res_normal.damage, 35);

    // Fleeing target with zero armor takes doubled damage (70)
    let res_fleeing = spikes.resolve_damage(true, BasisPoints::ZERO);
    assert_eq!(res_fleeing.damage, 70);

    // Armor mitigation: 3000 BPS (30%) on 35 damage = (35 * 7000 + 5000) / 10000 = 25
    let res_armored = spikes.resolve_damage(false, BasisPoints(3000));
    assert_eq!(res_armored.damage, 25);

    // Acid pool: permanent armor shred of 2000 BPS
    let acid = TrapType::Acid;
    let res_acid = acid.resolve_damage(false, BasisPoints::ZERO);
    assert_eq!(res_acid.damage, 15);
    assert_eq!(res_acid.armor_shred_bps, BasisPoints(2000));
    assert_eq!(res_acid.poison_ticks, 3);
}

#[test]
fn test_trap_detection_and_disarm_rules() {
    // 1. Rogue base detection (8000 BPS)
    assert!(detect_trap(
        HeroClass::Rogue,
        false,
        PanicLevel::Serene,
        BasisPoints(7999)
    ));
    assert!(!detect_trap(
        HeroClass::Rogue,
        false,
        PanicLevel::Serene,
        BasisPoints(8000)
    ));

    // 2. Warrior base detection (2000 BPS)
    assert!(detect_trap(
        HeroClass::Warrior,
        false,
        PanicLevel::Serene,
        BasisPoints(1999)
    ));
    assert!(!detect_trap(
        HeroClass::Warrior,
        false,
        PanicLevel::Serene,
        BasisPoints(2000)
    ));

    // 3. Alerted status adds +2000 BPS
    assert!(detect_trap(
        HeroClass::Warrior,
        true,
        PanicLevel::Serene,
        BasisPoints(3999)
    ));
    assert!(!detect_trap(
        HeroClass::Warrior,
        true,
        PanicLevel::Serene,
        BasisPoints(4000)
    ));

    // 4. Blind panic disables all detection
    assert!(!detect_trap(
        HeroClass::Rogue,
        true,
        PanicLevel::BlindPanic,
        BasisPoints(0)
    ));
    assert!(!detect_trap(
        HeroClass::Warrior,
        true,
        PanicLevel::BlindPanic,
        BasisPoints(0)
    ));

    // 5. Disarming: only Rogue can disarm (7500 BPS)
    assert!(disarm_trap(HeroClass::Rogue, BasisPoints(7499)));
    assert!(!disarm_trap(HeroClass::Rogue, BasisPoints(7500)));
    assert!(!disarm_trap(HeroClass::Warrior, BasisPoints(0)));
    assert!(!disarm_trap(HeroClass::Mage, BasisPoints(0)));
}

#[test]
fn test_effective_damage_formula_and_critical_hits() {
    // Effective damage: clamp armor at 8000 BPS max
    let dmg = calculate_effective_damage(100, BasisPoints(10_000)); // Armor exceeds 8000 -> clamped to 8000
    assert_eq!(dmg, 20); // 100 * (10000 - 8000) / 10000 = 20

    // Minimum damage is always at least 1
    let min_dmg = calculate_effective_damage(0, BasisPoints::ZERO);
    assert_eq!(min_dmg, 1);

    // Critical strike: +50% (round half up)
    let normal = 20;
    let crit = apply_critical_hit(normal, true);
    assert_eq!(crit, 30); // 20 * 1.5 = 30

    let non_crit = apply_critical_hit(normal, false);
    assert_eq!(non_crit, 20);
}

#[test]
fn test_minion_combat_profiles() {
    let skel = UndeadKind::SkeletonGuardian;
    assert_eq!(skel.attack_damage(), 15);
    assert_eq!(skel.attack_cooldown_ticks(), 20);
    assert_eq!(skel.terror_on_hit_bps(), BasisPoints(500));
    assert!(!skel.ignores_target_armor());

    let zombie = UndeadKind::FleshWallZombie;
    assert_eq!(zombie.attack_damage(), 25);
    assert_eq!(zombie.attack_cooldown_ticks(), 40);
    assert_eq!(zombie.terror_on_hit_bps(), BasisPoints(1500));
    assert!(!zombie.ignores_target_armor());

    let spectre = UndeadKind::FallenSoulSpectre;
    assert_eq!(spectre.attack_damage(), 20);
    assert_eq!(spectre.attack_cooldown_ticks(), 20);
    assert_eq!(spectre.terror_on_hit_bps(), BasisPoints::ZERO);
    assert!(spectre.ignores_target_armor());
}

#[test]
fn test_atomic_death_sequence_execution() {
    let mut heroes = BTreeMap::new();
    let mut corpses = CorpseRegistry::new();
    let config = GameConfig::default();

    let leader_id = LogicId(1);
    let ally_id = LogicId(2);
    let corpse_id = LogicId(10);

    let mut leader = ChronoHero::new_ordinary(
        leader_id,
        HeroClass::Warrior,
        WorldCoord::new(FloorId(0), GridCoord::new(5, 5)),
    );
    leader.is_leader = true;

    let ally = ChronoHero::new_ordinary(
        ally_id,
        HeroClass::Cleric,
        WorldCoord::new(FloorId(0), GridCoord::new(5, 6)),
    );

    heroes.insert(leader_id, leader);
    heroes.insert(ally_id, ally);

    // Execute death of leader with ally in FOV
    let allies = [ally_id];
    let outcome_res = execute_death_sequence(
        leader_id,
        &mut heroes,
        &mut corpses,
        corpse_id,
        &config,
        &allies,
    );

    assert!(outcome_res.is_ok());
    let Ok(outcome) = outcome_res else {
        return;
    };

    // 1. Hero removed from living heroes
    assert!(!heroes.contains_key(&leader_id));
    assert!(heroes.contains_key(&ally_id));

    // 2. Corpse spawned at death location
    assert_eq!(outcome.settled_coord, GridCoord::new(5, 5));
    let corpse = corpses.get_corpse(corpse_id);
    assert!(corpse.is_some());
    let Some(c) = corpse else {
        return;
    };
    assert_eq!(c.state, CorpseState::Intact);
    assert_eq!(c.hero_class, HeroClass::Warrior);

    // 3. Death shock (+2500 BPS terror) applied to ally
    assert_eq!(outcome.death_shock_terror_bps, BasisPoints(2500));
    let living_ally = heroes.get(&ally_id);
    assert!(living_ally.is_some());
    let Some(a) = living_ally else {
        return;
    };
    assert_eq!(a.terror_bps, BasisPoints(2500));

    // 4. Ally switches to fleeing because leader died
    assert!(outcome.leader_died);
    assert!(a.is_fleeing);

    // 5. Soul essence harvest
    assert_eq!(outcome.mana_harvested, 20);
}
