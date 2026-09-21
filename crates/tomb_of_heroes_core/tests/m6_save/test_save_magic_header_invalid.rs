use tomb_of_heroes_core::{SaveEnvelope, SaveError, SaveHeader};

#[test]
fn test_save_magic_header_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let bad_bytes = b"BAD!some_corrupted_payload_bytes_that_should_not_pass";
    assert_eq!(
        SaveHeader::from_bytes(bad_bytes),
        Err(SaveError::InvalidMagic)
    );
    assert_eq!(
        SaveEnvelope::from_bytes(bad_bytes),
        Err(SaveError::InvalidMagic)
    );

    // Test too short bytes (less than 4)
    let tiny_bytes = b"TO";
    assert_eq!(
        SaveHeader::from_bytes(tiny_bytes),
        Err(SaveError::InvalidMagic)
    );
    assert_eq!(
        SaveEnvelope::from_bytes(tiny_bytes),
        Err(SaveError::InvalidMagic)
    );

    Ok(())
}
