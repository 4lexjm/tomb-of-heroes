use tomb_of_heroes_app::asset_gen::{
    generate_dungeon_sheet_image, generate_dungeon_sheet_png_bytes, SpriteIndex, SHEET_COLS,
    SHEET_HEIGHT, SHEET_ROWS, SHEET_WIDTH, TILE_SIZE,
};
use tomb_of_heroes_app::render::VisualEntityRef;
use tomb_of_heroes_app::simulation::WorldSimulation;
use tomb_of_heroes_core::chrono::memory::ChronoHero;
use tomb_of_heroes_core::necro::{Corpse, CorpseState, HeroClass};
use tomb_of_heroes_core::topology::{FloorId, GridCoord, WorldCoord};

#[test]
fn test_asset_generator_png_dimensions_and_validity() {
    assert_eq!(TILE_SIZE, 16);
    assert_eq!(SHEET_COLS, 8);
    assert_eq!(SHEET_ROWS, 4);
    assert_eq!(SHEET_WIDTH, 128);
    assert_eq!(SHEET_HEIGHT, 64);

    let img = generate_dungeon_sheet_image();
    assert_eq!(img.width(), 128);
    assert_eq!(img.height(), 64);

    let png_res = generate_dungeon_sheet_png_bytes();
    assert!(png_res.is_ok());
    let Ok(png_bytes) = png_res else {
        return;
    };
    assert!(png_bytes.len() > 8);
    assert_eq!(&png_bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]); // \x89PNG
}

#[test]
fn test_corpse_state_visual_transition_indices() {
    assert_eq!(SpriteIndex::CorpseIntact.index(), 24);
    assert_eq!(SpriteIndex::CorpseDamaged.index(), 25);
    assert_eq!(SpriteIndex::CorpseBones.index(), 26);

    let state_transitions = [
        (CorpseState::Intact, 24),
        (CorpseState::Damaged, 25),
        (CorpseState::Destroyed, 26),
    ];

    for (state, expected_index) in state_transitions {
        let index = match state {
            CorpseState::Intact => SpriteIndex::CorpseIntact.index(),
            CorpseState::Damaged => SpriteIndex::CorpseDamaged.index(),
            CorpseState::Destroyed => SpriteIndex::CorpseBones.index(),
        };
        assert_eq!(index, expected_index);
    }
}

#[test]
fn test_render_sync_visual_entity_ref_contract() {
    let mut sim = WorldSimulation::default();

    // Register 1 hero and 1 corpse
    let hero_id_res = sim.allocate_id();
    assert!(hero_id_res.is_ok());
    let Ok(hero_id) = hero_id_res else {
        return;
    };
    let hero = ChronoHero::new_ordinary(
        hero_id,
        HeroClass::Warrior,
        WorldCoord::new(FloorId(0), GridCoord::new(1, 1)),
    );
    sim.register_hero(hero);

    let corpse_id_res = sim.allocate_id();
    assert!(corpse_id_res.is_ok());
    let Ok(corpse_id) = corpse_id_res else {
        return;
    };

    let source_id_res = sim.allocate_id();
    assert!(source_id_res.is_ok());
    let Ok(source_id) = source_id_res else {
        return;
    };

    let corpse = Corpse::new(corpse_id, source_id, HeroClass::Mage, sim.config());
    let corpse_cfg = sim.config().corpse;
    let _ = sim
        .corpses_mut()
        .place_corpse(corpse, GridCoord::new(2, 2), &corpse_cfg);

    // Verify presence in core
    assert!(sim.heroes().contains_key(&hero_id));
    assert!(sim.corpses().get_corpse(corpse_id).is_some());

    // Verify VisualEntityRef bridges LogicId
    let hero_ref = VisualEntityRef(hero_id);
    let corpse_ref = VisualEntityRef(corpse_id);
    assert_eq!(hero_ref.0, hero_id);
    assert_eq!(corpse_ref.0, corpse_id);
    assert_ne!(hero_ref, corpse_ref);
}
