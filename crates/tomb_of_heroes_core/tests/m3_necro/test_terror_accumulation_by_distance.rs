use tomb_of_heroes_core::{compute_terror_accumulation, BasisPoints, Bravery};

#[test]
fn test_novice_accumulates_more_terror_at_distance_1_than_at_distance_4() {
    // Novice bravery: 20.00% = 2_000 BPS
    let novice_bravery = Bravery(BasisPoints(2_000));
    // Base terror potency: 50.00% = 5_000 BPS
    let potency = BasisPoints(5_000);

    let terror_dist_1 = compute_terror_accumulation(1, potency, novice_bravery);
    let terror_dist_4 = compute_terror_accumulation(4, potency, novice_bravery);

    // Novice at 1 tile must accumulate strictly more terror than at 4 tiles
    assert!(
        terror_dist_1 > terror_dist_4,
        "Expected terror at d=1 ({terror_dist_1}) > terror at d=4 ({terror_dist_4})"
    );

    // Verify exact formula arithmetic:
    // d=1: (5_000 * (6 - 1)) / (5 * 20) = (25_000) / 100 = 250
    // apply_bps(250, BasisPoints(10_000 - 2_000)) = (250 * 8_000 + 5_000) / 10_000 = 200
    assert_eq!(terror_dist_1, 200);

    // d=4: (5_000 * (6 - 4)) / (5 * 20) = (10_000) / 100 = 100
    // apply_bps(100, BasisPoints(10_000 - 2_000)) = (100 * 8_000 + 5_000) / 10_000 = 80
    assert_eq!(terror_dist_4, 80);
}

#[test]
fn test_veteran_high_bravery_terror_resistance() {
    // Inquisitor Veteran: 95.00% = 9_500 BPS
    let veteran_bravery = Bravery(BasisPoints(9_500));
    let potency = BasisPoints(5_000);

    let terror_dist_1 = compute_terror_accumulation(1, potency, veteran_bravery);
    // d=1: base = 250.
    // apply_bps(250, BasisPoints(10_000 - 9_500)) = (250 * 500 + 5_000) / 10_000 = 130_000 / 10_000 = 13
    assert_eq!(terror_dist_1, 13);
}

#[test]
fn test_absolute_bravery_zero_terror() {
    // Absolute Bravery: 10_000 BPS (100.00%)
    let absolute_bravery = Bravery(BasisPoints(10_000));
    let potency = BasisPoints(5_000);

    let terror = compute_terror_accumulation(1, potency, absolute_bravery);
    assert_eq!(terror, 0);
}
