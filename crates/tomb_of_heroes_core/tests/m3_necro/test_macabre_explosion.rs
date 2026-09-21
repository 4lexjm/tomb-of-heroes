use tomb_of_heroes_core::{
    explode_corpse, sanctify_corpse, BasisPoints, Corpse, CorpseState, GameConfig, HeroClass,
    LogicId, NecroError,
};

#[test]
fn test_macabre_explosion_intact_corpse_success() {
    let config = GameConfig::default();
    let mut corpse = Corpse::new(LogicId(20), LogicId(200), HeroClass::Warrior, &config);

    assert_eq!(corpse.state, CorpseState::Intact);

    let res = explode_corpse(&mut corpse, &config.necro);
    assert!(res.is_ok());

    let Ok(explosion) = res else {
        return;
    };
    assert_eq!(explosion.damage, 80);
    assert_eq!(explosion.terror_bps, BasisPoints(2_000));
    assert_eq!(explosion.radius, 2);

    // Corpse is consumed and Destroyed
    assert_eq!(corpse.state, CorpseState::Destroyed);
}

#[test]
fn test_macabre_explosion_damaged_corpse_rejected() {
    let config = GameConfig::default();
    let mut corpse = Corpse::new(LogicId(21), LogicId(201), HeroClass::Rogue, &config);
    corpse.state = CorpseState::Damaged;

    let res = explode_corpse(&mut corpse, &config.necro);
    assert_eq!(res, Err(NecroError::InvalidCorpseState));
    // State remains Damaged
    assert_eq!(corpse.state, CorpseState::Damaged);
}

#[test]
fn test_macabre_explosion_sanctified_corpse_rejected() {
    let config = GameConfig::default();
    let mut corpse = Corpse::new(LogicId(22), LogicId(202), HeroClass::Cleric, &config);

    let sanctify_res = sanctify_corpse(&mut corpse);
    assert!(sanctify_res.is_ok());

    let res = explode_corpse(&mut corpse, &config.necro);
    assert_eq!(res, Err(NecroError::CorpseSanctified));
}
