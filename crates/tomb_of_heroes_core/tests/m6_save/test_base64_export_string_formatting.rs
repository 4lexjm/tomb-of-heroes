use tomb_of_heroes_core::{
    export_to_base64_armor, import_from_base64_armor, pack_world, DungeonMasterSeed, GameConfig,
    LogicWorld, SaveError,
};

#[test]
fn test_base64_export_armor_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let seed = DungeonMasterSeed(777);
    let mut world = LogicWorld::new(config, seed);
    world.step();

    let campaign_id = [7u8; 16];
    let envelope = pack_world(&world, campaign_id, 3)?;

    let armored = export_to_base64_armor(&envelope);
    assert!(armored.starts_with("-----BEGIN TOMB OF HEROES SAVE-----\n"));
    assert!(armored.ends_with("\n-----END TOMB OF HEROES SAVE-----"));

    let restored_envelope = import_from_base64_armor(&armored)?;
    assert_eq!(envelope, restored_envelope);

    // Verify whitespace and carriage return resilience
    let modified_armored = armored.replace('\n', "\r\n  \t  \n");
    let restored_from_dirty = import_from_base64_armor(&modified_armored)?;
    assert_eq!(envelope, restored_from_dirty);

    // Corrupted base64 content returns Base64Error
    let bad_armored =
        "-----BEGIN TOMB OF HEROES SAVE-----\nINVALID!!!BASE64???\n-----END TOMB OF HEROES SAVE-----";
    let bad_res = import_from_base64_armor(bad_armored);
    assert!(matches!(bad_res, Err(SaveError::Base64Error(_))));

    // Missing headers returns Base64Error
    let missing_headers = "VE9IUwEAAAB...";
    let missing_res = import_from_base64_armor(missing_headers);
    assert!(matches!(missing_res, Err(SaveError::Base64Error(_))));

    Ok(())
}
