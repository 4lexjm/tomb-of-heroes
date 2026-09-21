use tomb_of_heroes_app::viewport::{SafeZone, ViewportGeometry};

#[test]
fn test_safe_zone_clamping() {
    // In a 426x240 viewport:
    // W_logical = 426, W_safe = 320 -> (426 - 320) / 2 = 53
    // Safe Zone x in [53, 373]
    let safe_zone = SafeZone::from_logical_width(426);
    assert_eq!(safe_zone.x_min, 53);
    assert_eq!(safe_zone.x_max, 373);
    assert_eq!(safe_zone.y_min, 0);
    assert_eq!(safe_zone.y_max, 240);

    // Also via ViewportGeometry::safe_zone()
    let geom = ViewportGeometry::from_physical_dimensions(1920, 1080, 426);
    let geom_safe_zone = geom.safe_zone();
    assert_eq!(geom_safe_zone.x_min, 53);
    assert_eq!(geom_safe_zone.x_max, 373);
    assert_eq!(geom_safe_zone.y_min, 0);
    assert_eq!(geom_safe_zone.y_max, 240);

    // Points inside are preserved
    assert!(safe_zone.is_inside(53, 0));
    assert!(safe_zone.is_inside(373, 240));
    assert!(safe_zone.is_inside(200, 120));
    assert_eq!(safe_zone.clamp(200, 120), (200, 120));
    assert_eq!(safe_zone.clamp(53, 0), (53, 0));
    assert_eq!(safe_zone.clamp(373, 240), (373, 240));

    // Points outside are strictly clamped
    assert!(!safe_zone.is_inside(52, 120));
    assert!(!safe_zone.is_inside(374, 120));
    assert!(!safe_zone.is_inside(200, -1));
    assert!(!safe_zone.is_inside(200, 241));

    assert_eq!(safe_zone.clamp(0, 0), (53, 0));
    assert_eq!(safe_zone.clamp(500, 300), (373, 240));
    assert_eq!(safe_zone.clamp(-10, -50), (53, 0));
    assert_eq!(safe_zone.clamp(1000, 100), (373, 100));
}
