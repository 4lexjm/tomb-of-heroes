use tomb_of_heroes_core::{
    Corpse, CorpseConfig, CorpseRegistry, GameConfig, GridCoord, HeroClass, LogicId,
};

#[test]
fn test_corpse_stack_and_slide_at_capacity_3() {
    let game_config = GameConfig::default();
    let config: CorpseConfig = game_config.corpse;
    assert_eq!(config.max_per_tile, 3);

    let mut registry = CorpseRegistry::new();
    let origin = GridCoord::new(5, 5);

    let c1 = Corpse::new(LogicId(1), LogicId(101), HeroClass::Warrior, &game_config);
    let c2 = Corpse::new(LogicId(2), LogicId(102), HeroClass::Rogue, &game_config);
    let c3 = Corpse::new(LogicId(3), LogicId(103), HeroClass::Cleric, &game_config);
    let c4 = Corpse::new(LogicId(4), LogicId(104), HeroClass::Mage, &game_config);

    // Corpses 1, 2, 3 fit on the origin tile
    let pos1 = registry.place_corpse(c1, origin, &config);
    let pos2 = registry.place_corpse(c2, origin, &config);
    let pos3 = registry.place_corpse(c3, origin, &config);

    assert_eq!(pos1, Ok(origin));
    assert_eq!(pos2, Ok(origin));
    assert_eq!(pos3, Ok(origin));
    assert_eq!(registry.corpse_count_at(origin), 3);

    // 4th corpse exceeds tile capacity (3), must slide to adjacent neighbor
    let pos4_res = registry.place_corpse(c4, origin, &config);
    assert!(pos4_res.is_ok());

    let Ok(pos4) = pos4_res else {
        return;
    };
    // Origin tile remains at capacity 3
    assert_eq!(registry.corpse_count_at(origin), 3);
    // 4th corpse is at a different tile
    assert_ne!(pos4, origin);
    // Distance to origin must be 1 (adjacent neighbor)
    assert_eq!(origin.chebyshev_distance(pos4), 1);
    // The neighbor tile now has 1 corpse
    assert_eq!(registry.corpse_count_at(pos4), 1);
}
