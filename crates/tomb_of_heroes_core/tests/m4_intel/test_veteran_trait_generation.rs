use tomb_of_heroes_core::{
    generate_trauma_traits, BasisPoints, GameConfig, HeroClass, HeroRunStats, LogicId, TraumaTrait,
    VeteranProfile,
};

#[test]
fn test_veteran_trait_generation_pyrophobia_and_paranoia() {
    let config = GameConfig::default();

    // 1. Hero suffered fire damage and HP dropped below 1_000 BPS (10%)
    let fire_stats = HeroRunStats {
        min_hp_bps: BasisPoints(800), // 8.00% < 10.00%
        suffered_fire_damage: true,
        mechanical_traps_survived: 0,
        undead_killed: 0,
        consecutive_survivals: 1,
    };
    let traits = generate_trauma_traits(&fire_stats, &config.intel);
    assert!(traits.contains(&TraumaTrait::Pyrophobia));

    // 2. Hero did not suffer fire damage -> no Pyrophobia even if low HP
    let physical_stats = HeroRunStats {
        min_hp_bps: BasisPoints(500),
        suffered_fire_damage: false,
        mechanical_traps_survived: 0,
        undead_killed: 0,
        consecutive_survivals: 1,
    };
    let traits_phys = generate_trauma_traits(&physical_stats, &config.intel);
    assert!(!traits_phys.contains(&TraumaTrait::Pyrophobia));

    // 3. Hero survived 4 mechanical traps (> 3)
    let trap_stats = HeroRunStats {
        min_hp_bps: BasisPoints(5_000),
        suffered_fire_damage: false,
        mechanical_traps_survived: 4,
        undead_killed: 0,
        consecutive_survivals: 1,
    };
    let traits_trap = generate_trauma_traits(&trap_stats, &config.intel);
    assert!(traits_trap.contains(&TraumaTrait::TrapParanoia));

    // 4. Hero survived only 3 mechanical traps (not strictly > 3)
    let trap_stats_3 = HeroRunStats {
        min_hp_bps: BasisPoints(5_000),
        suffered_fire_damage: false,
        mechanical_traps_survived: 3,
        undead_killed: 0,
        consecutive_survivals: 1,
    };
    let traits_trap_3 = generate_trauma_traits(&trap_stats_3, &config.intel);
    assert!(!traits_trap_3.contains(&TraumaTrait::TrapParanoia));

    // 5. Undead slayer and vengeful tenacity
    let combo_stats = HeroRunStats {
        min_hp_bps: BasisPoints(900),
        suffered_fire_damage: true,
        mechanical_traps_survived: 5,
        undead_killed: 8,
        consecutive_survivals: 3,
    };
    let all_traits = generate_trauma_traits(&combo_stats, &config.intel);
    assert!(all_traits.contains(&TraumaTrait::Pyrophobia));
    assert!(all_traits.contains(&TraumaTrait::TrapParanoia));
    assert!(all_traits.contains(&TraumaTrait::UndeadSlayer));
    assert!(all_traits.contains(&TraumaTrait::VengefulTenacity));

    // 6. VeteranProfile instantiation with generated traits
    let veteran = VeteranProfile::new(
        LogicId(42),
        123456789,
        HeroClass::Warrior,
        1,
        1,
        vec![],
        all_traits.clone(),
        None,
    );
    assert_eq!(veteran.trauma_traits, all_traits);
}
