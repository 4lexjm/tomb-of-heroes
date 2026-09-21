use tomb_of_heroes_core::{
    pack_world, unpack_world, ChronoHero, Corpse, DungeonMasterSeed, FloorId, GameConfig,
    GridCoord, HeroClass, LogicId, LogicWorld, SaveEnvelope, WorldCoord,
};

#[test]
fn test_save_envelope_roundtrip_zstd() -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(1337);
    let mut world = LogicWorld::new(config.clone(), seed);

    for _ in 0..10 {
        world.step();
    }

    // Register heroes
    let hero1 = ChronoHero::new(
        LogicId(101),
        HeroClass::Warrior,
        true,
        WorldCoord::new(FloorId(0), GridCoord::new(5, 5)),
    );
    let hero2 = ChronoHero::new(
        LogicId(102),
        HeroClass::Mage,
        false,
        WorldCoord::new(FloorId(0), GridCoord::new(6, 6)),
    );
    world.register_hero(hero1);
    world.register_hero(hero2);

    // Register corpses
    let corpse1 = Corpse::new(LogicId(201), LogicId(101), HeroClass::Warrior, &config);
    let corpse2 = Corpse::new(LogicId(202), LogicId(102), HeroClass::Mage, &config);
    let _ = world
        .corpses_mut()
        .place_corpse(corpse1, GridCoord::new(5, 5), &config.corpse);
    let _ = world
        .corpses_mut()
        .place_corpse(corpse2, GridCoord::new(6, 6), &config.corpse);

    // Advance 5 more ticks
    for _ in 0..5 {
        world.step();
    }

    let campaign_id = [0x42u8; 16];
    let compression_level = 3;

    let envelope = pack_world(&world, campaign_id, compression_level)?;
    assert_eq!(envelope.header.campaign_id, campaign_id);
    assert_eq!(envelope.header.save_tick, world.current_tick().as_u64());
    assert_eq!(envelope.header.state_hash, world.state_hash().as_u64());
    assert!(envelope.header.uncompressed_size > 0);
    assert!(envelope.header.compressed_size > 0);
    assert_eq!(
        envelope.header.compressed_size as usize,
        envelope.payload.len()
    );

    // Binary serialization & deserialization roundtrip of SaveEnvelope
    let envelope_bytes = envelope.to_bytes();
    let decoded_envelope = SaveEnvelope::from_bytes(&envelope_bytes)?;
    assert_eq!(envelope, decoded_envelope);

    // Unpack world and verify bit-for-bit equivalence and StateHash match
    let restored_world = unpack_world(&decoded_envelope)?;
    assert_eq!(world, restored_world);
    assert_eq!(world.state_hash(), restored_world.state_hash());

    Ok(())
}
