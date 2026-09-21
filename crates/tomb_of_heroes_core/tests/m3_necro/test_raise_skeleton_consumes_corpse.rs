use tomb_of_heroes_core::{
    raise_undead, Corpse, CorpseState, GameConfig, HeroClass, LogicId, NecroError, UndeadKind,
};

#[test]
fn test_raise_skeleton_success_deducts_mana_and_destroys_corpse() {
    let config = GameConfig::default();
    let mut corpse = Corpse::new(LogicId(1), LogicId(42), HeroClass::Warrior, &config);
    let mut mana: u32 = 100;

    let minion_res = raise_undead(&mut corpse, UndeadKind::SkeletonGuardian, &mut mana);
    assert!(minion_res.is_ok());

    let Ok(minion) = minion_res else {
        return;
    };
    assert_eq!(minion.kind, UndeadKind::SkeletonGuardian);
    assert_eq!(minion.source_hero_id, LogicId(42));
    assert_eq!(minion.source_class, HeroClass::Warrior);

    // 100 - 40 = 60 mana remaining
    assert_eq!(mana, 60);
    // Corpse consumed into Destroyed
    assert_eq!(corpse.state, CorpseState::Destroyed);

    // Attempting to raise on Destroyed corpse must fail
    let re_raise_res = raise_undead(&mut corpse, UndeadKind::SkeletonGuardian, &mut mana);
    assert_eq!(re_raise_res, Err(NecroError::InvalidCorpseState));
    // Mana must not be deducted on failure
    assert_eq!(mana, 60);
}

#[test]
fn test_raise_skeleton_on_damaged_corpse_succeeds() {
    let config = GameConfig::default();
    let mut corpse = Corpse::new(LogicId(2), LogicId(43), HeroClass::Rogue, &config);
    corpse.state = CorpseState::Damaged;

    let mut mana: u32 = 40;
    let minion_res = raise_undead(&mut corpse, UndeadKind::SkeletonGuardian, &mut mana);
    assert!(minion_res.is_ok());
    assert_eq!(mana, 0);
    assert_eq!(corpse.state, CorpseState::Destroyed);
}

#[test]
fn test_raise_skeleton_insufficient_mana_rejected() {
    let config = GameConfig::default();
    let mut corpse = Corpse::new(LogicId(3), LogicId(44), HeroClass::Warrior, &config);
    let mut mana: u32 = 39; // Requires 40

    let minion_res = raise_undead(&mut corpse, UndeadKind::SkeletonGuardian, &mut mana);
    assert_eq!(minion_res, Err(NecroError::InsufficientMana));
    // Mana and corpse state untouched
    assert_eq!(mana, 39);
    assert_eq!(corpse.state, CorpseState::Intact);
}

#[test]
fn test_raise_zombie_requires_intact_corpse_and_60_mana() {
    let config = GameConfig::default();
    let mut corpse_damaged = Corpse::new(LogicId(4), LogicId(45), HeroClass::Warrior, &config);
    corpse_damaged.state = CorpseState::Damaged;

    let mut mana: u32 = 100;
    // Zombie on damaged corpse must fail with InvalidCorpseState
    let res_damaged = raise_undead(&mut corpse_damaged, UndeadKind::FleshWallZombie, &mut mana);
    assert_eq!(res_damaged, Err(NecroError::InvalidCorpseState));
    assert_eq!(mana, 100);

    // Zombie on intact corpse succeeds with 60 mana
    let mut corpse_intact = Corpse::new(LogicId(5), LogicId(46), HeroClass::Warrior, &config);
    let res_intact = raise_undead(&mut corpse_intact, UndeadKind::FleshWallZombie, &mut mana);
    assert!(res_intact.is_ok());
    assert_eq!(mana, 40); // 100 - 60
    assert_eq!(corpse_intact.state, CorpseState::Destroyed);
}

#[test]
fn test_raise_spectre_requires_mage_class() {
    let config = GameConfig::default();
    let mut warrior_corpse = Corpse::new(LogicId(6), LogicId(47), HeroClass::Warrior, &config);
    let mut mana: u32 = 100;

    // Non-mage corpse rejected for Spectre
    let res_warrior = raise_undead(
        &mut warrior_corpse,
        UndeadKind::FallenSoulSpectre,
        &mut mana,
    );
    assert_eq!(res_warrior, Err(NecroError::HeroClassMismatch));
    assert_eq!(mana, 100);

    // Mage corpse succeeds
    let mut mage_corpse = Corpse::new(LogicId(7), LogicId(48), HeroClass::Mage, &config);
    let res_mage = raise_undead(&mut mage_corpse, UndeadKind::FallenSoulSpectre, &mut mana);
    assert!(res_mage.is_ok());
    assert_eq!(mage_corpse.state, CorpseState::Destroyed);
}
