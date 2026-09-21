use tomb_of_heroes_core::{unpack_world, SaveEnvelope, SaveError, SaveHeader, SAVE_MAGIC};

#[test]
fn test_empty_save_payload_from_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let header = SaveHeader {
        magic: SAVE_MAGIC,
        format_version: 1,
        flags: 0,
        campaign_id: [0u8; 16],
        save_tick: 0,
        uncompressed_size: 0,
        compressed_size: 0,
        payload_crc32: 0,
        state_hash: 0,
    };
    let header_bytes = header.to_bytes();

    // Decoding bytes containing only the header (0 payload bytes) must return EmptyPayload
    let res = SaveEnvelope::from_bytes(&header_bytes);
    assert_eq!(res, Err(SaveError::EmptyPayload));

    Ok(())
}

#[test]
fn test_empty_save_payload_unpack() -> Result<(), Box<dyn std::error::Error>> {
    let header = SaveHeader {
        magic: SAVE_MAGIC,
        format_version: 1,
        flags: 0,
        campaign_id: [0u8; 16],
        save_tick: 0,
        uncompressed_size: 0,
        compressed_size: 0,
        payload_crc32: 0,
        state_hash: 0,
    };
    let empty_envelope = SaveEnvelope {
        header,
        payload: Vec::new(),
    };

    // Unpacking an envelope with empty payload must return EmptyPayload
    let res = unpack_world(&empty_envelope);
    assert_eq!(res, Err(SaveError::EmptyPayload));

    Ok(())
}
