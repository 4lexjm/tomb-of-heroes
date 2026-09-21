//! Simulation state snapshots and ring buffer storage.
//!
//! Conforms to `SPEC-REQ-CHRONO-002`.

use serde::{Deserialize, Serialize};

use crate::config::ChronoConfig;
use crate::hash::StateHash;
use crate::time::Tick;
use crate::world::LogicWorld;

/// Full deterministic keyframe snapshot of the simulation world state.
///
/// Specified in `SPEC-REQ-CHRONO-002`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Discrete tick at which the snapshot was captured.
    pub tick: Tick,
    /// Cryptographic state hash computed at capture.
    pub state_hash: StateHash,
    /// Fully replicated deterministic logical world state.
    pub world: LogicWorld,
}

impl StateSnapshot {
    /// Captures a complete deterministic snapshot of the given world.
    #[must_use]
    pub fn capture(world: &LogicWorld) -> Self {
        Self {
            tick: world.current_tick(),
            state_hash: world.state_hash(),
            world: world.clone(),
        }
    }

    /// Restores the target world to this snapshot's state.
    pub fn restore(&self, world: &mut LogicWorld) {
        *world = self.world.clone();
    }
}

/// Fixed-capacity rolling ring buffer storing periodic keyframe snapshots.
///
/// Specified in `SPEC-REQ-CHRONO-002`.
/// Retains up to `capacity` snapshots (nominal 12) captured every
/// `snapshot_interval_ticks` (nominal 100 ticks).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotRingBuffer {
    snapshots: Vec<StateSnapshot>,
    capacity: usize,
}

impl SnapshotRingBuffer {
    /// Creates a new snapshot ring buffer with the specified maximum capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            snapshots: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Creates a new snapshot ring buffer using the engine chronomancy configuration.
    #[must_use]
    pub fn with_config(config: &ChronoConfig) -> Self {
        Self::new(config.ring_buffer_capacity)
    }

    /// Returns the maximum capacity of the ring buffer.
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the number of snapshots currently stored.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Returns `true` if the ring buffer contains no snapshots.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Pushes a snapshot into the ring buffer.
    ///
    /// If the buffer has reached `capacity`, the oldest snapshot is evicted.
    pub fn push(&mut self, snapshot: StateSnapshot) {
        if self.capacity == 0 {
            return;
        }
        if self.snapshots.len() >= self.capacity {
            self.snapshots.remove(0);
        }
        self.snapshots.push(snapshot);
    }

    /// Captures a snapshot of the world if the current tick matches the configured cadence.
    ///
    /// Nominal cadence: captures every `config.snapshot_interval_ticks` ticks (e.g. 100, 200, 300).
    /// Does not capture at tick 0.
    pub fn capture_if_due(&mut self, world: &LogicWorld, config: &ChronoConfig) -> bool {
        let tick = world.current_tick().0;
        if tick > 0
            && config.snapshot_interval_ticks > 0
            && tick.is_multiple_of(config.snapshot_interval_ticks)
        {
            self.push(StateSnapshot::capture(world));
            true
        } else {
            false
        }
    }

    /// Returns the simulation tick of the oldest snapshot in the buffer, if any.
    #[must_use]
    pub fn oldest_tick(&self) -> Option<Tick> {
        self.snapshots.first().map(|s| s.tick)
    }

    /// Returns the simulation tick of the newest snapshot in the buffer, if any.
    #[must_use]
    pub fn newest_tick(&self) -> Option<Tick> {
        self.snapshots.last().map(|s| s.tick)
    }

    /// Returns a reference to the oldest snapshot, if any.
    #[must_use]
    pub fn oldest_snapshot(&self) -> Option<&StateSnapshot> {
        self.snapshots.first()
    }

    /// Returns a reference to the newest snapshot, if any.
    #[must_use]
    pub fn newest_snapshot(&self) -> Option<&StateSnapshot> {
        self.snapshots.last()
    }

    /// Finds the snapshot $S_k$ such that $k = \max(\{ t \in \text{Snapshots} \mid t \le target \})$.
    #[must_use]
    pub fn find_closest_preceding(&self, target: Tick) -> Option<&StateSnapshot> {
        self.snapshots.iter().rev().find(|s| s.tick <= target)
    }

    /// Returns an immutable slice of all stored snapshots in chronological order.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[StateSnapshot] {
        &self.snapshots
    }

    /// Returns an immutable slice of all stored snapshots.
    #[inline]
    #[must_use]
    pub fn snapshots(&self) -> &[StateSnapshot] {
        &self.snapshots
    }

    /// Clears all snapshots from the ring buffer.
    pub fn clear(&mut self) {
        self.snapshots.clear();
    }
}
