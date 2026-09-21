//! Adventurer squad retreat decisions, tactical escape, and DM countermeasures.
//!
//! Conforms to `SPEC-REQ-INTEL-001` and `SPEC-REQ-INTEL-002`.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::chrono::command::GateState;
use crate::config::RetreatConfig;
use crate::id::LogicId;
use crate::math::{div_bps, BasisPoints};
use crate::necro::terror::TerrorPoints;
use crate::topology::coordinates::WorldCoord;

/// Global behavioral state of an adventurer exploration squad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SquadState {
    /// Incursion phase: standard progression and room exploration.
    Incursion,
    /// Combat phase: engaging hostile dungeon defenders.
    Combat,
    /// Tactical retreat phase: disengaging and fleeing towards nearest extraction exit.
    Retreat,
}

/// An individual adventurer participating in an expedition squad.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadMember {
    /// Unique persistent entity identifier.
    pub id: LogicId,
    /// Current remaining hit points.
    pub current_hp: u32,
    /// Nominal maximum hit points.
    pub max_hp: u32,
    /// Accumulated psychological terror points gauge.
    pub terror_points: TerrorPoints,
    /// Flag indicating if this member is the squad leader.
    pub is_leader: bool,
}

impl SquadMember {
    /// Creates a new squad member.
    #[must_use]
    pub const fn new(
        id: LogicId,
        current_hp: u32,
        max_hp: u32,
        terror_points: TerrorPoints,
        is_leader: bool,
    ) -> Self {
        Self {
            id,
            current_hp,
            max_hp,
            terror_points,
            is_leader,
        }
    }

    /// Returns `true` if this squad member is still alive.
    #[inline]
    #[must_use]
    pub const fn is_alive(&self) -> bool {
        self.current_hp > 0
    }
}

/// Coordinated expedition squad of adventurers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Squad {
    /// Unique squad identifier.
    pub squad_id: LogicId,
    /// Members belonging to this squad.
    pub members: Vec<SquadMember>,
    /// Current behavioral state.
    pub state: SquadState,
}

impl Squad {
    /// Creates a new squad in initial `Incursion` state.
    #[must_use]
    pub fn new(squad_id: LogicId, members: Vec<SquadMember>) -> Self {
        Self {
            squad_id,
            members,
            state: SquadState::Incursion,
        }
    }

    /// Evaluates retreat conditions and updates `state` to `Retreat` if triggered.
    pub fn update_state(&mut self, config: &RetreatConfig) {
        if should_squad_retreat(self, config) {
            self.state = SquadState::Retreat;
        }
    }
}

/// Evaluates whether a squad must transition to tactical retreat.
///
/// Conforms to `SPEC-REQ-INTEL-001`.
/// Triggers retreat if:
/// 1. Any living member has health ratio < `config.hp_threshold` (25.00%).
/// 2. Squad casualty ratio >= `config.casualty_threshold` (50.00%) or squad leader is dead.
/// 3. More than 50% of living squad members are in Blind Panic (terror >= `config.blind_panic_threshold`).
#[must_use]
pub fn should_squad_retreat(squad: &Squad, config: &RetreatConfig) -> bool {
    if squad.members.is_empty() {
        return false;
    }

    let mut total_members: u32 = 0;
    let mut dead_members: u32 = 0;
    let mut living_members: u32 = 0;
    let mut blind_panic_members: u32 = 0;
    let mut leader_eliminated = false;
    let mut critical_individual_hp = false;

    for member in &squad.members {
        total_members = total_members.saturating_add(1);
        if member.current_hp == 0 {
            dead_members = dead_members.saturating_add(1);
            if member.is_leader {
                leader_eliminated = true;
            }
        } else {
            living_members = living_members.saturating_add(1);
            if member.terror_points.0 >= config.blind_panic_threshold {
                blind_panic_members = blind_panic_members.saturating_add(1);
            }
            if member.max_hp > 0 {
                if let Ok(hp_bps) = div_bps(member.current_hp, member.max_hp) {
                    if hp_bps < config.hp_threshold {
                        critical_individual_hp = true;
                    }
                }
            }
        }
    }

    // Condition 1: Critical individual HP (< 2_500 BPS)
    if critical_individual_hp {
        return true;
    }

    // Condition 2: Squad leader eliminated
    if leader_eliminated {
        return true;
    }

    // Condition 2 (bis): Squad casualties >= 5_000 BPS (50.00%)
    if total_members > 0 {
        if let Ok(casualty_ratio) = div_bps(dead_members, total_members) {
            if casualty_ratio >= config.casualty_threshold {
                return true;
            }
        }
    }

    // Condition 3: Majority of living squad in Blind Panic (> 50%)
    if living_members > 0 && blind_panic_members.saturating_mul(2) > living_members {
        return true;
    }

    false
}

/// Computes the retreat speed bonus in basis points.
///
/// Conforms to `SPEC-REQ-INTEL-001` (+1_500 BPS during retreat).
#[must_use]
pub fn squad_flee_speed_bonus(state: SquadState, config: &RetreatConfig) -> BasisPoints {
    if state == SquadState::Retreat {
        config.flee_speed_bonus_bps
    } else {
        BasisPoints::ZERO
    }
}

/// Portcullis countermeasure lowered to block escape routes.
///
/// Specified in `SPEC-REQ-INTEL-002`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Portcullis {
    /// Logical gate identifier.
    pub gate_id: LogicId,
    /// Grid coordinate in the dungeon.
    pub coord: WorldCoord,
    /// Current mechanical gate state.
    pub state: GateState,
    /// Cumulative ticks spent by fuyards attempting to force this gate open.
    pub forcing_progress_ticks: u32,
}

impl Portcullis {
    /// Creates a new portcullis at the specified coordinate.
    #[must_use]
    pub const fn new(gate_id: LogicId, coord: WorldCoord, state: GateState) -> Self {
        Self {
            gate_id,
            coord,
            state,
            forcing_progress_ticks: 0,
        }
    }

    /// Returns `true` if this portcullis physically blocks movement.
    #[inline]
    #[must_use]
    pub const fn is_blocking(&self) -> bool {
        matches!(self.state, GateState::Closed | GateState::Locked)
    }

    /// Advances forcing progress and transitions to `Open` when threshold is met.
    ///
    /// Returns `true` if the gate is now open.
    pub fn tick_forcing(&mut self, elapsed_ticks: u32, config: &RetreatConfig) -> bool {
        if !self.is_blocking() {
            return true;
        }
        self.forcing_progress_ticks = self.forcing_progress_ticks.saturating_add(elapsed_ticks);
        if self.forcing_progress_ticks >= config.portcullis_force_ticks {
            self.state = GateState::Open;
            true
        } else {
            false
        }
    }
}

/// Dimensional anchor anti-teleportation countermeasure.
///
/// Specified in `SPEC-REQ-INTEL-002`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimensionalAnchor {
    /// Unique anchor identifier.
    pub anchor_id: LogicId,
    /// Center anchor location.
    pub coord: WorldCoord,
    /// Radius of magical inhibition.
    pub radius: u32,
    /// Inhibition rate in basis points.
    pub inhibition_bps: BasisPoints,
    /// Whether the device is active.
    pub is_active: bool,
}

impl DimensionalAnchor {
    /// Creates an active dimensional anchor according to configuration.
    #[must_use]
    pub const fn new(anchor_id: LogicId, coord: WorldCoord, config: &RetreatConfig) -> Self {
        Self {
            anchor_id,
            coord,
            radius: config.dimensional_anchor_radius,
            inhibition_bps: config.dimensional_anchor_inhibition_bps,
            is_active: true,
        }
    }

    /// Checks if a target coordinate falls within the teleportation inhibition zone.
    #[must_use]
    pub fn inhibits_teleport(&self, target_coord: WorldCoord) -> bool {
        if !self.is_active || self.coord.floor != target_coord.floor {
            return false;
        }
        let dist = self.coord.coord.chebyshev_distance(target_coord.coord);
        dist <= self.radius && self.inhibition_bps.0 >= 10_000
    }
}
