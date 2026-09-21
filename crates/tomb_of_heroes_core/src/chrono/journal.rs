//! Event-sourced action journal and error types.
//!
//! Conforms to `SPEC-REQ-CHRONO-001`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::chrono::command::TimedCommand;
use crate::id::LogicId;
use crate::time::Tick;

/// Errors arising during command journal manipulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalError {
    /// Attempted to record a command that breaks chronological or identifier ordering.
    UnorderedCommand {
        /// Tick of the most recently recorded command.
        last_tick: Tick,
        /// Identifier of the most recently recorded command.
        last_id: LogicId,
        /// Tick of the offending command.
        attempted_tick: Tick,
        /// Identifier of the offending command.
        attempted_id: LogicId,
    },
    /// Attempted to record a command with an identifier already present in the journal.
    DuplicateCommandId {
        /// Conflicting identifier.
        id: LogicId,
    },
}

impl std::fmt::Display for JournalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnorderedCommand {
                last_tick,
                last_id,
                attempted_tick,
                attempted_id,
            } => write!(
                f,
                "Command out of order: last was ({last_tick}, {last_id}), attempted ({attempted_tick}, {attempted_id})"
            ),
            Self::DuplicateCommandId { id } => {
                write!(f, "Duplicate command ID in journal: {id}")
            }
        }
    }
}

impl std::error::Error for JournalError {}

/// Append-only event-sourced journal recording deterministic player and script actions.
///
/// Specified in `SPEC-REQ-CHRONO-001`.
/// Enforces strict monotonically increasing `(tick, command_id)` ordering and globally
/// unique command identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ActionJournal {
    commands: Vec<TimedCommand>,
    recorded_ids: BTreeSet<LogicId>,
}

impl ActionJournal {
    /// Creates a new, empty action journal.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            commands: Vec::new(),
            recorded_ids: BTreeSet::new(),
        }
    }

    /// Constructs an action journal from a pre-existing list of commands,
    /// validating chronological ordering and identifier uniqueness.
    pub fn from_commands(commands: Vec<TimedCommand>) -> Result<Self, JournalError> {
        let mut journal = Self::new();
        for cmd in commands {
            journal.record_command(cmd)?;
        }
        Ok(journal)
    }

    /// Returns `true` if no commands have been recorded.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Returns the total count of recorded commands.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Returns the most recently recorded command, if any.
    #[inline]
    #[must_use]
    pub fn last_command(&self) -> Option<&TimedCommand> {
        self.commands.last()
    }

    /// Returns `true` if a command with the given identifier is already recorded in the journal.
    #[inline]
    #[must_use]
    pub fn contains_command_id(&self, id: LogicId) -> bool {
        self.recorded_ids.contains(&id)
    }

    /// Appends a new command to the journal, verifying monotonicity and identifier uniqueness.
    ///
    /// # Errors
    /// - [`JournalError::DuplicateCommandId`]: if `command.command_id` was already recorded.
    /// - [`JournalError::UnorderedCommand`]: if `command.tick` is prior to the last recorded tick,
    ///   or if simultaneous with the last tick and `command.command_id <= last.command_id`.
    pub fn record_command(&mut self, command: TimedCommand) -> Result<(), JournalError> {
        if self.recorded_ids.contains(&command.command_id) {
            return Err(JournalError::DuplicateCommandId {
                id: command.command_id,
            });
        }

        if let Some(last) = self.commands.last() {
            let is_strictly_after = (command.tick > last.tick)
                || (command.tick == last.tick && command.command_id > last.command_id);

            if !is_strictly_after {
                return Err(JournalError::UnorderedCommand {
                    last_tick: last.tick,
                    last_id: last.command_id,
                    attempted_tick: command.tick,
                    attempted_id: command.command_id,
                });
            }
        }

        self.recorded_ids.insert(command.command_id);
        self.commands.push(command);
        Ok(())
    }

    /// Returns an immutable slice of all recorded commands in strict chronological order.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[TimedCommand] {
        &self.commands
    }

    /// Returns a slice of commands occurring within the closed interval `[from, to]`.
    ///
    /// If `from > to` or if no commands fall in the range, returns an empty slice `&[]`.
    #[must_use]
    pub fn commands_in_interval(&self, from: Tick, to: Tick) -> &[TimedCommand] {
        if from > to || self.commands.is_empty() {
            return &[];
        }

        let start_idx = self.commands.partition_point(|cmd| cmd.tick < from);
        let end_idx = self.commands.partition_point(|cmd| cmd.tick <= to);

        if start_idx >= end_idx {
            &[]
        } else {
            &self.commands[start_idx..end_idx]
        }
    }

    /// Discards all commands recorded with ticks strictly greater than `tick`.
    ///
    /// Prunes corresponding command identifiers so they may be re-recorded in divergent timelines.
    pub fn truncate_after(&mut self, tick: Tick) {
        let split_idx = self.commands.partition_point(|cmd| cmd.tick <= tick);
        for cmd in &self.commands[split_idx..] {
            self.recorded_ids.remove(&cmd.command_id);
        }
        self.commands.truncate(split_idx);
    }

    /// Clears all recorded commands and tracked identifiers.
    pub fn clear(&mut self) {
        self.commands.clear();
        self.recorded_ids.clear();
    }
}
