//! Tactile HUD, Chronomancy Controls, and Save Profile Interface with `bevy_egui`.
//!
//! Complies with `SPEC-REQ-FRONT-005` (Touch Target Size >= 44x44 pt).

use bevy::app::{App, Plugin, Update};
use bevy::ecs::system::{Res, ResMut, Resource};
use bevy_egui::egui::{self, Color32, ProgressBar, RichText, Vec2};
use bevy_egui::EguiContexts;

use tomb_of_heroes_core::campaign::WavePhase;
use tomb_of_heroes_core::chrono::memory::ChronoHero;
use tomb_of_heroes_core::math::BasisPoints;
use tomb_of_heroes_core::necro::{CorpseState, HeroClass};
use tomb_of_heroes_core::rng::DungeonMasterSeed;
use tomb_of_heroes_core::save::{
    export_to_base64_string, import_from_base64_string, pack_world, unpack_world,
    DEFAULT_CAMPAIGN_ID, DEFAULT_COMPRESSION_LEVEL,
};
use tomb_of_heroes_core::topology::{FloorId, TileOpacity, WorldCoord};

use crate::render::{ActiveFloor, DungeonTopologyResource, SelectedTile};
use crate::simulation::WorldSimulation;

/// Minimum physical touch target dimension (Apple HIG & Material Design standard).
pub const MIN_TOUCH_TARGET_SIZE: Vec2 = Vec2::new(44.0, 44.0);

/// Active tactical placement tool selected in the HUD action bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlacementTool {
    #[default]
    None,
    Wall,
    Spikes,
    Acid,
    Skeleton,
    Zombie,
}

/// HUD interactive state resource.
#[derive(Resource, Debug, Clone)]
pub struct HudState {
    pub mana: u32,
    pub max_mana: u32,
    pub infamy: u32,
    pub alert_level: u32,
    pub is_paused: bool,
    pub selected_tool: PlacementTool,
    pub show_inspector: bool,
    pub show_about_modal: bool,
    pub show_chrono_window: bool,
    pub rewind_ticks: u64,
    pub show_save_modal: bool,
    pub save_export_text: String,
    pub save_import_text: String,
    pub save_feedback: Option<String>,
    pub wave: u32,
    pub max_waves: u32,
    pub wave_phase: WavePhase,
    pub heart_hp: u32,
    pub heart_max_hp: u32,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            mana: 85,
            max_mana: 100,
            infamy: 120,
            alert_level: 2, // 0: Normal, 1: Suspicieux, 2: Élevé, 3: Critique
            is_paused: false,
            selected_tool: PlacementTool::None,
            show_inspector: false,
            show_about_modal: false,
            show_chrono_window: false,
            rewind_ticks: 20,
            show_save_modal: false,
            save_export_text: String::new(),
            save_import_text: String::new(),
            save_feedback: None,
            wave: 1,
            max_waves: 5,
            wave_phase: WavePhase::Preparation,
            heart_hp: 500,
            heart_max_hp: 500,
        }
    }
}

/// Helper creating a standardized button that enforces the >= 44x44 pt touch target size.
pub fn touch_btn(text: impl Into<egui::WidgetText>) -> egui::Button<'static> {
    egui::Button::new(text).min_size(MIN_TOUCH_TARGET_SIZE)
}

/// Plugin providing the tactile `bevy_egui` heads-up display.
#[derive(Debug, Default, Clone, Copy)]
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<HudState>() {
            app.init_resource::<HudState>();
        }
        app.add_systems(
            Update,
            (
                hud_status_bar_system,
                hud_action_bar_system,
                hud_chronomancy_window_system,
                hud_save_modal_system,
                hud_inspector_panel_system,
                hud_placement_execution_system,
                hud_about_modal_system,
                hud_end_game_modal_system,
            ),
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems: Top Status Bar
// ─────────────────────────────────────────────────────────────────────────────

pub fn hud_status_bar_system(
    mut contexts: EguiContexts,
    mut hud_state: ResMut<HudState>,
    mut active_floor: ResMut<ActiveFloor>,
    simulation: Res<WorldSimulation>,
    logo_res: Option<Res<crate::render::GameLogoResource>>,
) {
    let logo_tex_id = logo_res
        .as_ref()
        .map(|res| contexts.add_image(res.texture.clone()));
    let ctx = contexts.ctx_mut();

    hud_state.mana = simulation.mana();
    hud_state.max_mana = simulation.max_mana();
    hud_state.infamy = simulation.campaign().infamy;
    hud_state.wave = simulation.campaign().current_wave;
    hud_state.max_waves = simulation.campaign().max_waves;
    hud_state.wave_phase = simulation.campaign().phase;
    hud_state.heart_hp = simulation.heart().current_hp;
    hud_state.heart_max_hp = simulation.heart().max_hp;

    egui::TopBottomPanel::top("status_bar")
        .frame(
            egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin {
                left: 14.0,
                right: 14.0,
                top: 36.0, // Safe margin for mobile system status bar and camera notch
                bottom: 6.0,
            }),
        )
        .show(ctx, |ui| {
            // Row 1: Brand & Logo | Wave Pill | Quick Actions (Pause, Chrono, Save)
            ui.horizontal(|ui| {
                // Brand button with dark & macabre logo image
                let brand_clicked = if let Some(tex_id) = logo_tex_id {
                    let img = egui::Image::new(egui::load::SizedTexture::new(
                        tex_id,
                        egui::vec2(24.0, 24.0),
                    ));
                    ui.add(
                        egui::Button::image_and_text(
                            img,
                            RichText::new("TOMB OF HEROES")
                                .color(Color32::from_rgb(0x00, 0xE5, 0xFF))
                                .strong()
                                .size(12.0),
                        )
                        .min_size(egui::vec2(44.0, 44.0)),
                    )
                    .on_hover_text("Le Sanctuaire & Lore")
                    .clicked()
                } else {
                    ui.add(touch_btn(
                        RichText::new("💀 TOMB OF HEROES")
                            .color(Color32::from_rgb(0x00, 0xE5, 0xFF))
                            .strong()
                            .size(12.0),
                    ))
                    .on_hover_text("Le Sanctuaire & Lore")
                    .clicked()
                };

                if brand_clicked {
                    hud_state.show_about_modal = !hud_state.show_about_modal;
                }

                // Wave & Phase Pill centered
                let (phase_str, phase_color) = match hud_state.wave_phase {
                    WavePhase::Preparation => ("PRÉP", Color32::from_rgb(0x5A, 0xC5, 0x4F)),
                    WavePhase::Incursion => ("ASSAUT", Color32::from_rgb(0xFF, 0x44, 0x44)),
                    WavePhase::Debriefing => ("FIN", Color32::from_rgb(0xF4, 0xB4, 0x1B)),
                    WavePhase::Victory => ("VICTOIRE", Color32::from_rgb(0x00, 0xE5, 0xFF)),
                    WavePhase::Defeat => ("DÉFAITE", Color32::from_rgb(0xFF, 0x00, 0x00)),
                };
                ui.label(
                    RichText::new(format!(
                        "V.{}/{} [{}]",
                        hud_state.wave, hud_state.max_waves, phase_str
                    ))
                    .color(phase_color)
                    .strong()
                    .size(12.0),
                );

                // Right-aligned quick action buttons
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(touch_btn("💾"))
                        .on_hover_text("Sauvegardes")
                        .clicked()
                    {
                        hud_state.show_save_modal = !hud_state.show_save_modal;
                    }

                    let chrono_icon = if hud_state.show_chrono_window {
                        "⏳X"
                    } else {
                        "⏳"
                    };
                    if ui
                        .add(touch_btn(chrono_icon))
                        .on_hover_text("Chronomancie")
                        .clicked()
                    {
                        hud_state.show_chrono_window = !hud_state.show_chrono_window;
                    }

                    let pause_icon = if hud_state.is_paused { "▶" } else { "⏸" };
                    if ui
                        .add(touch_btn(pause_icon))
                        .on_hover_text("Pause / Play")
                        .clicked()
                    {
                        hud_state.is_paused = !hud_state.is_paused;
                    }
                });
            });

            ui.add_space(2.0);

            // Row 2: Vitals Gauges (Mana, Heart) | Floor Dropdown & Tick
            ui.horizontal(|ui| {
                // Mana Bar
                let mana_ratio = if hud_state.max_mana > 0 {
                    (hud_state.mana as f32 / hud_state.max_mana as f32).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                ui.add(
                    ProgressBar::new(mana_ratio)
                        .text(format!("💧 {}", hud_state.mana))
                        .desired_width(75.0),
                );

                // Heart Bar
                let heart_ratio = if hud_state.heart_max_hp > 0 {
                    (hud_state.heart_hp as f32 / hud_state.heart_max_hp as f32).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                ui.add(
                    ProgressBar::new(heart_ratio)
                        .text(format!("❤️ {}", hud_state.heart_hp))
                        .desired_width(75.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let tick = simulation.current_tick().0;
                    ui.label(RichText::new(format!("T:{tick}")).monospace().size(11.0));

                    egui::ComboBox::from_id_salt("floor_dropdown")
                        .selected_text(format!("Étage {} ▾", active_floor.0 .0))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut active_floor.0,
                                FloorId(0),
                                "Étage 0 : Sanctuaire",
                            );
                            ui.selectable_value(
                                &mut active_floor.0,
                                FloorId(1),
                                "Étage 1 : Catacombes",
                            );
                            ui.selectable_value(
                                &mut active_floor.0,
                                FloorId(2),
                                "Étage 2 : Crypte du Cœur",
                            );
                        });
                });
            });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems: Bottom Action Bar
// ─────────────────────────────────────────────────────────────────────────────

pub fn hud_action_bar_system(
    mut contexts: EguiContexts,
    mut hud_state: ResMut<HudState>,
    mut simulation: ResMut<WorldSimulation>,
    _active_floor: Res<ActiveFloor>,
    tool_icons: Option<Res<crate::render::ToolIconsResource>>,
) {
    let tool_tex = tool_icons.as_ref().map(|icons| {
        (
            contexts.add_image(icons.wall.clone()),
            contexts.add_image(icons.spikes.clone()),
            contexts.add_image(icons.acid.clone()),
            contexts.add_image(icons.skeleton.clone()),
            contexts.add_image(icons.zombie.clone()),
        )
    });
    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::bottom("action_bar")
        .frame(
            egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin {
                left: 16.0,
                right: 16.0,
                top: 8.0,
                bottom: 24.0, // Safe padding for rounded display corners and Android gesture bar
            }),
        )
        .show(ctx, |ui| {
            // Line 1: Contextual Info Banner for Selected Placement Tool
            let (title, cost_str, desc) = match hud_state.selected_tool {
                PlacementTool::Wall => (
                    "Mur de Pierre",
                    "10 💧",
                    "Bloque le passage et force un détour.",
                ),
                PlacementTool::Spikes => (
                    "Piège à Piques",
                    "15 💧",
                    "Inflige 25 dégâts physiques aux intrus.",
                ),
                PlacementTool::Acid => (
                    "Fosse d'Acide",
                    "25 💧",
                    "Dégâts continus et dégradation d'armure.",
                ),
                PlacementTool::Skeleton => (
                    "Squelette Guerrier",
                    "20 💧",
                    "Gardien de base patrouillant le secteur.",
                ),
                PlacementTool::Zombie => (
                    "Zombie Pestilentiel",
                    "35 💧",
                    "Colosse résistant qui diffuse l'effroi.",
                ),
                PlacementTool::None => (
                    "Mode Tactique",
                    "-",
                    "Touchez un outil pour le poser, ou une case pour l'inspecter.",
                ),
            };

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(title)
                        .color(Color32::from_rgb(0xFF, 0xDD, 0x55))
                        .size(13.0)
                        .strong(),
                );
                if hud_state.selected_tool != PlacementTool::None {
                    ui.label(
                        RichText::new(format!("• Coût: {cost_str}"))
                            .color(Color32::from_rgb(0x41, 0x7E, 0xBD))
                            .size(12.0)
                            .strong(),
                    );
                }
            });

            ui.label(
                RichText::new(desc)
                    .color(Color32::from_rgb(0xCF, 0xCF, 0xCF))
                    .size(11.0),
            );

            ui.add_space(4.0);

            // Line 2: Scrollable Tools (Left) + Pinned Action Button (Right)
            let avail_w = ui.available_width();
            let action_btn_w = 80.0;
            let spacing = 8.0;
            let tools_w = (avail_w - action_btn_w - spacing).max(120.0);

            ui.horizontal(|ui| {
                // Left: Single-Row Horizontal Scroll Area with strict max_width
                ui.allocate_ui_with_layout(
                    egui::vec2(tools_w, 48.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        egui::ScrollArea::horizontal()
                            .max_width(tools_w)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let tool_data = [
                                        (
                                            PlacementTool::Wall,
                                            "10💧",
                                            tool_tex.as_ref().map(|t| t.0),
                                        ),
                                        (
                                            PlacementTool::Spikes,
                                            "15💧",
                                            tool_tex.as_ref().map(|t| t.1),
                                        ),
                                        (
                                            PlacementTool::Acid,
                                            "25💧",
                                            tool_tex.as_ref().map(|t| t.2),
                                        ),
                                        (
                                            PlacementTool::Skeleton,
                                            "20💧",
                                            tool_tex.as_ref().map(|t| t.3),
                                        ),
                                        (
                                            PlacementTool::Zombie,
                                            "35💧",
                                            tool_tex.as_ref().map(|t| t.4),
                                        ),
                                    ];

                                    for (tool, cost, opt_tex) in tool_data {
                                        let is_active = hud_state.selected_tool == tool;

                                        let btn_response = if let Some(tex_id) = opt_tex {
                                            let img =
                                                egui::Image::new(egui::load::SizedTexture::new(
                                                    tex_id,
                                                    egui::vec2(24.0, 24.0),
                                                ));
                                            let text_color = if is_active {
                                                Color32::YELLOW
                                            } else {
                                                Color32::WHITE
                                            };
                                            let btn = egui::Button::image_and_text(
                                                img,
                                                RichText::new(cost).color(text_color).size(11.0),
                                            )
                                            .min_size(egui::vec2(52.0, 44.0))
                                            .selected(is_active);
                                            ui.add(btn)
                                        } else {
                                            let btn = egui::Button::new(
                                                RichText::new(cost)
                                                    .color(if is_active {
                                                        Color32::YELLOW
                                                    } else {
                                                        Color32::WHITE
                                                    })
                                                    .size(12.0),
                                            )
                                            .min_size(egui::vec2(44.0, 44.0))
                                            .selected(is_active);
                                            ui.add(btn)
                                        };

                                        if btn_response.clicked() {
                                            hud_state.selected_tool =
                                                if is_active { PlacementTool::None } else { tool };
                                        }
                                    }
                                });
                            });
                    },
                );

                // Right: Pinned Action / Attack Button
                ui.allocate_ui_with_layout(
                    egui::vec2(action_btn_w, 48.0),
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| match hud_state.wave_phase {
                        WavePhase::Preparation => {
                            let btn = egui::Button::new(
                                RichText::new("⚔ ASSAUT")
                                    .color(Color32::from_rgb(0xFF, 0xDD, 0x55))
                                    .size(12.0)
                                    .strong(),
                            )
                            .fill(Color32::from_rgb(0x60, 0x1A, 0x1A))
                            .min_size(egui::vec2(action_btn_w, 44.0));
                            if ui.add(btn).clicked() {
                                simulation.world_mut().start_incursion();
                            }
                        }
                        WavePhase::Incursion => {
                            let btn = egui::Button::new(
                                RichText::new("⏭ STEP")
                                    .color(Color32::from_rgb(0x55, 0xFF, 0x55))
                                    .size(12.0)
                                    .strong(),
                            )
                            .fill(Color32::from_rgb(0x1B, 0x4D, 0x2E))
                            .min_size(egui::vec2(action_btn_w, 44.0));
                            if ui.add(btn).clicked() {
                                simulation.step();
                            }
                        }
                        WavePhase::Debriefing => {
                            let btn = egui::Button::new(
                                RichText::new("➡ VAGUE")
                                    .color(Color32::from_rgb(0x55, 0xFF, 0x55))
                                    .size(12.0)
                                    .strong(),
                            )
                            .fill(Color32::from_rgb(0x1B, 0x4D, 0x2E))
                            .min_size(egui::vec2(action_btn_w, 44.0));
                            if ui.add(btn).clicked() {
                                simulation.world_mut().campaign_mut().advance_to_next_wave();
                            }
                        }
                        WavePhase::Victory | WavePhase::Defeat => {}
                    },
                );
            });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems: Chronomancy Rewind Window
// ─────────────────────────────────────────────────────────────────────────────

pub fn hud_chronomancy_window_system(
    mut contexts: EguiContexts,
    mut hud_state: ResMut<HudState>,
    mut simulation: ResMut<WorldSimulation>,
) {
    if !hud_state.show_chrono_window {
        return;
    }

    let ctx = contexts.ctx_mut();
    let mut is_open = hud_state.show_chrono_window;

    egui::Window::new("⏳ Chronomancie — Rembobinage Temporel")
        .open(&mut is_open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Restaure l'état du donjon à un instant antérieur T - n.");

            ui.horizontal(|ui| {
                ui.label("Ticks à remonter:");
                ui.add(egui::Slider::new(&mut hud_state.rewind_ticks, 5..=100));
            });

            let n = hud_state.rewind_ticks;
            let mana_cost = ((n * 3) / 10).max(1) as u32;
            let paradox_risk = (n * 15).min(10_000) as u32;

            ui.separator();
            ui.label(format!("Coût en Mana: {mana_cost}"));
            ui.label(format!("Anxiété Paradoxale (Risque): {paradox_risk} BPS"));

            ui.separator();

            if ui
                .add(touch_btn(
                    RichText::new("Confirmer le Rembobinage")
                        .color(Color32::LIGHT_BLUE)
                        .strong(),
                ))
                .clicked()
                && hud_state.mana >= mana_cost
            {
                hud_state.mana = hud_state.mana.saturating_sub(mana_cost);

                // Inflict paradox anxiety onto chrono-aware heroes
                for hero in simulation.heroes_mut().values_mut() {
                    if hero.has_chrono_awareness {
                        hero.apply_paradox_anxiety(BasisPoints(paradox_risk));
                    }
                }

                hud_state.show_chrono_window = false;
            }
        });

    hud_state.show_chrono_window = is_open;
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems: Save Manager Modal
// ─────────────────────────────────────────────────────────────────────────────

pub fn hud_save_modal_system(
    mut contexts: EguiContexts,
    mut hud_state: ResMut<HudState>,
    mut simulation: ResMut<WorldSimulation>,
) {
    if !hud_state.show_save_modal {
        return;
    }

    let ctx = contexts.ctx_mut();
    let mut is_open = hud_state.show_save_modal;

    egui::Window::new("💾 Gestionnaire de Profils & Sauvegardes")
        .open(&mut is_open)
        .min_width(340.0)
        .show(ctx, |ui| {
            ui.label("Exportez votre progression en Base64 ou restaurez un profil.");

            ui.separator();

            if ui.add(touch_btn("Exporter l'état en Base64")).clicked() {
                match pack_world(
                    simulation.world(),
                    DEFAULT_CAMPAIGN_ID,
                    DEFAULT_COMPRESSION_LEVEL,
                ) {
                    Ok(envelope) => {
                        hud_state.save_export_text = export_to_base64_string(&envelope);
                        hud_state.save_feedback = Some("Exportation Base64 générée !".to_string());
                    }
                    Err(e) => {
                        hud_state.save_feedback = Some(format!("Erreur export: {e:?}"));
                    }
                }
            }

            if !hud_state.save_export_text.is_empty() {
                ui.label("Texte de sauvegarde exporté:");
                ui.add(
                    egui::TextEdit::multiline(&mut hud_state.save_export_text)
                        .desired_rows(3)
                        .font(egui::TextStyle::Monospace),
                );
            }

            ui.separator();

            ui.label("Importer une chaîne Base64:");
            ui.add(
                egui::TextEdit::multiline(&mut hud_state.save_import_text)
                    .desired_rows(3)
                    .font(egui::TextStyle::Monospace),
            );

            if ui.add(touch_btn("Charger la sauvegarde Base64")).clicked() {
                match import_from_base64_string(&hud_state.save_import_text) {
                    Ok(envelope) => match unpack_world(&envelope) {
                        Ok(world) => {
                            *simulation.world_mut() = world;
                            hud_state.save_feedback =
                                Some("Sauvegarde restaurée avec succès !".to_string());
                        }
                        Err(e) => {
                            hud_state.save_feedback = Some(format!("Erreur décompression: {e:?}"));
                        }
                    },
                    Err(e) => {
                        hud_state.save_feedback = Some(format!("Erreur décodage: {e:?}"));
                    }
                }
            }

            if let Some(ref msg) = hud_state.save_feedback {
                ui.separator();
                ui.label(RichText::new(msg).color(Color32::LIGHT_GREEN).strong());
            }
        });

    hud_state.show_save_modal = is_open;
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems: Tile & Entity Inspector Panel
// ─────────────────────────────────────────────────────────────────────────────

pub fn hud_inspector_panel_system(
    mut contexts: EguiContexts,
    mut hud_state: ResMut<HudState>,
    mut simulation: ResMut<WorldSimulation>,
    selected: Res<SelectedTile>,
    topology: Res<DungeonTopologyResource>,
    active_floor: Res<ActiveFloor>,
) {
    if !hud_state.show_inspector {
        return;
    }
    let Some(coord) = selected.coord else {
        return;
    };

    let ctx = contexts.ctx_mut();
    let mut is_open = hud_state.show_inspector;

    egui::Window::new("🔍 Inspecteur Tactique")
        .open(&mut is_open)
        .default_pos(egui::pos2(16.0, 110.0))
        .collapsible(true)
        .movable(true)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(RichText::new(format!("Case: ({}, {})", coord.x, coord.y)).strong());

            let is_passable = topology
                .dungeon
                .floor(active_floor.0)
                .is_some_and(|f| f.is_passable(coord));
            let terrain = if is_passable {
                "Sol de pierre"
            } else {
                "Mur de roche"
            };
            ui.label(format!("Terrain: {terrain}"));

            ui.separator();

            // Check if hero present
            let mut hero_found = false;
            for (&hero_id, hero) in simulation.heroes() {
                if hero.position.floor == active_floor.0 && hero.position.coord == coord {
                    hero_found = true;
                    ui.label(
                        RichText::new(format!("Aventurier: {:?}", hero.hero_class))
                            .color(Color32::LIGHT_BLUE)
                            .strong(),
                    );
                    ui.label(format!("Terreur: {}/10000 BPS", hero.terror_bps.0));
                    ui.label(format!("Conscience Chrono: {}", hero.has_chrono_awareness));

                    if ui.add(touch_btn("Infliger Terreur +1000 BPS")).clicked() {
                        if let Some(h) = simulation.hero_mut(hero_id) {
                            h.apply_paradox_anxiety(BasisPoints(1000));
                        }
                    }
                    break;
                }
            }

            // Check if corpse present
            if !hero_found {
                let corpse_entry = simulation
                    .corpses()
                    .iter()
                    .find(|(_, pos, _)| *pos == coord)
                    .map(|(id, _, c)| {
                        (
                            id,
                            c.hero_class,
                            c.state,
                            c.structural_hp,
                            c.soul_essence_value,
                        )
                    });

                if let Some((id, hero_class, state, structural_hp, soul_essence)) = corpse_entry {
                    ui.label(
                        RichText::new(format!("Dépouille: {hero_class:?}"))
                            .color(Color32::LIGHT_RED)
                            .strong(),
                    );
                    ui.label(format!("État: {state:?}"));
                    ui.label(format!("PV Structurels: {structural_hp}"));
                    ui.label(format!("Essence d'âme: {soul_essence}"));

                    if state != CorpseState::Destroyed
                        && ui.add(touch_btn("Relever Squelette (20 Mana)")).clicked()
                        && hud_state.mana >= 20
                    {
                        hud_state.mana = hud_state.mana.saturating_sub(20);
                        if let Some(c) = simulation.corpses_mut().get_corpse_mut(id) {
                            c.state = CorpseState::Destroyed;
                        }
                    }
                }
            }
        });

    hud_state.show_inspector = is_open;
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems: Placement Tool Execution
// ─────────────────────────────────────────────────────────────────────────────

pub fn hud_placement_execution_system(
    mut hud_state: ResMut<HudState>,
    mut simulation: ResMut<WorldSimulation>,
    mut topology: ResMut<DungeonTopologyResource>,
    active_floor: Res<ActiveFloor>,
    selected: Res<SelectedTile>,
) {
    if hud_state.selected_tool == PlacementTool::None {
        return;
    }
    let Some(coord) = selected.coord else {
        return;
    };

    // Execute placement when a tool is active and a tile is selected
    match hud_state.selected_tool {
        PlacementTool::Wall => {
            if let Some(floor) = topology.dungeon.floor_mut(active_floor.0) {
                let current = floor.is_passable(coord);
                let _ = floor.set_passable(coord, !current);
                let _ = floor.set_opacity(
                    coord,
                    if current {
                        TileOpacity::Opaque
                    } else {
                        TileOpacity::Transparent
                    },
                );
            }
            if let Some(floor) = simulation
                .world_mut()
                .dungeon_mut()
                .floor_mut(active_floor.0)
            {
                let current = floor.is_passable(coord);
                let _ = floor.set_passable(coord, !current);
                let _ = floor.set_opacity(
                    coord,
                    if current {
                        TileOpacity::Opaque
                    } else {
                        TileOpacity::Transparent
                    },
                );
            }
            hud_state.selected_tool = PlacementTool::None;
        }
        PlacementTool::Spikes => {
            if hud_state.mana >= 15 {
                hud_state.mana = hud_state.mana.saturating_sub(15);
                simulation.record_hazard(WorldCoord::new(active_floor.0, coord));
            }
            hud_state.selected_tool = PlacementTool::None;
        }
        PlacementTool::Acid => {
            if hud_state.mana >= 25 {
                hud_state.mana = hud_state.mana.saturating_sub(25);
                simulation.record_hazard(WorldCoord::new(active_floor.0, coord));
            }
            hud_state.selected_tool = PlacementTool::None;
        }
        PlacementTool::Skeleton => {
            if hud_state.mana >= 20 {
                hud_state.mana = hud_state.mana.saturating_sub(20);
                if let Ok(id) = simulation.allocate_id() {
                    let minion = ChronoHero::new_ordinary(
                        id,
                        HeroClass::Warrior,
                        WorldCoord::new(active_floor.0, coord),
                    );
                    simulation.register_hero(minion);
                }
            }
            hud_state.selected_tool = PlacementTool::None;
        }
        PlacementTool::Zombie => {
            if hud_state.mana >= 30 {
                hud_state.mana = hud_state.mana.saturating_sub(30);
                if let Ok(id) = simulation.allocate_id() {
                    let minion = ChronoHero::new_ordinary(
                        id,
                        HeroClass::Paladin,
                        WorldCoord::new(active_floor.0, coord),
                    );
                    simulation.register_hero(minion);
                }
            }
            hud_state.selected_tool = PlacementTool::None;
        }
        PlacementTool::None => {}
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems: About & Lore Modal (Dark & Macabre Game Logo)
// ─────────────────────────────────────────────────────────────────────────────

pub fn hud_about_modal_system(
    mut contexts: EguiContexts,
    mut hud_state: ResMut<HudState>,
    logo_res: Option<Res<crate::render::GameLogoResource>>,
) {
    if !hud_state.show_about_modal {
        return;
    }

    let texture_id = logo_res
        .as_ref()
        .map(|res| contexts.add_image(res.texture.clone()));
    let ctx = contexts.ctx_mut();
    let mut is_open = hud_state.show_about_modal;
    let mut close_requested = false;

    egui::Window::new("💀 TOMB OF HEROES — Le Sanctuaire")
        .open(&mut is_open)
        .default_pos(egui::pos2(24.0, 70.0))
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                if let Some(tex_id) = texture_id {
                    ui.image(egui::load::SizedTexture::new(tex_id, egui::vec2(180.0, 180.0)));
                    ui.add_space(6.0);
                }

                ui.label(
                    RichText::new("TOMB OF HEROES")
                        .size(18.0)
                        .strong()
                        .color(Color32::from_rgb(0x00, 0xE5, 0xFF)),
                );
                ui.label(
                    RichText::new("— Stratégie Déterministe & Nécromancie Tactique —")
                        .italics()
                        .color(Color32::from_rgb(0xA5, 0x30, 0x30)),
                );

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(
                    "Vous régnez sur les cryptes millénaires. Les aventuriers de la surface descendent piller votre Sanctuaire.\n\nConstruisez des remparts de roche, tendez des pièges d'acide mortels, relevez les cadavres des héros déchus en gardiens squelettes et zombies, et altérez le flux temporel pour triompher.",
                );

                ui.add_space(10.0);
                if ui.add(touch_btn("Fermer")).clicked() {
                    close_requested = true;
                }
            });
        });

    hud_state.show_about_modal = is_open && !close_requested;
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems: End Game Modal (Victory / Defeat)
// ─────────────────────────────────────────────────────────────────────────────

pub fn hud_end_game_modal_system(
    mut contexts: EguiContexts,
    mut hud_state: ResMut<HudState>,
    mut simulation: ResMut<WorldSimulation>,
    logo_res: Option<Res<crate::render::GameLogoResource>>,
) {
    let phase = simulation.campaign().phase;
    if !matches!(phase, WavePhase::Victory | WavePhase::Defeat) {
        return;
    }

    let texture_id = logo_res
        .as_ref()
        .map(|res| contexts.add_image(res.texture.clone()));
    let ctx = contexts.ctx_mut();
    let title = if phase == WavePhase::Victory {
        "🏆 VICTOIRE DE CAMPAGNE"
    } else {
        "💀 DÉFAITE DU DONJON"
    };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                if let Some(tex_id) = texture_id {
                    ui.image(egui::load::SizedTexture::new(
                        tex_id,
                        egui::vec2(120.0, 120.0),
                    ));
                    ui.add_space(4.0);
                }

                if phase == WavePhase::Victory {
                    ui.label(
                        RichText::new("Le Maître de Guilde a été terrassé et le donjon triomphe !")
                            .color(Color32::from_rgb(0x00, 0xE5, 0xFF))
                            .size(16.0)
                            .strong(),
                    );
                } else {
                    ui.label(
                        RichText::new("Le Cœur du Donjon a été anéanti par les aventuriers !")
                            .color(Color32::from_rgb(0xFF, 0x44, 0x44))
                            .size(16.0)
                            .strong(),
                    );
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            let stats = &simulation.campaign().stats;
            ui.label(RichText::new("📊 Bilan Récapitulatif :").strong());
            ui.label(format!("• Vagues repoussées : {}", stats.waves_cleared));
            ui.label(format!(
                "• Héros éliminés au total : {}",
                stats.heroes_killed_total
            ));
            for (class, count) in &stats.heroes_killed_by_class {
                ui.label(format!("   - {class:?} : {count}"));
            }
            ui.label(format!(
                "• Morts sous panique aveugle : {}",
                stats.heroes_died_of_panic
            ));
            ui.label(format!(
                "• Héros évadés : {} (dont {} indemnes)",
                stats.heroes_escaped_total, stats.heroes_escaped_unhurt
            ));
            ui.label(format!(
                "• Essence d'âme / mana récoltée : {}",
                stats.mana_harvested_total
            ));
            ui.label(format!(
                "• Gardiens morts-vivants créés : {}",
                stats.corpses_converted_total
            ));
            ui.label(format!(
                "• Rembobinages chronomantiques : {}",
                stats.rewinds_performed
            ));
            ui.label(format!(
                "• Paradoxe accumulé : {}",
                stats.paradox_accumulated
            ));
            ui.label(format!(
                "• Infamie finale : {}",
                simulation.campaign().infamy
            ));

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                // Tactile Button: Restart Game (>= 44x44 pt)
                if ui
                    .add(touch_btn(
                        RichText::new("🔄 Nouvelle Partie")
                            .color(Color32::from_rgb(0x5A, 0xC5, 0x4F))
                            .strong(),
                    ))
                    .clicked()
                {
                    let new_seed = simulation.current_tick().0.saturating_add(777);
                    let cfg = simulation.config().clone();
                    *simulation =
                        WorldSimulation::from_config_and_seed(cfg, DungeonMasterSeed(new_seed));
                }

                // Tactile Button: Export Save Base64 (>= 44x44 pt)
                if ui
                    .add(touch_btn(
                        RichText::new("📋 Exporter Sauvegarde")
                            .color(Color32::LIGHT_BLUE)
                            .strong(),
                    ))
                    .clicked()
                {
                    if let Ok(envelope) = pack_world(
                        simulation.world(),
                        DEFAULT_CAMPAIGN_ID,
                        DEFAULT_COMPRESSION_LEVEL,
                    ) {
                        hud_state.save_export_text = export_to_base64_string(&envelope);
                        hud_state.show_save_modal = true;
                    }
                }
            });
        });
}
