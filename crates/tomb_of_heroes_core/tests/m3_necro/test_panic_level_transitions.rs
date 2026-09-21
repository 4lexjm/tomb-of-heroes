use tomb_of_heroes_core::{GameConfig, PanicLevel, TerrorConfig, TerrorPoints};

#[test]
fn test_panic_level_transitions_at_exact_thresholds() {
    let config = TerrorConfig::default();

    // Serene: < 3_000
    assert_eq!(PanicLevel::from_points(0, &config), PanicLevel::Serene);
    assert_eq!(PanicLevel::from_points(2_999, &config), PanicLevel::Serene);

    // Shaken: 3_000..6_000
    assert_eq!(PanicLevel::from_points(3_000, &config), PanicLevel::Shaken);
    assert_eq!(PanicLevel::from_points(5_999, &config), PanicLevel::Shaken);

    // Disrupted: 6_000..8_500
    assert_eq!(
        PanicLevel::from_points(6_000, &config),
        PanicLevel::Disrupted
    );
    assert_eq!(
        PanicLevel::from_points(8_499, &config),
        PanicLevel::Disrupted
    );

    // BlindPanic: >= 8_500
    assert_eq!(
        PanicLevel::from_points(8_500, &config),
        PanicLevel::BlindPanic
    );
    assert_eq!(
        PanicLevel::from_points(10_000, &config),
        PanicLevel::BlindPanic
    );
}

#[test]
fn test_terror_points_panic_level_method() {
    let game_config = GameConfig::default();

    let tp_serene = TerrorPoints(1_500);
    assert_eq!(
        tp_serene.panic_level(&game_config.terror),
        PanicLevel::Serene
    );

    let tp_shaken = TerrorPoints(3_000);
    assert_eq!(
        tp_shaken.panic_level(&game_config.terror),
        PanicLevel::Shaken
    );

    let tp_disrupted = TerrorPoints(7_200);
    assert_eq!(
        tp_disrupted.panic_level(&game_config.terror),
        PanicLevel::Disrupted
    );

    let tp_panic = TerrorPoints(9_000);
    assert_eq!(
        tp_panic.panic_level(&game_config.terror),
        PanicLevel::BlindPanic
    );
}
