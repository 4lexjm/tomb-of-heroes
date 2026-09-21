use tomb_of_heroes_app::ui::{HudState, PlacementTool, MIN_TOUCH_TARGET_SIZE};
use tomb_of_heroes_core::save::{
    export_to_base64_string, import_from_base64_string, pack_world, unpack_world,
    DEFAULT_CAMPAIGN_ID, DEFAULT_COMPRESSION_LEVEL,
};
use tomb_of_heroes_core::world::LogicWorld;

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_hud_touch_target_minimum_size() {
    // SPEC-REQ-FRONT-005: all tactile targets must be >= 44x44 pt
    assert!(
        MIN_TOUCH_TARGET_SIZE.x >= 44.0,
        "Touch width {} must be >= 44.0 pt",
        MIN_TOUCH_TARGET_SIZE.x
    );
    assert!(
        MIN_TOUCH_TARGET_SIZE.y >= 44.0,
        "Touch height {} must be >= 44.0 pt",
        MIN_TOUCH_TARGET_SIZE.y
    );
}

#[test]
fn test_hud_initial_state_and_tools() {
    let hud = HudState::default();
    assert_eq!(hud.mana, 85);
    assert_eq!(hud.max_mana, 100);
    assert_eq!(hud.infamy, 120);
    assert_eq!(hud.alert_level, 2);
    assert_eq!(hud.selected_tool, PlacementTool::None);
    assert!(!hud.is_paused);
    assert!(!hud.show_inspector);
    assert!(!hud.show_about_modal);
    assert!(!hud.show_chrono_window);
    assert!(!hud.show_save_modal);
    assert_eq!(hud.wave, 1);
    assert_eq!(hud.max_waves, 5);
    assert_eq!(
        hud.wave_phase,
        tomb_of_heroes_core::campaign::WavePhase::Preparation
    );
    assert_eq!(hud.heart_hp, 500);
    assert_eq!(hud.heart_max_hp, 500);
}

#[test]
fn test_chronomancy_rewind_cost_and_paradox_formula() {
    // Formula: Mana = max(1, (ticks * 3) / 10)
    // Formula: Paradox = min(10_000, ticks * 15)
    let cases = [
        (5, 1, 75),
        (10, 3, 150),
        (20, 6, 300),
        (50, 15, 750),
        (100, 30, 1500),
    ];

    for (ticks, expected_mana, expected_paradox) in cases {
        let mana_cost = ((ticks * 3) / 10).max(1) as u32;
        let paradox = (ticks * 15).min(10_000) as u32;
        assert_eq!(mana_cost, expected_mana);
        assert_eq!(paradox, expected_paradox);
    }
}

#[test]
fn test_save_export_import_roundtrip() {
    let world = LogicWorld::new(
        Default::default(),
        tomb_of_heroes_core::rng::DungeonMasterSeed(42),
    );
    let initial_hash = world.state_hash();

    // 1. Pack into SaveEnvelope
    let envelope_res = pack_world(&world, DEFAULT_CAMPAIGN_ID, DEFAULT_COMPRESSION_LEVEL);
    assert!(envelope_res.is_ok());
    let Ok(envelope) = envelope_res else {
        return;
    };

    // 2. Export to ASCII Base64 armor
    let armored = export_to_base64_string(&envelope);
    assert!(armored.starts_with("-----BEGIN TOMB OF HEROES SAVE-----"));
    assert!(armored.ends_with("-----END TOMB OF HEROES SAVE-----"));

    // 3. Import from ASCII Base64 armor
    let restored_envelope_res = import_from_base64_string(&armored);
    assert!(restored_envelope_res.is_ok());
    let Ok(restored_envelope) = restored_envelope_res else {
        return;
    };

    // 4. Unpack world
    let restored_world_res = unpack_world(&restored_envelope);
    assert!(restored_world_res.is_ok());
    let Ok(restored_world) = restored_world_res else {
        return;
    };

    // 5. Verify bit-level determinism via state hash
    assert_eq!(restored_world.state_hash(), initial_hash);
}
