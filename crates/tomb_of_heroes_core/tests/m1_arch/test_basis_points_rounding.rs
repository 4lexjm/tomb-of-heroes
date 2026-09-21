use tomb_of_heroes_core::{apply_bps, apply_bps_i32, div_bps, BasisPoints, MathError};

#[test]
fn test_basis_points_exact_values() {
    // Exact value: 1_000 * 10_000 BPS (100.00%) = 1_000
    let result = apply_bps(1_000, BasisPoints(10_000));
    assert_eq!(result, 1_000);
}

#[test]
fn test_basis_points_round_half_up() {
    // Round half up: 250 * 1_500 BPS (15.00%) = (375_000 + 5_000) / 10_000 = 38
    let result = apply_bps(250, BasisPoints(1_500));
    assert_eq!(result, 38);
}

#[test]
fn test_basis_points_rounding_thirds() {
    // Rounding on thirds: 333 * 3_333 BPS = (1_109_889 + 5_000) / 10_000 = 111
    let result = apply_bps(333, BasisPoints(3_333));
    assert_eq!(result, 111);
}

#[test]
fn test_basis_points_overflow_near_u32_max() {
    // Multiplication near u32::MAX must not overflow or panic due to u64 promotion
    let result = apply_bps(u32::MAX, BasisPoints(10_000));
    assert_eq!(result, u32::MAX);
}

#[test]
fn test_division_by_zero() {
    // div_bps(100, 0) must return Err(MathError::DivisionByZero) without panic
    let result = div_bps(100, 0);
    assert_eq!(result, Err(MathError::DivisionByZero));
}

#[test]
fn test_division_exact() {
    // div_bps(250, 1000) = 25.00% = 2500 BPS
    let result = div_bps(250, 1_000);
    assert_eq!(result, Ok(BasisPoints(2_500)));
}

#[test]
fn test_division_rounding_and_overflow() {
    // Thirds rounding: 1 / 3 = 33.33% = 3333 BPS
    assert_eq!(div_bps(1, 3), Ok(BasisPoints(3_333)));
    // Two thirds rounding: 2 / 3 = 66.67% = 6667 BPS
    assert_eq!(div_bps(2, 3), Ok(BasisPoints(6_667)));

    // Division resulting in overflow: u32::MAX / 1 * 10_000 > u32::MAX
    assert_eq!(div_bps(u32::MAX, 1), Err(MathError::Overflow));
}

#[test]
fn test_signed_basis_points() {
    // Positive values
    assert_eq!(apply_bps_i32(1_000, BasisPoints(10_000)), 1_000);
    assert_eq!(apply_bps_i32(250, BasisPoints(1_500)), 38);

    // Negative values (symmetric rounding away from zero)
    assert_eq!(apply_bps_i32(-1_000, BasisPoints(10_000)), -1_000);
    assert_eq!(apply_bps_i32(-250, BasisPoints(1_500)), -38);

    // Zero
    assert_eq!(apply_bps_i32(0, BasisPoints(10_000)), 0);

    // Limits
    assert_eq!(apply_bps_i32(i32::MAX, BasisPoints(10_000)), i32::MAX);
    assert_eq!(apply_bps_i32(i32::MIN, BasisPoints(10_000)), i32::MIN);
}

#[test]
fn test_basis_points_edge_cases() {
    // Zero multiplicand or zero factor
    assert_eq!(apply_bps(0, BasisPoints(10_000)), 0);
    assert_eq!(apply_bps(1_000, BasisPoints::ZERO), 0);

    // Round-half-up boundary exact checks: (val * bps + 5_000) / 10_000
    // 1 * 5_000 = 5_000 -> (5_000 + 5_000) / 10_000 = 1
    assert_eq!(apply_bps(1, BasisPoints(5_000)), 1);
    // 1 * 4_999 = 4_999 -> (4_999 + 5_000) / 10_000 = 0
    assert_eq!(apply_bps(1, BasisPoints(4_999)), 0);
    // 1 * 5_001 = 5_001 -> (5_001 + 5_000) / 10_000 = 1
    assert_eq!(apply_bps(1, BasisPoints(5_001)), 1);

    // Surcharge (> 100%)
    assert_eq!(apply_bps(100, BasisPoints(15_000)), 150);
}

#[test]
fn test_math_error_display() {
    let err_div = MathError::DivisionByZero;
    let err_ovf = MathError::Overflow;
    assert_eq!(
        format!("{err_div}"),
        "division by zero in deterministic math"
    );
    assert_eq!(
        format!("{err_ovf}"),
        "integer overflow in deterministic math"
    );
}

#[test]
fn test_basis_points_conversions_and_serde() {
    let bps = BasisPoints::new(2_500);
    assert_eq!(bps.raw(), 2_500);
    assert_eq!(u32::from(bps), 2_500);
    assert_eq!(BasisPoints::from(2_500), bps);

    let serialized_res = serde_json::to_string(&bps);
    assert!(serialized_res.is_ok());
    if let Ok(json) = serialized_res {
        assert_eq!(json, "2500");
        let deserialized_res = serde_json::from_str::<BasisPoints>(&json);
        assert!(deserialized_res.is_ok());
        if let Ok(b) = deserialized_res {
            assert_eq!(b, bps);
        }
    }
}
