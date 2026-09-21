use tomb_of_heroes_core::{
    pack_world, unpack_world, DungeonMasterSeed, GameConfig, LogicWorld, SaveError,
};

#[test]
fn test_crc32_header_mismatch_detected() -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(42);
    let mut world = LogicWorld::new(config, seed);
    for _ in 0..5 {
        world.step();
    }

    let campaign_id = [0xAA; 16];
    let mut envelope = pack_world(&world, campaign_id, 3)?;

    // Tamper with the CRC in the header to ensure decompression succeeds but CRC check fails
    envelope.header.payload_crc32 ^= 0xFFFFFFFF;

    let result = unpack_world(&envelope);
    assert_eq!(result, Err(SaveError::CrcMismatch));

    Ok(())
}

#[test]
fn test_crc32_payload_corruption_detected() -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(42);
    let mut world = LogicWorld::new(config, seed);
    for _ in 0..5 {
        world.step();
    }

    let campaign_id = [0xBB; 16];
    let mut envelope = pack_world(&world, campaign_id, 3)?;

    // Alter byte in payload: zstd decompression fails or CRC mismatch occurs
    if !envelope.payload.is_empty() {
        let last_idx = envelope.payload.len() - 1;
        envelope.payload[last_idx] ^= 0x55;
    }

    let result = unpack_world(&envelope);
    assert!(
        matches!(
            result,
            Err(SaveError::CrcMismatch) | Err(SaveError::DecompressionFailed(_))
        ),
        "Expected CrcMismatch or DecompressionFailed, got: {:?}",
        result
    );

    Ok(())
}
