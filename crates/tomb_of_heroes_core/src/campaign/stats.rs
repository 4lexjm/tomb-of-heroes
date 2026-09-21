//! Aggregated campaign and end-game summary statistics.
//!
//! Conforms to `SPEC-REQ-WAVE-005` (`docs/specs/11_vagues_campagne_victoire.md`).

use alloc::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::necro::HeroClass;

/// Aggregated end-game campaign statistics.
///
/// Conforms to `SPEC-REQ-WAVE-005`:
/// - Total waves successfully defended.
/// - Total hero invaders slain, overall and broken down by archetype class.
/// - Total hero invaders who perished from blind panic.
/// - Total hero invaders escaped (and escaped unhurt).
/// - Soul essence / mana harvested.
/// - Dead bodies raised or converted into minions.
/// - Chronomantic rewinds executed and paradox accumulated.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CampaignStats {
    /// Total waves successfully defended.
    pub waves_cleared: u32,
    /// Total hero invaders slain.
    pub heroes_killed_total: u32,
    /// Total hero invaders slain broken down by archetype class.
    pub heroes_killed_by_class: BTreeMap<HeroClass, u32>,
    /// Total hero invaders died while suffering blind panic.
    pub heroes_died_of_panic: u32,
    /// Total hero invaders who successfully escaped to the surface.
    pub heroes_escaped_total: u32,
    /// Total hero invaders who escaped unharmed (at full max HP).
    pub heroes_escaped_unhurt: u32,
    /// Total soul essence / mana harvested.
    pub mana_harvested_total: u32,
    /// Total corpses raised or converted to undead minions.
    pub corpses_converted_total: u32,
    /// Total chronomantic temporal rewinds executed.
    pub rewinds_performed: u32,
    /// Total cumulative temporal paradox anxiety generated.
    pub paradox_accumulated: u32,
}

impl CampaignStats {
    /// Creates a fresh zeroed campaign statistics tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a hero kill with class breakdown and panic state.
    pub fn record_kill(&mut self, hero_class: HeroClass, was_panicked: bool) {
        self.heroes_killed_total = self.heroes_killed_total.saturating_add(1);
        let count = self.heroes_killed_by_class.entry(hero_class).or_insert(0);
        *count = (*count).saturating_add(1);
        if was_panicked {
            self.heroes_died_of_panic = self.heroes_died_of_panic.saturating_add(1);
        }
    }

    /// Records an adventurer escape.
    pub fn record_escape(&mut self, unhurt: bool) {
        self.heroes_escaped_total = self.heroes_escaped_total.saturating_add(1);
        if unhurt {
            self.heroes_escaped_unhurt = self.heroes_escaped_unhurt.saturating_add(1);
        }
    }

    /// Records harvested mana from traps or death sequences.
    pub fn record_mana_harvested(&mut self, amount: u32) {
        self.mana_harvested_total = self.mana_harvested_total.saturating_add(amount);
    }

    /// Records a corpse raised or converted.
    pub fn record_corpse_conversion(&mut self) {
        self.corpses_converted_total = self.corpses_converted_total.saturating_add(1);
    }

    /// Records a temporal rewind and resulting paradox points.
    pub fn record_rewind(&mut self, paradox_delta: u32) {
        self.rewinds_performed = self.rewinds_performed.saturating_add(1);
        self.paradox_accumulated = self.paradox_accumulated.saturating_add(paradox_delta);
    }
}
