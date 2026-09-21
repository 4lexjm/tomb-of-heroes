use tomb_of_heroes_core::{
    should_squad_retreat, squad_flee_speed_bonus, BasisPoints, GameConfig, LogicId, Squad,
    SquadMember, SquadState, TerrorPoints,
};

#[test]
fn test_retreat_trigger_on_casualties() {
    let config = GameConfig::default();

    // 1. Nominal squad of 4 healthy members
    let members = vec![
        SquadMember::new(LogicId(1), 100, 100, TerrorPoints::ZERO, true),
        SquadMember::new(LogicId(2), 100, 100, TerrorPoints::ZERO, false),
        SquadMember::new(LogicId(3), 100, 100, TerrorPoints::ZERO, false),
        SquadMember::new(LogicId(4), 100, 100, TerrorPoints::ZERO, false),
    ];
    let mut squad = Squad::new(LogicId(10), members);

    // Initial state: Incursion, should not retreat
    assert_eq!(squad.state, SquadState::Incursion);
    assert!(!should_squad_retreat(&squad, &config.retreat));
    assert_eq!(
        squad_flee_speed_bonus(squad.state, &config.retreat),
        BasisPoints::ZERO
    );

    // 2. Kill 2 members out of 4 (50% casualties, matches 5_000 BPS threshold)
    squad.members[2].current_hp = 0;
    squad.members[3].current_hp = 0;

    // Must trigger retreat
    assert!(should_squad_retreat(&squad, &config.retreat));
    squad.update_state(&config.retreat);
    assert_eq!(squad.state, SquadState::Retreat);

    // Flee speed bonus applied (+1_500 BPS)
    assert_eq!(
        squad_flee_speed_bonus(squad.state, &config.retreat),
        config.retreat.flee_speed_bonus_bps
    );
}

#[test]
fn test_retreat_trigger_on_leader_eliminated() {
    let config = GameConfig::default();

    // Squad of 4: Leader killed, but only 1 casualty out of 4 (25% < 50%)
    let members = vec![
        SquadMember::new(LogicId(1), 0, 100, TerrorPoints::ZERO, true), // Leader dead
        SquadMember::new(LogicId(2), 100, 100, TerrorPoints::ZERO, false),
        SquadMember::new(LogicId(3), 100, 100, TerrorPoints::ZERO, false),
        SquadMember::new(LogicId(4), 100, 100, TerrorPoints::ZERO, false),
    ];
    let mut squad = Squad::new(LogicId(11), members);

    assert!(should_squad_retreat(&squad, &config.retreat));
    squad.update_state(&config.retreat);
    assert_eq!(squad.state, SquadState::Retreat);
}

#[test]
fn test_retreat_trigger_on_critical_individual_hp() {
    let config = GameConfig::default();

    // Critical HP threshold is 2_500 BPS (25.00%).
    // 24 / 100 = 24.00% < 25.00%.
    let members = vec![
        SquadMember::new(LogicId(1), 100, 100, TerrorPoints::ZERO, true),
        SquadMember::new(LogicId(2), 24, 100, TerrorPoints::ZERO, false), // Critical HP
        SquadMember::new(LogicId(3), 100, 100, TerrorPoints::ZERO, false),
        SquadMember::new(LogicId(4), 100, 100, TerrorPoints::ZERO, false),
    ];
    let squad = Squad::new(LogicId(12), members);

    assert!(should_squad_retreat(&squad, &config.retreat));
}

#[test]
fn test_retreat_trigger_on_majority_blind_panic() {
    let config = GameConfig::default();

    // Blind panic threshold is 8_500 points.
    // 3 out of 4 living members in Blind Panic (> 50%).
    let members = vec![
        SquadMember::new(LogicId(1), 100, 100, TerrorPoints(9_000), true),
        SquadMember::new(LogicId(2), 100, 100, TerrorPoints(8_500), false),
        SquadMember::new(LogicId(3), 100, 100, TerrorPoints(8_600), false),
        SquadMember::new(LogicId(4), 100, 100, TerrorPoints(1_000), false),
    ];
    let squad = Squad::new(LogicId(13), members);

    assert!(should_squad_retreat(&squad, &config.retreat));
}
