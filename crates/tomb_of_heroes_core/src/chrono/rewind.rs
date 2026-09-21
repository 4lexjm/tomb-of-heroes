//! Deterministic temporal rewind algorithm and mana cost computation.
//!
//! Conforms to `SPEC-REQ-CHRONO-002` and `SPEC-REQ-CHRONO-003`.

use std::collections::{BTreeMap, BTreeSet};

use crate::chrono::command::CoreCommand;
use crate::chrono::error::ChronoError;
use crate::chrono::journal::ActionJournal;
use crate::chrono::memory::ChronoMemory;
use crate::chrono::snapshot::SnapshotRingBuffer;
use crate::config::ChronoConfig;
use crate::time::Tick;
use crate::topology::{FloorId, GridCoord, WorldCoord};
use crate::world::LogicWorld;

/// Computes the temporal mana cost for a rewind operation from `current_tick` to `target_tick`.
///
/// Formula: $\text{BaseCost} + \lfloor (\Delta T \times \text{TickCostBps}) / 10\,000 \rfloor$.
/// If `target_tick == current_tick`, returns 0.
///
/// # Errors
/// Returns [`ChronoError::FutureTargetTick`] if `target_tick > current_tick`.
pub fn compute_rewind_cost(
    current_tick: Tick,
    target_tick: Tick,
    config: &ChronoConfig,
) -> Result<u32, ChronoError> {
    if target_tick > current_tick {
        return Err(ChronoError::FutureTargetTick {
            target: target_tick,
            current: current_tick,
        });
    }

    if target_tick == current_tick {
        return Ok(0);
    }

    let delta_ticks = current_tick.0 - target_tick.0;
    let bps = config.rewind_tick_cost_bps.0 as u64;
    let tick_cost = (delta_ticks.saturating_mul(bps)) / 10_000;
    let total = (config.rewind_base_cost_mana as u64).saturating_add(tick_cost);

    let cost_u32 = if total > u32::MAX as u64 {
        u32::MAX
    } else {
        total as u32
    };

    Ok(cost_u32)
}

/// Executes a deterministic temporal rewind operation to `target_tick`.
///
/// Conforms to `SPEC-REQ-CHRONO-002` and `SPEC-REQ-CHRONO-003`.
///
/// # Execution Steps:
/// 1. Validation: $T_{\text{oldest\_snapshot}} \le T_{\text{target}} \le T_{\text{current}}$.
///    If `target == current` -> returns `Ok(())` (no-op).
/// 2. If $T_{\text{target}} < T_{\text{oldest}}$ -> returns `Err(ChronoError::SnapshotExpired)`.
/// 3. Computes mana cost. If `*available_mana < cost` -> returns `Err(ChronoError::InsufficientMana)`.
/// 4. Collects hazards from the discarded future interval $]T_{\text{target}}, T_{\text{current}}]$.
/// 5. Finds snapshot $S_k$ where $k = \max(\{ t \in \text{Snapshots} \mid t \le T_{\text{target}} \})$.
/// 6. Restores the world state from $S_k$.
/// 7. Deterministically replays commands in $]k, T_{\text{target}}]$.
/// 8. Injects retained `ChronoMemory` into chrono-aware heroes, and paradox terror into ordinary allies.
/// 9. Truncates commands in `ActionJournal` strictly after $T_{\text{target}}$.
/// 10. Deducts mana cost from `available_mana`.
///
/// # Errors
/// - [`ChronoError::FutureTargetTick`]: if `target_tick > world.current_tick()`.
/// - [`ChronoError::SnapshotExpired`]: if `target_tick` is prior to the oldest snapshot.
/// - [`ChronoError::InsufficientMana`]: if `*available_mana` is insufficient.
pub fn execute_rewind(
    world: &mut LogicWorld,
    journal: &mut ActionJournal,
    snapshots: &SnapshotRingBuffer,
    target_tick: Tick,
    available_mana: &mut u32,
) -> Result<(), ChronoError> {
    let current_tick = world.current_tick();

    // 1. Validation: future target tick
    if target_tick > current_tick {
        return Err(ChronoError::FutureTargetTick {
            target: target_tick,
            current: current_tick,
        });
    }

    // 1. Validation: target == current tick is a no-op
    if target_tick == current_tick {
        return Ok(());
    }

    // 2. Oldest snapshot check
    let oldest_tick = snapshots
        .oldest_tick()
        .ok_or(ChronoError::SnapshotExpired)?;
    if target_tick < oldest_tick {
        return Err(ChronoError::SnapshotExpired);
    }

    // 3. Mana calculation & verification
    let cost = compute_rewind_cost(current_tick, target_tick, &world.config().chrono)?;
    if *available_mana < cost {
        return Err(ChronoError::InsufficientMana);
    }

    // 4. Collect future hazards from discarded future interval ]T_target, T_current]
    let mut future_hazards = BTreeSet::new();
    let future_commands =
        journal.commands_in_interval(Tick(target_tick.0.saturating_add(1)), current_tick);
    for cmd in future_commands {
        if let CoreCommand::ArmTrap { floor, x, y, .. } = &cmd.payload {
            future_hazards.insert(WorldCoord::new(FloorId(*floor), GridCoord::new(*x, *y)));
        }
    }
    // Also include any hazards currently recorded on the world
    for hazard in world.hazards() {
        future_hazards.insert(*hazard);
    }

    // Collect memories of chrono-aware heroes currently in the world before rollback
    let mut aware_memories: BTreeMap<crate::id::LogicId, ChronoMemory> = BTreeMap::new();
    let has_any_aware_hero = world.heroes().values().any(|h| h.has_chrono_awareness);
    for hero in world.heroes().values() {
        if hero.has_chrono_awareness {
            let mut mem = hero.chrono_memory.clone();
            mem.anticipated_hazards
                .extend(future_hazards.iter().copied());
            aware_memories.insert(hero.hero_id, mem);
        }
    }

    // 5. Find snapshot S_k with k = max({ t in Snapshots | t <= target })
    let snapshot = snapshots
        .find_closest_preceding(target_tick)
        .ok_or(ChronoError::SnapshotExpired)?;
    let k = snapshot.tick;

    // 6. Restore world from S_k
    snapshot.restore(world);

    // 7. Replay commands in ]k, T_target]
    let mut t = k.0;
    while t < target_tick.0 {
        world.step();
        t = world.current_tick().0;
        let cmds = journal.commands_in_interval(Tick(t), Tick(t));
        for cmd in cmds {
            world.apply_command(&cmd.payload);
        }
    }

    // 8. Cognitive adaptation: restore ChronoMemory on aware heroes & paradox terror on non-aware
    let paradox_terror = world.config().chrono.paradox_terror_bps;
    for hero in world.heroes_mut().values_mut() {
        if hero.has_chrono_awareness {
            if let Some(preserved) = aware_memories.remove(&hero.hero_id) {
                hero.chrono_memory = preserved;
            } else {
                hero.chrono_memory
                    .anticipated_hazards
                    .extend(future_hazards.iter().copied());
            }
        } else if has_any_aware_hero {
            hero.apply_paradox_anxiety(paradox_terror);
        }
    }

    // 9. Truncate journal beyond T_target
    journal.truncate_after(target_tick);

    // 10. Deduct mana
    *available_mana = available_mana.saturating_sub(cost);

    Ok(())
}
