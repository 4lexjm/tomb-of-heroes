use tomb_of_heroes_core::{Corpse, CorpseConfig, CorpseState, GameConfig, HeroClass, LogicId};

#[test]
fn test_corpse_creation_intact_with_nominal_hp() {
    let config = GameConfig::default();
    let corpse_id = LogicId(101);
    let source_hero_id = LogicId(42);
    let hero_class = HeroClass::Warrior;

    let corpse = Corpse::new(corpse_id, source_hero_id, hero_class, &config);

    assert_eq!(corpse.corpse_id, corpse_id);
    assert_eq!(corpse.source_hero_id, source_hero_id);
    assert_eq!(corpse.hero_class, HeroClass::Warrior);
    assert_eq!(corpse.state, CorpseState::Intact);
    assert_eq!(corpse.structural_hp, 50);
    assert_eq!(corpse.structural_hp, config.corpse.nominal_structural_hp);
    assert!(corpse.soul_essence_value > 0);
    assert!(!corpse.is_sanctified);
}

#[test]
fn test_corpse_damage_transitions_to_damaged_and_destroyed() {
    let config = CorpseConfig::default();
    let mut corpse = Corpse::new(
        LogicId(1),
        LogicId(10),
        HeroClass::Paladin,
        &GameConfig::default(),
    );

    // Initial state: 50 HP -> Intact
    assert_eq!(corpse.state, CorpseState::Intact);
    assert_eq!(corpse.structural_hp, 50);

    // Inflict 20 damage -> 30 HP (> 25 HP) -> still Intact
    corpse.apply_damage(20, &config);
    assert_eq!(corpse.structural_hp, 30);
    assert_eq!(corpse.state, CorpseState::Intact);

    // Inflict 5 more damage -> 25 HP (<= 25 HP) -> Damaged
    corpse.apply_damage(5, &config);
    assert_eq!(corpse.structural_hp, 25);
    assert_eq!(corpse.state, CorpseState::Damaged);

    // Inflict 15 more damage -> 10 HP -> Damaged
    corpse.apply_damage(15, &config);
    assert_eq!(corpse.structural_hp, 10);
    assert_eq!(corpse.state, CorpseState::Damaged);

    // Inflict 10 more damage -> 0 HP -> Destroyed
    corpse.apply_damage(10, &config);
    assert_eq!(corpse.structural_hp, 0);
    assert_eq!(corpse.state, CorpseState::Destroyed);

    // Overkill damage should saturate at 0 and remain Destroyed
    corpse.apply_damage(50, &config);
    assert_eq!(corpse.structural_hp, 0);
    assert_eq!(corpse.state, CorpseState::Destroyed);
}
