//! Discrete chronomancy command types for the event-sourced action journal.
//!
//! Conforms to `SPEC-REQ-CHRONO-001`.

use serde::{Deserialize, Serialize};

use crate::id::LogicId;
use crate::time::Tick;

/// Discrete trap mechanical types available for placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TrapType {
    /// Rotating blade mechanism causing laceration damage.
    Blade,
    /// Retractable floor spikes triggered by pressure.
    Spikes,
    /// Concealed floor pit dropping victims down.
    Pitfall,
    /// Pressurized flame vent inflicting fire damage.
    Fire,
    /// Noxious vapor emitter causing persistent damage over time.
    PoisonGas,
}

/// Logical state of portcullises and reinforced doors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GateState {
    /// Portcullis is raised; passage is unobstructed.
    Open,
    /// Portcullis is lowered; passage is blocked.
    Closed,
    /// Portcullis is barred and locked mechanically or magically.
    Locked,
}

pub use crate::necro::UndeadKind;

/// Core domain gameplay mutations initiated by the Dungeon Master or scripts.
///
/// Specified in `SPEC-REQ-CHRONO-001`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreCommand {
    /// Arms or constructs a trap mechanism at specified grid coordinates.
    ArmTrap {
        /// Dungeon floor index.
        floor: u8,
        /// Horizontal discrete grid coordinate.
        x: i32,
        /// Vertical discrete grid coordinate.
        y: i32,
        /// Type of trap mechanism to arm.
        trap_type: TrapType,
    },
    /// Manually triggers an active trap mechanism by its identifier.
    TriggerTrapManual {
        /// Identifier of the target trap.
        trap_id: LogicId,
    },
    /// Toggles or transitions the state of a portcullis or reinforced gate.
    TogglePortcullis {
        /// Identifier of the target gate.
        gate_id: LogicId,
        /// Desired target operational state.
        target_state: GateState,
    },
    /// Raises a fallen adventurer corpse as a servant minion.
    RaiseCorpse {
        /// Identifier of the corpse entity to animate.
        corpse_id: LogicId,
        /// Minion archetype to summon.
        target_undead: UndeadKind,
    },
    /// Orders a temporal rewind to a prior simulation tick.
    ChannelTemporalRewind {
        /// Destination tick for the rewind operation.
        target_tick: Tick,
    },
}

/// Immutable, timestamped gameplay command recorded in the [`ActionJournal`].
///
/// Specified in `SPEC-REQ-CHRONO-001`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedCommand {
    /// Simulation tick at which the command occurred.
    pub tick: Tick,
    /// Globally unique logical command identifier.
    pub command_id: LogicId,
    /// Inner payload specifying the domain action.
    pub payload: CoreCommand,
}
