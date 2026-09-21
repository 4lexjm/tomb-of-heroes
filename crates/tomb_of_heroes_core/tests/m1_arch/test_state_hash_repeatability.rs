use rand_xoshiro::rand_core::RngCore;
use tomb_of_heroes_core::{DungeonMasterSeed, GameConfig, LogicWorld, RngStreamKind};

#[test]
fn test_state_hash_repeatability_across_identical_runs() {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(777_888);

    let mut world1 = LogicWorld::new(config.clone(), seed);
    let mut world2 = LogicWorld::new(config, seed);

    // Initial state hash equality
    assert_eq!(
        world1.state_hash(),
        world2.state_hash(),
        "Initial StateHash must be bit-for-bit identical on identical setup"
    );

    // Perform the same identical sequence of operations on both instances
    for _ in 0..25 {
        world1.step();
        world2.step();

        let _ = world1.allocate_id();
        let _ = world2.allocate_id();

        let _ = world1
            .rng_bank_mut()
            .stream_mut(RngStreamKind::Combat)
            .next_u64();
        let _ = world2
            .rng_bank_mut()
            .stream_mut(RngStreamKind::Combat)
            .next_u64();

        assert_eq!(
            world1.state_hash(),
            world2.state_hash(),
            "StateHash diverged during deterministic step progression"
        );
    }
}

#[test]
fn test_state_hash_changes_on_progression() {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(123);

    let mut world = LogicWorld::new(config, seed);
    let initial_hash = world.state_hash();

    // Advancing one step must mutate the state hash
    world.step();
    let stepped_hash = world.state_hash();
    assert_ne!(
        initial_hash, stepped_hash,
        "Step progression must change the StateHash"
    );

    // Allocating an ID must mutate the state hash
    let _ = world.allocate_id();
    let allocated_hash = world.state_hash();
    assert_ne!(
        stepped_hash, allocated_hash,
        "LogicId allocation must change the StateHash"
    );

    // Drawing PRNG values must mutate the state hash
    let _ = world
        .rng_bank_mut()
        .stream_mut(RngStreamKind::Combat)
        .next_u64();
    let rng_hash = world.state_hash();
    assert_ne!(
        allocated_hash, rng_hash,
        "PRNG stream draw must change the StateHash"
    );
}

#[test]
fn test_state_hash_divergence_on_different_seeds() {
    let config = GameConfig::default();
    let world1 = LogicWorld::new(config.clone(), DungeonMasterSeed(1));
    let world2 = LogicWorld::new(config, DungeonMasterSeed(2));

    assert_ne!(
        world1.state_hash(),
        world2.state_hash(),
        "Different seeds must yield distinct StateHash"
    );
}
