//! Dungeon Tilemap, Entity Sprites, and Fog of War Rendering Module.
//!
//! Complies with `SPEC-REQ-FRONT-002`, `SPEC-REQ-FRONT-003`, and `SPEC-REQ-FRONT-004`.

use std::path::Path;

use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{Assets, Handle};
use bevy::ecs::change_detection::DetectChanges;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::EventReader;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query, Res, ResMut, Resource};
use bevy::image::Image;
use bevy::input::mouse::MouseButton;
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::input::ButtonInput;
use bevy::math::{UVec2, Vec2, Vec3};
use bevy::render::camera::Camera;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::view::Visibility;
use bevy::sprite::{Sprite, TextureAtlas, TextureAtlasLayout};
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::EguiContexts;

use tomb_of_heroes_core::config::TopologyConfig;
use tomb_of_heroes_core::rng::DungeonMasterSeed;
use tomb_of_heroes_core::topology::gen::{generate_procedural_dungeon, DungeonGeneratorConfig};
use tomb_of_heroes_core::topology::links::VerticalLinkKind;
use tomb_of_heroes_core::{
    CorpseState, DungeonGrid, FloorId, GridCoord, GuildIntelRegister, HeroClass, HeroKnowledgeMap,
    LogicId, TileVisibility, WorldCoord,
};

use crate::asset_gen::{ensure_dungeon_sheet_exists, SpriteIndex};
use crate::camera::PixelCamera;
use crate::simulation::WorldSimulation;

// ─────────────────────────────────────────────────────────────────────────────
// Z-Layers Stacking Heights (SPEC-REQ-FRONT-003)
// ─────────────────────────────────────────────────────────────────────────────
pub const Z_FLOOR: f32 = 0.0;
pub const Z_GROUND_DECORS: f32 = 1.0;
pub const Z_CORPSES: f32 = 2.0;
pub const Z_CREATURES: f32 = 3.0;
pub const Z_WALLS: f32 = 4.0;
pub const Z_FOG: f32 = 5.0;
pub const Z_OVERLAYS: f32 = 6.0;
pub const Z_CURSOR: f32 = 7.0;

/// Conversion from discrete grid coordinates to Bevy world space.
///
/// Implements `SPEC-REQ-FRONT-003`:
/// $$X_{\text{monde}} = x \times 16.0, \quad Y_{\text{monde}} = -y \times 16.0$$
#[inline]
#[must_use]
pub fn grid_to_world(coord: GridCoord, z: f32) -> Vec3 {
    Vec3::new(coord.x as f32 * 16.0, -(coord.y as f32) * 16.0, z)
}

/// Conversion from Bevy world space coordinates to discrete grid coordinates.
#[inline]
#[must_use]
pub fn world_to_grid(world_pos: Vec2) -> GridCoord {
    let x = ((world_pos.x + 8.0) / 16.0).floor() as i32;
    let y = ((-world_pos.y + 8.0) / 16.0).floor() as i32;
    GridCoord::new(x, y)
}

// ─────────────────────────────────────────────────────────────────────────────
// Components & Resources
// ─────────────────────────────────────────────────────────────────────────────

/// Component binding a Bevy visual sprite entity to a deterministic `LogicId`.
///
/// Implements `SPEC-REQ-FRONT-003` (Bridge Pattern).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VisualEntityRef(pub LogicId);

/// Tag component identifying static floor and wall tile entities.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisualTile {
    pub floor: FloorId,
    pub coord: GridCoord,
}

/// Tag component identifying fog of war overlay tiles.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisualFogTile {
    pub floor: FloorId,
    pub coord: GridCoord,
}

/// Tag component identifying the active tactical selection cursor.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct VisualCursor;

/// Shared handles to the procedural sprite atlas texture and layout.
#[derive(Resource, Debug, Clone)]
pub struct SpriteAtlasResource {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

/// Shared handle to the dark and macabre game logo texture.
#[derive(Resource, Debug, Clone)]
pub struct GameLogoResource {
    pub texture: Handle<Image>,
}

/// Resource designating the active floor visualized by the player.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveFloor(pub FloorId);

impl Default for ActiveFloor {
    fn default() -> Self {
        Self(FloorId(0))
    }
}

/// Resource storing user selection on the tactical dungeon grid.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SelectedTile {
    pub coord: Option<GridCoord>,
    pub entity_id: Option<LogicId>,
}

/// Resource managing multi-floor dungeon layout topology.
#[derive(Resource, Debug, Clone)]
pub struct DungeonTopologyResource {
    pub dungeon: DungeonGrid,
}

impl Default for DungeonTopologyResource {
    fn default() -> Self {
        let config = DungeonGeneratorConfig::default();
        let topo_config = TopologyConfig::default();
        let seed = DungeonMasterSeed(42);
        let dungeon = match generate_procedural_dungeon(seed, &config, &topo_config) {
            Ok(gen) => gen.grid,
            Err(_) => DungeonGrid::new(),
        };

        Self { dungeon }
    }
}

/// Resource tracking collective adventurer spatial knowledge.
#[derive(Resource, Debug, Clone)]
pub struct HeroKnowledgeResource {
    pub knowledge: HeroKnowledgeMap,
}

impl Default for HeroKnowledgeResource {
    fn default() -> Self {
        let mut knowledge = HeroKnowledgeMap::new();
        // Initially visible central area around starting adventurers
        for y in -2..=2 {
            for x in -2..=2 {
                knowledge.set_visibility(GridCoord::new(x, y), TileVisibility::InSight);
            }
        }
        // Surrounding explored area
        for y in -3..=3 {
            for x in -3..=3 {
                let coord = GridCoord::new(x, y);
                if knowledge.visibility(coord) == TileVisibility::Unexplored {
                    knowledge.set_visibility(coord, TileVisibility::Explored);
                }
            }
        }
        Self { knowledge }
    }
}

/// Resource managing guild intelligence and compromised sectors.
#[derive(Resource, Debug, Default, Clone)]
pub struct GuildIntelResource {
    pub intel: GuildIntelRegister,
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugins
// ─────────────────────────────────────────────────────────────────────────────

/// Plugin handling static dungeon geometry, fog of war, and tile selection.
#[derive(Debug, Default, Clone, Copy)]
pub struct TilemapRenderPlugin;

impl Plugin for TilemapRenderPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<ActiveFloor>() {
            app.init_resource::<ActiveFloor>();
        }
        if !app.world().contains_resource::<SelectedTile>() {
            app.init_resource::<SelectedTile>();
        }
        if !app.world().contains_resource::<DungeonTopologyResource>() {
            app.init_resource::<DungeonTopologyResource>();
        }
        if !app.world().contains_resource::<HeroKnowledgeResource>() {
            app.init_resource::<HeroKnowledgeResource>();
        }
        if !app.world().contains_resource::<GuildIntelResource>() {
            app.init_resource::<GuildIntelResource>();
        }

        app.add_systems(Startup, setup_tilemap_system);
        app.add_systems(
            Update,
            (
                update_tilemap_on_floor_change_system,
                update_fog_and_intel_overlay_system,
                update_cursor_system,
                tile_selection_input_system,
            ),
        );
    }
}

/// Plugin synchronizing dynamic entities (heroes, monsters, corpses) from `LogicWorld`.
#[derive(Debug, Default, Clone, Copy)]
pub struct EntityRenderPlugin;

impl Plugin for EntityRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_entities_from_core_system);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems: Tilemap & Fog
// ─────────────────────────────────────────────────────────────────────────────

/// Initializes the sprite atlas and spawns initial dungeon geometry.
pub fn setup_tilemap_system(
    mut commands: Commands,
    images: Option<ResMut<Assets<Image>>>,
    texture_atlas_layouts: Option<ResMut<Assets<TextureAtlasLayout>>>,
    active_floor: Res<ActiveFloor>,
    topology: Res<DungeonTopologyResource>,
) {
    let (Some(mut images), Some(mut texture_atlas_layouts)) = (images, texture_atlas_layouts)
    else {
        return;
    };

    // Ensure procedural sprite sheet exists on disk for external tools / debug
    let sheet_path = Path::new("assets/textures/dungeon_sheet.png");
    let _ = ensure_dungeon_sheet_exists(sheet_path);

    // Build the in-memory Image directly from procedural DB16 generator (100% resilient across platforms)
    let img = crate::asset_gen::generate_dungeon_sheet_image();
    let image = Image::new(
        Extent3d {
            width: img.width(),
            height: img.height(),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        img.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let texture = images.add(image);
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 4, None, None);
    let layout_handle = texture_atlas_layouts.add(layout);

    let atlas = SpriteAtlasResource {
        texture,
        layout: layout_handle,
    };
    commands.insert_resource(atlas.clone());

    // Register embedded dark & macabre game logo resource
    let logo_image = crate::asset_gen::load_game_logo_image();
    let logo_handle = images.add(logo_image);
    commands.insert_resource(GameLogoResource {
        texture: logo_handle,
    });

    // Spawn tiles for active floor
    spawn_floor_tiles(&mut commands, &atlas, active_floor.0, &topology.dungeon);

    // Spawn tactical selection cursor (initially hidden)
    commands.spawn((
        VisualCursor,
        Sprite {
            image: atlas.texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: atlas.layout.clone(),
                index: SpriteIndex::TileCursor.index(),
            }),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, Z_CURSOR),
        Visibility::Hidden,
    ));
}

/// Spawns floor dalles, walls, and fog tiles for the specified floor.
fn spawn_floor_tiles(
    commands: &mut Commands,
    atlas: &SpriteAtlasResource,
    floor_id: FloorId,
    dungeon: &DungeonGrid,
) {
    let Some(floor_grid) = dungeon.floor(floor_id) else {
        return;
    };

    let min_coord = floor_grid.min_coord();
    let width = floor_grid.width() as i32;
    let height = floor_grid.height() as i32;

    for dy in 0..height {
        for dx in 0..width {
            let coord = GridCoord::new(min_coord.x + dx, min_coord.y + dy);
            let is_passable = floor_grid.is_passable(coord);

            let (sprite_index, z_layer) = if is_passable {
                let world_coord = WorldCoord::new(floor_id, coord);
                let links = dungeon.links_at(world_coord);
                if let Some(link) = links.first() {
                    match link.kind {
                        VerticalLinkKind::Stairs | VerticalLinkKind::Ladder => {
                            if link.destination.floor > floor_id {
                                (SpriteIndex::StairsDown.index(), Z_GROUND_DECORS)
                            } else {
                                (SpriteIndex::StairsUp.index(), Z_GROUND_DECORS)
                            }
                        }
                        VerticalLinkKind::Pitfall => {
                            (SpriteIndex::AcidPit.index(), Z_GROUND_DECORS)
                        }
                        VerticalLinkKind::OneWayPortal => {
                            (SpriteIndex::SanctifiedFloor.index(), Z_FLOOR)
                        }
                    }
                } else if floor_id == FloorId(2) && coord.x == 12 && coord.y == 12 {
                    (SpriteIndex::SanctifiedFloor.index(), Z_FLOOR)
                } else {
                    (SpriteIndex::FloorTile.index(), Z_FLOOR)
                }
            } else {
                (SpriteIndex::WallTile.index(), Z_WALLS)
            };

            let pos = grid_to_world(coord, z_layer);

            // Spawn base terrain tile
            commands.spawn((
                VisualTile {
                    floor: floor_id,
                    coord,
                },
                Sprite {
                    image: atlas.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: sprite_index,
                    }),
                    ..Default::default()
                },
                Transform::from_translation(pos),
            ));

            // Spawn fog tile overlay
            let fog_pos = grid_to_world(coord, Z_FOG);
            commands.spawn((
                VisualFogTile {
                    floor: floor_id,
                    coord,
                },
                Sprite {
                    image: atlas.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: SpriteIndex::FogUnexplored.index(),
                    }),
                    ..Default::default()
                },
                Transform::from_translation(fog_pos),
            ));
        }
    }
}

/// Rebuilds tilemap when the active floor changes.
pub fn update_tilemap_on_floor_change_system(
    mut commands: Commands,
    active_floor: Res<ActiveFloor>,
    atlas_res: Option<Res<SpriteAtlasResource>>,
    topology: Res<DungeonTopologyResource>,
    tile_query: Query<(Entity, &VisualTile)>,
    fog_query: Query<(Entity, &VisualFogTile)>,
) {
    if !active_floor.is_changed() {
        return;
    }
    let Some(atlas) = atlas_res else {
        return;
    };

    // Despawn previous floor tiles
    for (entity, _) in &tile_query {
        commands.entity(entity).despawn();
    }
    for (entity, _) in &fog_query {
        commands.entity(entity).despawn();
    }

    // Spawn new floor tiles
    spawn_floor_tiles(&mut commands, &atlas, active_floor.0, &topology.dungeon);
}

/// Updates fog of war and intel overlay visibility on the dungeon.
///
/// Implements `SPEC-REQ-FRONT-004`.
pub fn update_fog_and_intel_overlay_system(
    active_floor: Res<ActiveFloor>,
    _knowledge_res: Res<HeroKnowledgeResource>,
    intel_res: Res<GuildIntelResource>,
    mut fog_query: Query<(&VisualFogTile, &mut Sprite, &mut Visibility)>,
) {
    for (fog_tile, mut sprite, mut visibility) in &mut fog_query {
        if fog_tile.floor != active_floor.0 {
            continue;
        }

        let world_coord = WorldCoord::new(fog_tile.floor, fog_tile.coord);

        // Check Guild Intel compromised sector overlay (K >= 128)
        if intel_res.intel.known_tiles.contains(&world_coord) {
            *visibility = Visibility::Inherited;
            if let Some(ref mut atlas) = sprite.texture_atlas {
                atlas.index = SpriteIndex::IntelCompromised.index();
            }
            continue;
        }

        // Dungeon Master Omniscient Vision:
        // As dungeon master, the player sees the entire dungeon layout.
        // Unexplored fog is hidden so floor tiles, walls, and decors are crisp and visible.
        *visibility = Visibility::Hidden;
    }
}

/// Updates the position and visibility of the selection cursor.
pub fn update_cursor_system(
    selected: Res<SelectedTile>,
    mut cursor_query: Query<(&mut Transform, &mut Visibility), With<VisualCursor>>,
) {
    let Ok((mut transform, mut visibility)) = cursor_query.get_single_mut() else {
        return;
    };

    if let Some(coord) = selected.coord {
        transform.translation = grid_to_world(coord, Z_CURSOR);
        *visibility = Visibility::Inherited;
    } else {
        *visibility = Visibility::Hidden;
    }
}

/// Detects mouse click or single-finger tap to select a dungeon tile.
#[allow(clippy::too_many_arguments)]
pub fn tile_selection_input_system(
    mut touch_events: EventReader<TouchInput>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<PixelCamera>>,
    simulation: Res<WorldSimulation>,
    active_floor: Res<ActiveFloor>,
    mut selected: ResMut<SelectedTile>,
    mut contexts: EguiContexts,
    hud_state: Option<ResMut<crate::ui::HudState>>,
) {
    let ctx = contexts.ctx_mut();
    if ctx.wants_pointer_input() {
        return;
    }

    let Ok(window) = window_query.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let mut tap_pos = None;

    // Check Touch Tap (Ended phase without major movement)
    for ev in touch_events.read() {
        if ev.phase == TouchPhase::Ended {
            tap_pos = Some(ev.position);
            break;
        }
    }

    // Check Mouse Click (Left button just pressed)
    if tap_pos.is_none() && mouse_button.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            tap_pos = Some(cursor_pos);
        }
    }

    let Some(screen_pos) = tap_pos else {
        return;
    };

    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
        let grid_coord = world_to_grid(world_pos);
        selected.coord = Some(grid_coord);

        // Check if a hero is at this tile
        let mut found_entity = None;
        for (hero_id, hero) in simulation.heroes() {
            if hero.position.floor == active_floor.0 && hero.position.coord == grid_coord {
                found_entity = Some(*hero_id);
                break;
            }
        }

        // If no hero, check corpses
        if found_entity.is_none() {
            for (id, pos, _) in simulation.corpses().iter() {
                if pos == grid_coord {
                    found_entity = Some(id);
                    break;
                }
            }
        }

        selected.entity_id = found_entity;

        if let Some(mut hud) = hud_state {
            if hud.selected_tool == crate::ui::PlacementTool::None {
                hud.show_inspector = true;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems: Dynamic Entity Sync (SPEC-REQ-FRONT-003)
// ─────────────────────────────────────────────────────────────────────────────

/// Synchronizes heroes, monsters, and corpses from the deterministic Core into Bevy sprites.
///
/// Implements `SPEC-REQ-FRONT-003`.
pub fn sync_entities_from_core_system(
    mut commands: Commands,
    simulation: Res<WorldSimulation>,
    active_floor: Res<ActiveFloor>,
    atlas_res: Option<Res<SpriteAtlasResource>>,
    mut entity_query: Query<(Entity, &VisualEntityRef, &mut Transform, &mut Sprite)>,
) {
    let Some(atlas) = atlas_res else {
        return;
    };

    let mut tracked_ids = std::collections::BTreeSet::new();

    // 1. Synchronize Living Heroes
    for (hero_id, hero) in simulation.heroes() {
        if hero.position.floor != active_floor.0 {
            continue;
        }
        tracked_ids.insert(*hero_id);

        let target_pos = grid_to_world(hero.position.coord, Z_CREATURES);
        let sprite_idx = if hero.terror_bps.0 >= 8000 {
            SpriteIndex::HeroPanicked.index()
        } else {
            match hero.hero_class {
                HeroClass::Warrior => SpriteIndex::HeroWarrior.index(),
                HeroClass::Cleric => SpriteIndex::HeroCleric.index(),
                HeroClass::Paladin => SpriteIndex::HeroPaladin.index(),
                HeroClass::Mage => SpriteIndex::HeroMage.index(),
                HeroClass::Rogue => SpriteIndex::HeroRogue.index(),
            }
        };

        let mut found = false;
        for (_, visual_ref, mut transform, mut sprite) in &mut entity_query {
            if visual_ref.0 == *hero_id {
                transform.translation = target_pos;
                if let Some(ref mut texture_atlas) = sprite.texture_atlas {
                    texture_atlas.index = sprite_idx;
                }
                found = true;
                break;
            }
        }

        if !found {
            commands.spawn((
                VisualEntityRef(*hero_id),
                Sprite {
                    image: atlas.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: sprite_idx,
                    }),
                    ..Default::default()
                },
                Transform::from_translation(target_pos),
            ));
        }
    }

    // 2. Synchronize Fallen Corpses
    for (corpse_id, pos, corpse) in simulation.corpses().iter() {
        tracked_ids.insert(corpse_id);

        let target_pos = grid_to_world(pos, Z_CORPSES);
        let sprite_idx = match corpse.state {
            CorpseState::Intact => SpriteIndex::CorpseIntact.index(),
            CorpseState::Damaged => SpriteIndex::CorpseDamaged.index(),
            CorpseState::Destroyed => SpriteIndex::CorpseBones.index(),
        };

        let mut found = false;
        for (_, visual_ref, mut transform, mut sprite) in &mut entity_query {
            if visual_ref.0 == corpse_id {
                transform.translation = target_pos;
                if let Some(ref mut texture_atlas) = sprite.texture_atlas {
                    texture_atlas.index = sprite_idx;
                }
                found = true;
                break;
            }
        }

        if !found {
            commands.spawn((
                VisualEntityRef(corpse_id),
                Sprite {
                    image: atlas.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: sprite_idx,
                    }),
                    ..Default::default()
                },
                Transform::from_translation(target_pos),
            ));
        }
    }

    // 3. Despawn destroyed or disappeared entities
    for (entity, visual_ref, _, _) in &entity_query {
        if !tracked_ids.contains(&visual_ref.0) {
            commands.entity(entity).despawn();
        }
    }
}
