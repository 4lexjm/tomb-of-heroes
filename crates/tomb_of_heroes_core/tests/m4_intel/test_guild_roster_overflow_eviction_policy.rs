use tomb_of_heroes_core::{
    GameConfig, GuildRoster, HeroClass, LogicId, RosterOutcome, Tick, VeteranProfile,
};

#[test]
fn test_guild_roster_overflow_eviction_policy() {
    let config = GameConfig::default();
    let mut roster = GuildRoster::with_config(&config.intel);

    assert_eq!(roster.max_capacity, 32);

    // Insert 32 veterans
    // Veteran 1: 1 survived / 10 total = 10.00% survival ratio, last run at tick 1_000 (oldest, lowest ratio)
    let vet1 = VeteranProfile::new(
        LogicId(1),
        1001,
        HeroClass::Warrior,
        2,
        1,
        vec![],
        vec![],
        None,
    )
    .with_stats(10, Tick(1_000));
    assert_eq!(
        roster.register(vet1, &config.intel),
        RosterOutcome::Registered
    );

    // Fill remaining 31 slots with higher survival ratios
    for i in 2..=32 {
        let vet = VeteranProfile::new(
            LogicId(i),
            1000 + i,
            HeroClass::Rogue,
            2,
            5,
            vec![],
            vec![],
            None,
        )
        .with_stats(10, Tick(i * 1_000));
        assert_eq!(
            roster.register(vet, &config.intel),
            RosterOutcome::Registered
        );
    }

    assert_eq!(roster.len(), 32);

    // Insert 33rd veteran: 8 survived / 10 total = 80.00% survival ratio, tick 50_000
    let vet33 = VeteranProfile::new(
        LogicId(33),
        1033,
        HeroClass::Mage,
        3,
        8,
        vec![],
        vec![],
        None,
    )
    .with_stats(10, Tick(50_000));

    let outcome = roster.register(vet33, &config.intel);

    // Bounded strictly at 32
    assert_eq!(roster.len(), 32);

    // Veteran 1 must have been evicted (lowest ratio 10% vs 50%+, oldest run)
    assert_eq!(
        outcome,
        RosterOutcome::RegisteredWithEviction {
            evicted_id: LogicId(1)
        }
    );

    // Confirm LogicId(1) is no longer in roster
    let vet1_exists = roster.profiles.iter().any(|v| v.veteran_id == LogicId(1));
    assert!(!vet1_exists);

    // Confirm LogicId(33) is in roster
    let vet33_exists = roster.profiles.iter().any(|v| v.veteran_id == LogicId(33));
    assert!(vet33_exists);
}

#[test]
fn test_guild_roster_rank_5_honorable_retirement() {
    let config = GameConfig::default();
    let mut roster = GuildRoster::with_config(&config.intel);

    // Veteran reaches Rank 5 -> honorably retires to the royal court
    let vet_rank5 = VeteranProfile::new(
        LogicId(50),
        5000,
        HeroClass::Paladin,
        5, // Rank 5
        10,
        vec![],
        vec![],
        None,
    );

    let outcome = roster.register(vet_rank5, &config.intel);
    assert_eq!(outcome, RosterOutcome::HonorablyRetired);

    // Not added to active dungeon expedition roster
    assert_eq!(roster.len(), 0);
}
