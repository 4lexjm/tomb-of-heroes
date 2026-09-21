use bevy::math::{Vec2, Vec3};
use tomb_of_heroes_app::camera::CameraSettings;
use tomb_of_heroes_app::render::{grid_to_world, world_to_grid, Z_CREATURES, Z_FLOOR, Z_WALLS};
use tomb_of_heroes_app::viewport::compute_pixel_perfect_scale;
use tomb_of_heroes_core::topology::GridCoord;

#[test]
fn test_integer_scale_formula_various_resolutions() {
    // 320x240 => 1
    assert_eq!(compute_pixel_perfect_scale(320, 240, 320), 1);
    // 640x480 => 2
    assert_eq!(compute_pixel_perfect_scale(640, 480, 320), 2);
    // 1280x720 => min(1280/320, 720/240) = min(4, 3) = 3
    assert_eq!(compute_pixel_perfect_scale(1280, 720, 320), 3);
    // 1920x1080 => min(1920/320, 1080/240) = min(6, 4) = 4
    assert_eq!(compute_pixel_perfect_scale(1920, 1080, 320), 4);
    // 2560x1440 => min(2560/320, 1440/240) = min(8, 6) = 6
    assert_eq!(compute_pixel_perfect_scale(2560, 1440, 320), 6);
}

#[test]
fn test_zero_or_negative_window_size_scaling() {
    assert_eq!(compute_pixel_perfect_scale(0, 0, 320), 1);
    assert_eq!(compute_pixel_perfect_scale(100, 50, 320), 1);
}

#[test]
fn test_grid_to_world_coordinate_conversion() {
    // Origin
    let w0 = grid_to_world(GridCoord::new(0, 0), Z_FLOOR);
    assert_eq!(w0, Vec3::new(0.0, 0.0, Z_FLOOR));

    // Positive coordinates (Y inverted for Bevy upward Y)
    let w1 = grid_to_world(GridCoord::new(5, 3), Z_CREATURES);
    assert_eq!(w1, Vec3::new(80.0, -48.0, Z_CREATURES));

    // Negative coordinates
    let w2 = grid_to_world(GridCoord::new(-2, 4), Z_WALLS);
    assert_eq!(w2, Vec3::new(-32.0, -64.0, Z_WALLS));
}

#[test]
fn test_world_to_grid_roundtrip() {
    let coords = [
        GridCoord::new(0, 0),
        GridCoord::new(3, 7),
        GridCoord::new(-4, -5),
        GridCoord::new(10, -8),
    ];

    for coord in coords {
        let world_pos = grid_to_world(coord, 0.0);
        let recovered_coord = world_to_grid(Vec2::new(world_pos.x, world_pos.y));
        assert_eq!(
            recovered_coord, coord,
            "Coordinate roundtrip failed for {coord:?}"
        );
    }
}

#[test]
fn test_camera_zoom_clamping() {
    let mut settings = CameraSettings::default();
    assert_eq!(settings.zoom, 1);

    // Zoom in
    settings.zoom = (settings.zoom + 1).clamp(1, 4);
    assert_eq!(settings.zoom, 2);

    // Upper clamp
    settings.zoom = 10;
    settings.zoom = settings.zoom.clamp(1, 4);
    assert_eq!(settings.zoom, 4);

    // Lower clamp
    settings.zoom = 0;
    settings.zoom = settings.zoom.clamp(1, 4);
    assert_eq!(settings.zoom, 1);
}
