use tomb_of_heroes_core::{
    FloorId, GridCoord, NavigationError, VerticalLink, VerticalLinkKind, WorldCoord,
};

#[test]
fn test_pitfall_downward_and_upward_traversal() {
    let source = WorldCoord::new(FloorId(0), GridCoord::new(5, 5));
    let destination = WorldCoord::new(FloorId(1), GridCoord::new(5, 5));
    let pitfall = VerticalLink::new(VerticalLinkKind::Pitfall, source, destination);

    // Traversée descendante autorisée : de (FloorId(0), (5, 5)) vers (FloorId(1), (5, 5))
    let descent_result = pitfall.traverse(source);
    assert_eq!(descent_result, Ok(destination));

    // Tentative ascendante : rejet avec Err(NavigationError::PassageUnidirectional)
    let ascent_result = pitfall.traverse(destination);
    assert_eq!(ascent_result, Err(NavigationError::PassageUnidirectional));
}

#[test]
fn test_oneway_portal_upward_rejection() {
    let source = WorldCoord::new(FloorId(0), GridCoord::new(2, 2));
    let destination = WorldCoord::new(FloorId(2), GridCoord::new(2, 2));
    let portal = VerticalLink::new(VerticalLinkKind::OneWayPortal, source, destination);

    assert_eq!(portal.traverse(source), Ok(destination));
    assert_eq!(
        portal.traverse(destination),
        Err(NavigationError::PassageUnidirectional)
    );
}

#[test]
fn test_stairs_and_ladder_bidirectional_traversal() {
    let source = WorldCoord::new(FloorId(0), GridCoord::new(3, 3));
    let destination = WorldCoord::new(FloorId(1), GridCoord::new(3, 3));

    let stairs = VerticalLink::new(VerticalLinkKind::Stairs, source, destination);
    assert_eq!(stairs.traverse(source), Ok(destination));
    assert_eq!(stairs.traverse(destination), Ok(source));

    let ladder = VerticalLink::new(VerticalLinkKind::Ladder, source, destination);
    assert_eq!(ladder.traverse(source), Ok(destination));
    assert_eq!(ladder.traverse(destination), Ok(source));
}
