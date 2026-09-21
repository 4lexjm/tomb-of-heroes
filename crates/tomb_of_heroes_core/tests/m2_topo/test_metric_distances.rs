use tomb_of_heroes_core::{chebyshev_distance, dist_sq, manhattan_distance, GridCoord};

#[test]
fn test_manhattan_and_chebyshev_distance() {
    let a = GridCoord::new(0, 0);
    let b = GridCoord::new(5, 3);

    assert_eq!(a.manhattan_distance(b), 8);
    assert_eq!(manhattan_distance(a, b), 8);

    assert_eq!(a.chebyshev_distance(b), 5);
    assert_eq!(chebyshev_distance(a, b), 5);
}

#[test]
fn test_negative_coordinates_distances() {
    let c = GridCoord::new(-4, 2);
    let d = GridCoord::new(1, -3);

    assert_eq!(c.manhattan_distance(d), 10);
    assert_eq!(manhattan_distance(c, d), 10);

    assert_eq!(c.chebyshev_distance(d), 5);
    assert_eq!(chebyshev_distance(c, d), 5);
}

#[test]
fn test_euclidean_distance_squared() {
    let e = GridCoord::new(0, 0);
    let f = GridCoord::new(3, 4);

    assert_eq!(e.dist_sq(f), 25);
    assert_eq!(dist_sq(e, f), 25);
}
