//! Engine and gameplay configuration module.
//!
//! Provides structured configuration parameters for all simulation systems,
//! enforcing the ZERO magic number architectural guard. All gameplay constants
//! (tick rates, damage thresholds, terror levels, retreat rules, chronomancy)
//! must be read from this module.

use serde::{Deserialize, Serialize};

use crate::math::BasisPoints;

/// Fixed-tick clock simulation parameters.
///
/// Specified in `SPEC-REQ-ARCH-001`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickConfig {
    /// Nominal fixed-tick frequency (20 Hz).
    pub rate_hz: u32,
    /// Logical tick duration in milliseconds (50 ms).
    pub duration_ms: u32,
    /// Anti-death-spiral cap on consecutive tick catch-ups per frame (5 ticks).
    pub max_catchup_ticks_per_frame: u32,
}

impl Default for TickConfig {
    fn default() -> Self {
        Self {
            rate_hz: 20,
            duration_ms: 50,
            max_catchup_ticks_per_frame: 5,
        }
    }
}

/// Corpse integrity, stacking, and degradation parameters.
///
/// Specified in `SPEC-REQ-NECRO-001`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpseConfig {
    /// Maximum number of corpses allowed on a single tile (3 corpses).
    pub max_per_tile: u32,
    /// Nominal structural hit points of an intact corpse (50 HP).
    pub nominal_structural_hp: u32,
    /// Hit points threshold below which a corpse is considered Damaged (25 HP).
    pub damaged_threshold_hp: u32,
    /// Default soul essence harvested from an intact corpse (25 essence).
    pub default_soul_essence: u32,
    /// Default base terror potency projected by an intact corpse (5_000 BPS = 50.00%).
    pub default_terror_potency: BasisPoints,
    /// Default decay ticks before corpse rots away (1_200 ticks = 60 seconds).
    pub default_decay_ticks: u32,
}

impl Default for CorpseConfig {
    fn default() -> Self {
        Self {
            max_per_tile: 3,
            nominal_structural_hp: 50,
            damaged_threshold_hp: 25,
            default_soul_essence: 25,
            default_terror_potency: BasisPoints(5_000),
            default_decay_ticks: 1_200,
        }
    }
}

/// Psychological terror thresholds and limits.
///
/// Specified in `SPEC-REQ-NECRO-002`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrorConfig {
    /// Threshold for Shaken state (3_000 points / 30.00%).
    pub shaken_threshold: u32,
    /// Threshold for Disrupted state (6_000 points / 60.00%).
    pub disrupted_threshold: u32,
    /// Threshold for Blind Panic / Rout state (8_500 points / 85.00%).
    pub blind_panic_threshold: u32,
    /// Maximum terror gauge capacity (10_000 points / 100.00%).
    pub max_points: u32,
    /// Maximum Chebyshev distance for corpse terror projection (5 tiles).
    pub max_projection_distance: u32,
    /// Simulation tick frequency used for terror accumulation normalization (20 Hz).
    pub tick_rate_hz: u32,
}

impl Default for TerrorConfig {
    fn default() -> Self {
        Self {
            shaken_threshold: 3_000,
            disrupted_threshold: 6_000,
            blind_panic_threshold: 8_500,
            max_points: 10_000,
            max_projection_distance: 5,
            tick_rate_hz: 20,
        }
    }
}

/// Asymmetric necromancy parameters, mana costs, and macabre explosion.
///
/// Specified in `SPEC-REQ-NECRO-003` and `SPEC-REQ-NECRO-004`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NecroConfig {
    /// Dark Mana cost to raise a Skeleton Guardian (40 Mana).
    pub skeleton_mana_cost: u32,
    /// Dark Mana cost to raise a Flesh-Wall Zombie (60 Mana).
    pub zombie_mana_cost: u32,
    /// Dark Mana cost to raise a Fallen Soul Spectre (80 Mana).
    pub spectre_mana_cost: u32,
    /// Direct damage dealt by Macabre Explosion (80 HP).
    pub explosion_damage: u32,
    /// Instant terror inflicted by Macabre Explosion (2_000 BPS = 20.00%).
    pub explosion_terror_potency: BasisPoints,
    /// Radius of Macabre Explosion in Chebyshev distance (2 tiles).
    pub explosion_radius: u32,
    /// Channel duration in ticks for cleric sanctification (60 ticks = 3.0s).
    pub sanctify_channel_ticks: u32,
}

impl Default for NecroConfig {
    fn default() -> Self {
        Self {
            skeleton_mana_cost: 40,
            zombie_mana_cost: 60,
            spectre_mana_cost: 80,
            explosion_damage: 80,
            explosion_terror_potency: BasisPoints(2_000),
            explosion_radius: 2,
            sanctify_channel_ticks: 60,
        }
    }
}

/// Adventurer retreat triggering thresholds.
///
/// Specified in `SPEC-REQ-INTEL-001`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetreatConfig {
    /// Critical individual health threshold triggering retreat (25.00% = 2_500 BPS).
    pub hp_threshold: BasisPoints,
    /// Squad casualty ratio threshold triggering retreat (50.00% = 5_000 BPS).
    pub casualty_threshold: BasisPoints,
}

impl Default for RetreatConfig {
    fn default() -> Self {
        Self {
            hp_threshold: BasisPoints(2_500),
            casualty_threshold: BasisPoints(5_000),
        }
    }
}

/// Chronomancy snapshot intervals, ring buffer, and rewind costs.
///
/// Specified in `SPEC-REQ-CHRONO-002`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChronoConfig {
    /// Snapshot capture interval in simulation ticks (100 ticks = 5.0 seconds).
    pub snapshot_interval_ticks: u64,
    /// Number of rolling snapshots retained in memory ring buffer (12 snapshots).
    pub ring_buffer_capacity: usize,
    /// Base temporal mana cost for any rewind operation (20 Mana).
    pub rewind_base_cost_mana: u32,
    /// Mana cost factor per 100 ticks rewound in basis points (500 BPS).
    pub rewind_tick_cost_bps: BasisPoints,
}

impl Default for ChronoConfig {
    fn default() -> Self {
        Self {
            snapshot_interval_ticks: 100,
            ring_buffer_capacity: 12,
            rewind_base_cost_mana: 20,
            rewind_tick_cost_bps: BasisPoints(500),
        }
    }
}

/// Viewport and rendering safe zone dimensions.
///
/// Specified in `SPEC-DOMAIN-SAVE-VIEW` (Spec 06).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewportConfig {
    /// Safe Zone logical pixel width (320 px).
    pub safe_zone_width: u32,
    /// Safe Zone logical pixel height (240 px).
    pub safe_zone_height: u32,
}

impl Default for ViewportConfig {
    fn default() -> Self {
        Self {
            safe_zone_width: 320,
            safe_zone_height: 240,
        }
    }
}

/// Discrete 2.5D topology and navigation parameters.
///
/// Specified in `SPEC-REQ-TOPO-001`, `SPEC-REQ-TOPO-002`, and `SPEC-REQ-TOPO-005`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyConfig {
    /// Nominal duration to traverse stairs in ticks (40 ticks = 2.0 s).
    pub stairs_traversal_ticks: u64,
    /// Duration to traverse a ladder in ticks (80 ticks = 4.0 s).
    pub ladder_traversal_ticks: u64,
    /// Duration to fall through a pitfall in ticks (5 ticks = 0.25 s).
    pub pitfall_traversal_ticks: u64,
    /// Duration to traverse a one-way magical portal in ticks (10 ticks = 0.5 s).
    pub portal_traversal_ticks: u64,
    /// Vulnerability penalty when climbing a ladder in basis points (+25% = 2_500 BPS).
    pub ladder_vulnerability_bps: BasisPoints,
    /// Nominal cost in ticks to move orthogonally between adjacent tiles (10 ticks).
    pub orthogonal_step_cost: u32,
    /// Base fall damage per floor dropped for pitfall transitions.
    pub base_fall_damage: u32,
    /// Distance weight in frontier utility calculation (150 BPS/tile).
    pub frontier_distance_weight_bps: BasisPoints,
    /// Terror miasma weight in frontier utility calculation (200 BPS/point).
    pub frontier_terror_weight_bps: BasisPoints,
    /// Room area heuristic weight in frontier utility calculation (50 BPS/tile).
    pub frontier_room_weight_bps: BasisPoints,
}

impl Default for TopologyConfig {
    fn default() -> Self {
        Self {
            stairs_traversal_ticks: 40,
            ladder_traversal_ticks: 80,
            pitfall_traversal_ticks: 5,
            portal_traversal_ticks: 10,
            ladder_vulnerability_bps: BasisPoints(2_500),
            orthogonal_step_cost: 10,
            base_fall_damage: 20,
            frontier_distance_weight_bps: BasisPoints(150),
            frontier_terror_weight_bps: BasisPoints(200),
            frontier_room_weight_bps: BasisPoints(50),
        }
    }
}

/// Master gameplay and engine configuration structure.
///
/// Aggregates all domain subsystems configuration without magic numbers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameConfig {
    /// Fixed-tick simulation clock configuration.
    pub tick: TickConfig,
    /// Standard 100% basis points reference value.
    pub standard_bps: BasisPoints,
    /// Corpse stacking, integrity, and degradation configuration.
    pub corpse: CorpseConfig,
    /// Terror psychological thresholds.
    pub terror: TerrorConfig,
    /// Necromancy spells, costs, and minion parameters.
    pub necro: NecroConfig,
    /// Adventurer tactical retreat parameters.
    pub retreat: RetreatConfig,
    /// Chronomancy snapshots and temporal mana costs.
    pub chrono: ChronoConfig,
    /// Viewport Safe Zone dimensions.
    pub viewport: ViewportConfig,
    /// Discrete 2.5D topology and navigation configuration.
    pub topology: TopologyConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            tick: TickConfig::default(),
            standard_bps: BasisPoints::STANDARD,
            corpse: CorpseConfig::default(),
            terror: TerrorConfig::default(),
            necro: NecroConfig::default(),
            retreat: RetreatConfig::default(),
            chrono: ChronoConfig::default(),
            viewport: ViewportConfig::default(),
            topology: TopologyConfig::default(),
        }
    }
}
