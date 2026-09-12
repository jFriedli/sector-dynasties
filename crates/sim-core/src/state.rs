//! The authoritative simulation state and its single step function.
//!
//! `SimState` is the whole world: everything needed to save, load, or
//! resume the simulation lives here, including the per-domain RNG streams
//! (so resuming from a save produces the same future as an uninterrupted
//! run). The UI and CLI never mutate this directly; they call `step`.

use serde::{Deserialize, Serialize};

use crate::dynasty::{Character, Dynasty};
use crate::economy;
use crate::rng::SimRng;
use crate::time::SimClock;
use crate::world::Sector;
use crate::worldgen::generate_sector;

pub const SAVE_SCHEMA_VERSION: u32 = 1;
const STARTING_SYSTEM_COUNT: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimState {
    pub schema_version: u32,
    pub seed: u64,
    pub clock: SimClock,
    pub sector: Sector,
    pub dynasty: Dynasty,
    economy_rng: SimRng,
}

impl SimState {
    /// Build a brand new game from a seed: generate the sector and found
    /// the player's starting dynasty in its first city.
    pub fn new(seed: u64) -> Self {
        let sector = generate_sector(seed, STARTING_SYSTEM_COUNT);
        let mut character_rng = SimRng::from_seed(seed, "dynasty");

        let home_city = sector
            .systems
            .first()
            .and_then(|s| s.planets.first())
            .and_then(|p| p.countries.first())
            .and_then(|c| c.cities.first())
            .map(|c| c.id);

        let founder = Character {
            id: 1,
            name: "Founder".to_string(),
            age_years: 28 + character_rng.next_below(20),
            alive: true,
            wealth: character_rng.range_f64(1_000.0, 10_000.0),
            home_city,
        };

        let dynasty = Dynasty {
            name: "House Meridian".to_string(),
            head_character_id: founder.id,
            members: vec![founder],
        };

        SimState {
            schema_version: SAVE_SCHEMA_VERSION,
            seed,
            clock: SimClock::new(),
            sector,
            dynasty,
            economy_rng: SimRng::from_seed(seed, "economy"),
        }
    }

    /// Advance the simulation by one day. Lower-frequency systems check the
    /// clock rather than running every call.
    pub fn step_one_day(&mut self) {
        self.clock.advance_one_day();

        if self.clock.is_week_boundary() {
            economy::settle_week(&mut self.sector, &mut self.economy_rng);
        }
        // Monthly population updates and yearly demographic change hook in
        // here as their own systems; see docs/ROADMAP.md milestone
        // "Population and culture".
    }

    pub fn step_days(&mut self, days: u32) {
        for _ in 0..days {
            self.step_one_day();
        }
    }

    pub fn summary(&self) -> StateSummary {
        let city_count: usize = self
            .sector
            .systems
            .iter()
            .flat_map(|s| &s.planets)
            .flat_map(|p| &p.countries)
            .map(|c| c.cities.len())
            .sum();
        let total_population: u64 = self
            .sector
            .systems
            .iter()
            .flat_map(|s| &s.planets)
            .flat_map(|p| &p.countries)
            .flat_map(|c| &c.cities)
            .map(|city| city.population.size)
            .sum();

        StateSummary {
            tick: self.clock.tick,
            year: self.clock.year(),
            system_count: self.sector.systems.len(),
            city_count,
            total_population,
            dynasty_name: self.dynasty.name.clone(),
            dynasty_wealth: self.dynasty.total_wealth(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("SimState always serializes")
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// A compact, UI/CLI-friendly view of the state, so callers don't need to
/// walk the full hierarchy just to show a status line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateSummary {
    pub tick: u64,
    pub year: u64,
    pub system_count: usize,
    pub city_count: usize,
    pub total_population: u64,
    pub dynasty_name: String,
    pub dynasty_wealth: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invariants::check_invariants;

    #[test]
    fn same_seed_and_step_count_produce_identical_state() {
        let mut a = SimState::new(2026);
        let mut b = SimState::new(2026);
        a.step_days(400);
        b.step_days(400);
        assert_eq!(a.to_json(), b.to_json());
    }

    #[test]
    fn stepping_never_produces_invariant_violations() {
        let mut state = SimState::new(31337);
        for _ in 0..(365 * 3) {
            state.step_one_day();
            let violations = check_invariants(&state);
            assert!(
                violations.is_empty(),
                "invariant violations: {violations:?}"
            );
        }
    }

    #[test]
    fn save_and_load_round_trips_exactly() {
        let mut state = SimState::new(9);
        state.step_days(100);
        let json = state.to_json();
        let restored = SimState::from_json(&json).unwrap();
        assert_eq!(json, restored.to_json());
    }

    #[test]
    fn resuming_from_a_save_continues_the_same_future_as_an_uninterrupted_run() {
        let mut uninterrupted = SimState::new(555);
        uninterrupted.step_days(200);

        let mut paused = SimState::new(555);
        paused.step_days(100);
        let saved = paused.to_json();
        let mut resumed = SimState::from_json(&saved).unwrap();
        resumed.step_days(100);

        assert_eq!(uninterrupted.to_json(), resumed.to_json());
    }
}
