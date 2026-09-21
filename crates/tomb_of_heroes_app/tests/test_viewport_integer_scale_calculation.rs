use tomb_of_heroes_app::viewport::{compute_pixel_perfect_scale, ViewportGeometry};

#[test]
fn test_viewport_integer_scale_calculation() {
    // 1920x1080 -> Scale 4 (for nominal target 426)
    // 240 * 4 = 960 <= 1080, 426 * 4 = 1704 <= 1920
    let scale_1080p = compute_pixel_perfect_scale(1920, 1080, 426);
    assert_eq!(scale_1080p, 4);

    let geom_1080p = ViewportGeometry::from_physical_dimensions(1920, 1080, 426);
    assert_eq!(geom_1080p.scale, 4);
    assert_eq!(geom_1080p.logical_width, 426);
    assert_eq!(geom_1080p.logical_height, 240);
    assert_eq!(geom_1080p.rendered_width, 1704);
    assert_eq!(geom_1080p.rendered_height, 960);
    assert_eq!(geom_1080p.letterbox_x, 108); // (1920 - 1704) / 2
    assert_eq!(geom_1080p.letterbox_y, 60); // (1080 - 960) / 2

    // 2560x1440 -> Scale 6 (for nominal target 426)
    // 240 * 6 = 1440 <= 1440, 426 * 6 = 2556 <= 2560
    let scale_1440p = compute_pixel_perfect_scale(2560, 1440, 426);
    assert_eq!(scale_1440p, 6);

    let geom_1440p = ViewportGeometry::from_physical_dimensions(2560, 1440, 426);
    assert_eq!(geom_1440p.scale, 6);
    assert_eq!(geom_1440p.rendered_width, 2556);
    assert_eq!(geom_1440p.rendered_height, 1440);
    assert_eq!(geom_1440p.letterbox_x, 2); // (2560 - 2556) / 2
    assert_eq!(geom_1440p.letterbox_y, 0); // (1440 - 1440) / 2

    // 3840x2160 (4K) -> Scale 9 (for nominal target 426)
    // 240 * 9 = 2160 <= 2160, 426 * 9 = 3834 <= 3840
    let scale_4k = compute_pixel_perfect_scale(3840, 2160, 426);
    assert_eq!(scale_4k, 9);

    let geom_4k = ViewportGeometry::from_physical_dimensions(3840, 2160, 426);
    assert_eq!(geom_4k.scale, 9);
    assert_eq!(geom_4k.rendered_width, 3834);
    assert_eq!(geom_4k.rendered_height, 2160);
    assert_eq!(geom_4k.letterbox_x, 3);
    assert_eq!(geom_4k.letterbox_y, 0);

    // 1280x720 -> Scale 3 (for nominal target 426)
    // 240 * 3 = 720 <= 720, 426 * 3 = 1278 <= 1280
    let scale_720p = compute_pixel_perfect_scale(1280, 720, 426);
    assert_eq!(scale_720p, 3);

    let geom_720p = ViewportGeometry::from_physical_dimensions(1280, 720, 426);
    assert_eq!(geom_720p.scale, 3);
    assert_eq!(geom_720p.rendered_width, 1278);
    assert_eq!(geom_720p.rendered_height, 720);
    assert_eq!(geom_720p.letterbox_x, 1);
    assert_eq!(geom_720p.letterbox_y, 0);

    // Edge cases: minimum scale is always at least 1, no division by zero panics
    let scale_tiny = compute_pixel_perfect_scale(100, 100, 426);
    assert_eq!(scale_tiny, 1);

    let scale_zero_w = compute_pixel_perfect_scale(0, 1080, 426);
    assert_eq!(scale_zero_w, 1);

    let scale_zero_target = compute_pixel_perfect_scale(1920, 1080, 0);
    assert_eq!(scale_zero_target, 1);
}
