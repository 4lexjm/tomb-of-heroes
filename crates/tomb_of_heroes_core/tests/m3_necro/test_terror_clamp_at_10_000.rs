use tomb_of_heroes_core::{TerrorConfig, TerrorPoints};

#[test]
fn test_terror_strictly_clamped_at_10_000() {
    let config = TerrorConfig::default();
    assert_eq!(config.max_points, 10_000);

    let mut tp = TerrorPoints(9_900);
    tp.accumulate(200, &config);
    assert_eq!(tp.0, 10_000);

    // Massive terror burst must not exceed max_points (10_000)
    tp.accumulate(50_000, &config);
    assert_eq!(tp.0, 10_000);

    // Starting from 0 and over-accumulating
    let mut tp_zero = TerrorPoints(0);
    tp_zero.accumulate(15_000, &config);
    assert_eq!(tp_zero.0, 10_000);
}
