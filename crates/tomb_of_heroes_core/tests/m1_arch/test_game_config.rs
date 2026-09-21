use tomb_of_heroes_core::{BasisPoints, GameConfig};

#[test]
fn test_game_config_default_instantiation_and_constants() {
    let config = GameConfig::default();

    // 1. Fixed tick simulation constants (20 Hz, 50ms per tick, 5 catchup ticks)
    assert_eq!(config.tick.rate_hz, 20);
    assert_eq!(config.tick.duration_ms, 50);
    assert_eq!(config.tick.max_catchup_ticks_per_frame, 5);

    // 2. Standard basis points (10_000 = 100.00%)
    assert_eq!(config.standard_bps, BasisPoints(10_000));
    assert_eq!(BasisPoints::STANDARD, BasisPoints(10_000));
    assert_eq!(BasisPoints::FULL, BasisPoints(10_000));

    // 3. Corpse mechanics (3 corpses max per tile, 50 structural HP, damaged threshold at 25 HP)
    assert_eq!(config.corpse.max_per_tile, 3);
    assert_eq!(config.corpse.nominal_structural_hp, 50);
    assert_eq!(config.corpse.damaged_threshold_hp, 25);

    // 4. Terror thresholds (3_000 Shaken, 6_000 Disrupted, 8_500 Blind Panic, 10_000 Max)
    assert_eq!(config.terror.shaken_threshold, 3_000);
    assert_eq!(config.terror.disrupted_threshold, 6_000);
    assert_eq!(config.terror.blind_panic_threshold, 8_500);
    assert_eq!(config.terror.max_points, 10_000);

    // 5. Retreat thresholds (25% HP -> 2500 BPS, 50% losses -> 5000 BPS)
    assert_eq!(config.retreat.hp_threshold, BasisPoints(2_500));
    assert_eq!(config.retreat.casualty_threshold, BasisPoints(5_000));

    // 6. Chronomancy parameters (snapshot every 100 ticks, ring buffer 12, mana cost 20 + 500 BPS per 100 ticks)
    assert_eq!(config.chrono.snapshot_interval_ticks, 100);
    assert_eq!(config.chrono.ring_buffer_capacity, 12);
    assert_eq!(config.chrono.rewind_base_cost_mana, 20);
    assert_eq!(config.chrono.rewind_tick_cost_bps, BasisPoints(500));

    // 7. Safe Zone dimensions (320 x 240)
    assert_eq!(config.viewport.safe_zone_width, 320);
    assert_eq!(config.viewport.safe_zone_height, 240);
}

#[test]
fn test_game_config_serde_json_roundtrip() {
    let original = GameConfig::default();
    let serialized_res = serde_json::to_string_pretty(&original);
    assert!(serialized_res.is_ok());

    if let Ok(json_str) = serialized_res {
        let deserialized_res = serde_json::from_str::<GameConfig>(&json_str);
        assert!(deserialized_res.is_ok());
        if let Ok(cfg) = deserialized_res {
            assert_eq!(cfg, original);
        }
    }
}
