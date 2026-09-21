//! TDD Integration Tests for Procedural Multi-Floor Dungeon Generation.
//!
//! Validates `SPEC-REQ-GEN-001` through `SPEC-REQ-GEN-004` (`docs/specs/10_generation_procedurale_donjon.md`).

use tomb_of_heroes_core::config::TopologyConfig;
use tomb_of_heroes_core::rng::DungeonMasterSeed;
use tomb_of_heroes_core::topology::coordinates::FloorId;
use tomb_of_heroes_core::topology::gen::{generate_procedural_dungeon, DungeonGeneratorConfig};
use tomb_of_heroes_core::topology::links::VerticalLinkKind;

#[test]
fn test_bsp_room_generation_bounds() {
    let config = DungeonGeneratorConfig::default();
    let topo_config = TopologyConfig::default();
    let seed = DungeonMasterSeed(42);

    let res = generate_procedural_dungeon(seed, &config, &topo_config);
    assert!(res.is_ok());
    let Ok(dungeon) = res else {
        return;
    };

    assert_eq!(dungeon.floor_rooms.len(), 3);

    for (floor_id, rooms) in &dungeon.floor_rooms {
        assert!(
            rooms.len() >= 4,
            "Floor {:?} should have at least 4 rooms",
            floor_id
        );
        for room in rooms {
            let width = room.max_x.saturating_sub(room.min_x);
            let height = room.max_y.saturating_sub(room.min_y);
            assert!(
                width >= config.min_room_dim && width <= config.max_room_dim,
                "Room width {} out of bounds",
                width
            );
            assert!(
                height >= config.min_room_dim && height <= config.max_room_dim,
                "Room height {} out of bounds",
                height
            );
            // Must be within floor boundaries
            assert!(room.min_x >= 1 && room.max_x < config.width - 1);
            assert!(room.min_y >= 1 && room.max_y < config.height - 1);
        }
    }
}

#[test]
fn test_vertical_stairs_and_pitfalls_presence() {
    let config = DungeonGeneratorConfig::default();
    let topo_config = TopologyConfig::default();
    let seed = DungeonMasterSeed(101);

    let res = generate_procedural_dungeon(seed, &config, &topo_config);
    assert!(res.is_ok());
    let Ok(dungeon) = res else {
        return;
    };

    let stairs_count = dungeon
        .vertical_links
        .iter()
        .filter(|l| l.kind == VerticalLinkKind::Stairs)
        .count();
    let pitfalls_count = dungeon
        .vertical_links
        .iter()
        .filter(|l| l.kind == VerticalLinkKind::Pitfall)
        .count();

    // 3 floors -> 2 stairs transitions and 2 pitfall transitions
    assert_eq!(stairs_count, 2);
    assert_eq!(pitfalls_count, 2);

    // Verify coordinates coherence
    for link in &dungeon.vertical_links {
        assert_eq!(link.source.coord, link.destination.coord);
        assert_eq!(link.destination.floor.0, link.source.floor.0 + 1);
    }
}

#[test]
fn test_continuous_astar_path_surface_to_heart() {
    let config = DungeonGeneratorConfig::default();
    let topo_config = TopologyConfig::default();

    // Test across several diverse seeds
    let test_seeds = [1, 42, 1337, 8888, 654321];

    for seed_val in test_seeds {
        let seed = DungeonMasterSeed(seed_val);
        let res = generate_procedural_dungeon(seed, &config, &topo_config);
        assert!(
            res.is_ok(),
            "Failed to generate dungeon for seed {}",
            seed_val
        );
        let Ok(dungeon) = res else {
            return;
        };

        assert_eq!(dungeon.spawn_point.floor, FloorId(0));
        assert_eq!(dungeon.dungeon_heart.floor, FloorId(2));

        let path_res =
            dungeon
                .grid
                .find_path(dungeon.spawn_point, dungeon.dungeon_heart, &topo_config);
        assert!(path_res.is_ok(), "Pathfinding failed for seed {}", seed_val);
        let Ok(path) = path_res else {
            return;
        };

        assert!(!path.is_empty(), "Path should not be empty");
        assert_eq!(path.first().copied(), Some(dungeon.spawn_point));
        assert_eq!(path.last().copied(), Some(dungeon.dungeon_heart));

        // Must traverse all floors 0 -> 1 -> 2
        let floors_traversed: Vec<u8> = path.iter().map(|p| p.floor.0).collect();
        assert!(floors_traversed.contains(&0));
        assert!(floors_traversed.contains(&1));
        assert!(floors_traversed.contains(&2));
    }
}

#[test]
fn test_procedural_generation_strict_determinism() {
    let config = DungeonGeneratorConfig::default();
    let topo_config = TopologyConfig::default();
    let seed = DungeonMasterSeed(999);

    let res1 = generate_procedural_dungeon(seed, &config, &topo_config);
    let res2 = generate_procedural_dungeon(seed, &config, &topo_config);

    assert!(res1.is_ok());
    assert!(res2.is_ok());

    let Ok(dungeon1) = res1 else {
        return;
    };
    let Ok(dungeon2) = res2 else {
        return;
    };

    assert_eq!(dungeon1.spawn_point, dungeon2.spawn_point);
    assert_eq!(dungeon1.dungeon_heart, dungeon2.dungeon_heart);
    assert_eq!(dungeon1.vertical_links, dungeon2.vertical_links);
    assert_eq!(dungeon1.floor_rooms, dungeon2.floor_rooms);
    assert_eq!(dungeon1.grid, dungeon2.grid);
}
