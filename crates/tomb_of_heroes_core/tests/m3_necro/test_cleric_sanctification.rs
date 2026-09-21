use tomb_of_heroes_core::{
    raise_undead, sanctify_corpse, Corpse, CorpseState, GameConfig, HeroClass, LogicId, NecroError,
    UndeadKind,
};

#[test]
fn test_cleric_sanctification_destroys_corpse_and_prevents_reanimation() {
    let config = GameConfig::default();
    let mut corpse = Corpse::new(LogicId(10), LogicId(50), HeroClass::Paladin, &config);

    // Initial state: Intact, not sanctified
    assert_eq!(corpse.state, CorpseState::Intact);
    assert!(!corpse.is_sanctified);

    // Cleric performs sanctification
    let sanctify_res = sanctify_corpse(&mut corpse);
    assert!(sanctify_res.is_ok());

    // Postconditions: Corpse is Destroyed and marked is_sanctified = true
    assert_eq!(corpse.state, CorpseState::Destroyed);
    assert!(corpse.is_sanctified);

    // Any future necromantic reanimation attempt MUST return Err(NecroError::CorpseSanctified)
    let mut mana: u32 = 100;
    let raise_res = raise_undead(&mut corpse, UndeadKind::SkeletonGuardian, &mut mana);
    assert_eq!(raise_res, Err(NecroError::CorpseSanctified));

    // Mana was untouched
    assert_eq!(mana, 100);
}

#[test]
fn test_sanctify_already_sanctified_corpse_returns_error() {
    let config = GameConfig::default();
    let mut corpse = Corpse::new(LogicId(11), LogicId(51), HeroClass::Cleric, &config);

    let first_res = sanctify_corpse(&mut corpse);
    assert!(first_res.is_ok());

    let second_res = sanctify_corpse(&mut corpse);
    assert_eq!(second_res, Err(NecroError::CorpseSanctified));
}
