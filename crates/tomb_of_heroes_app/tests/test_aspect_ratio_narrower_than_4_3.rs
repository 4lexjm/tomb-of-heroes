use tomb_of_heroes_app::viewport::{
    compute_pixel_perfect_scale, ViewportGeometry, LOGICAL_HEIGHT, LOGICAL_WIDTH_MIN,
};

#[test]
fn test_aspect_ratio_narrower_than_4_3() {
    // On square screen 800x800, scale computed from W_min (320)
    // S = min(800 / 320 = 2, 800 / 240 = 3) = 2
    let scale_square = compute_pixel_perfect_scale(800, 800, LOGICAL_WIDTH_MIN);
    assert_eq!(scale_square, 2);

    let geom_square = ViewportGeometry::from_physical_dimensions(800, 800, LOGICAL_WIDTH_MIN);
    assert_eq!(geom_square.scale, 2);
    assert_eq!(geom_square.logical_width, 320);
    assert_eq!(geom_square.logical_height, 240);
    assert_eq!(geom_square.rendered_width, 640);
    assert_eq!(geom_square.rendered_height, 480);
    // Symmetric pillarboxing and letterboxing
    assert_eq!(geom_square.letterbox_x, 80); // (800 - 640) / 2
    assert_eq!(geom_square.letterbox_y, 160); // (800 - 480) / 2

    // Safe zone on 320x240 is full logical width
    let safe_zone = geom_square.safe_zone();
    assert_eq!(safe_zone.x_min, 0);
    assert_eq!(safe_zone.x_max, 320);
    assert_eq!(safe_zone.y_min, 0);
    assert_eq!(safe_zone.y_max, 240);

    // Highly vertical aspect ratio (e.g. smartphone portrait 600x1200)
    // S = min(600 / 320 = 1, 1200 / 240 = 5) = 1
    let scale_tall = compute_pixel_perfect_scale(600, 1200, LOGICAL_WIDTH_MIN);
    assert_eq!(scale_tall, 1);

    let geom_tall = ViewportGeometry::from_physical_dimensions(600, 1200, LOGICAL_WIDTH_MIN);
    assert_eq!(geom_tall.scale, 1);
    assert_eq!(geom_tall.rendered_width, 320);
    assert_eq!(geom_tall.rendered_height, 240);
    assert_eq!(geom_tall.letterbox_x, 140); // (600 - 320) / 2
    assert_eq!(geom_tall.letterbox_y, 480); // (1200 - 240) / 2

    // Check constants integrity
    assert_eq!(LOGICAL_HEIGHT, 240);
    assert_eq!(LOGICAL_WIDTH_MIN, 320);
}
