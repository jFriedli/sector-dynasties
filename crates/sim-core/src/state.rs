//! The authoritative simulation state and its single step function.
//!
//! `SimState` is the whole world: everything needed to save, load, or
//! resume the simulation lives here, including the per-domain RNG streams
//! (so resuming from a save produces the same future as an uninterrupted
//! run). The UI and CLI never mutate this directly; they call `step`.

use serde::{Deserialize, Serialize};

use crate::birth;
use crate::business::{self, Business, BusinessArchetype, FoundBusinessError};
use crate::career::{self, Career, CareerTrack, StartCareerError};
use crate::dynasty::{Character, Dynasty, DynastyMemberSummary};
use crate::economy;
use crate::migration;
use crate::mortality;
use crate::portrait::PortraitDescriptor;
use crate::rng::{RngDomainSummary, SimRng};
use crate::save::{load_and_migrate, SaveError};
use crate::time::SimClock;
use crate::traits;
use crate::world::{EntityId, Sector};
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
    /// Every founded business, regardless of owner. Empty on a save from
    /// before businesses existed; see `business::found_business` for how
    /// new ones are created.
    #[serde(default)]
    pub businesses: Vec<Business>,
    /// Every character's job, across every career track. Empty on a save
    /// from before careers existed; see `career::start_career` for how new
    /// ones are created.
    #[serde(default)]
    pub careers: Vec<Career>,
    economy_rng: SimRng,
    mortality_rng: SimRng,
    /// Placeholder domain on a save from before careers existed: no
    /// careers exist yet either, so this stream is never drawn from until
    /// a fresh one is started, at which point the save carries its own
    /// concrete state again. See `default_career_rng`.
    #[serde(default = "default_career_rng")]
    career_rng: SimRng,
}

/// Default for `SimState::career_rng` on saves predating the careers
/// system. Any fixed seed works: see the field's own doc comment for why
/// this never observably matters.
fn default_career_rng() -> SimRng {
    SimRng::from_seed(0, "career")
}

impl SimState {
    /// Build a brand new game from a seed: generate the sector and found
    /// the player's starting dynasty in its first city.
    pub fn new(seed: u64) -> Self {
        Self::new_with_system_count(seed, STARTING_SYSTEM_COUNT)
    }

    /// Like [`SimState::new`], but with an explicit system count instead of
    /// the bootstrap default. Exists so callers that need a different sector
    /// size (the `sim-cli` seed presets, see issue #21; the `step_days`
    /// benchmark in `benches/step_days.rs`, to compare performance at
    /// different sector sizes) don't have to reconstruct `SimState` by hand
    /// or duplicate the founding routine.
    pub fn new_with_system_count(seed: u64, system_count: u32) -> Self {
        let sector = generate_sector(seed, system_count);
        let mut character_rng = SimRng::from_seed(seed, "dynasty");

        let home_city = sector
            .systems
            .first()
            .and_then(|s| s.planets.first())
            .and_then(|p| p.countries.first())
            .and_then(|c| c.cities.first())
            .map(|c| c.id);

        let founder_id = 1;
        let founder = Character {
            id: founder_id,
            name: "Founder".to_string(),
            age_years: 28 + character_rng.next_below(20),
            alive: true,
            wealth: character_rng.range_f64(1_000.0, 10_000.0),
            home_city,
            portrait: PortraitDescriptor::generate_for_character(seed, founder_id),
            traits: traits::generate_for_character(seed, founder_id),
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
            businesses: Vec::new(),
            careers: Vec::new(),
            economy_rng: SimRng::from_seed(seed, "economy"),
            mortality_rng: SimRng::from_seed(seed, "dynasty:mortality"),
            career_rng: SimRng::from_seed(seed, "career"),
        }
    }

    /// Found a new business on behalf of the dynasty head, hosted in the
    /// city with id `host_city_id`. Fails without mutating `self` if the
    /// city doesn't exist or its specialization doesn't match what
    /// `archetype` requires; see `business::found_business_by_city_id`.
    pub fn found_business(
        &mut self,
        name: String,
        archetype: BusinessArchetype,
        host_city_id: EntityId,
    ) -> Result<EntityId, FoundBusinessError> {
        let id = self.businesses.iter().map(|b| b.id).max().unwrap_or(0) + 1;
        let founded = business::found_business_by_city_id(
            &self.sector,
            id,
            name,
            archetype,
            self.dynasty.head_character_id,
            host_city_id,
        )?;
        self.businesses.push(founded);
        Ok(id)
    }

    /// Start a new career for `character_id` on `track`, based in the city
    /// with id `employer_city_id`. Fails without mutating `self` if the
    /// city doesn't exist or the character already holds a career on that
    /// track; see `career::start_career`.
    pub fn start_career(
        &mut self,
        track: CareerTrack,
        character_id: EntityId,
        employer_city_id: EntityId,
    ) -> Result<EntityId, StartCareerError> {
        let id = self.careers.iter().map(|c| c.id).max().unwrap_or(0) + 1;
        let started = career::start_career(
            id,
            track,
            character_id,
            employer_city_id,
            &self.sector,
            &self.careers,
        )?;
        self.careers.push(started);
        Ok(id)
    }

    /// Advance the simulation by one day. Lower-frequency systems check the
    /// clock rather than running every call.
    pub fn step_one_day(&mut self) {
        self.clock.advance_one_day();

        if self.clock.is_week_boundary() {
            economy::settle_week(&mut self.sector, &mut self.economy_rng);
            business::settle_week(&mut self.businesses, &self.sector, &mut self.dynasty);
            career::settle_week(
                &mut self.careers,
                &self.sector,
                &mut self.dynasty,
                &mut self.career_rng,
            );
        }
        if self.clock.is_month_boundary() {
            migration::run_monthly_migration(&mut self.sector);
        }
        if self.clock.is_year_boundary() {
            // Age and roll mortality before checking for a birth: this way
            // a head who dies this year correctly has no child this year,
            // and a newborn is never immediately aged/mortality-rolled in
            // the same tick it's born (it would otherwise never visibly be
            // age 0 to any external observer, since both run atomically
            // here).
            mortality::age_and_roll_mortality(&mut self.dynasty, &mut self.mortality_rng);
            birth::maybe_birth_child(self.seed, &mut self.dynasty, self.clock.year());
        }
        // Further yearly demographic change (culture) hooks in here as its
        // own system; see docs/ROADMAP.md milestone "Population and
        // culture". Monthly migration is handled above.
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
            seed: self.seed,
            system_count: self.sector.systems.len(),
            city_count,
            total_population,
            dynasty_name: self.dynasty.name.clone(),
            dynasty_wealth: self.dynasty.total_wealth(),
            dynasty_members: self.dynasty.member_summaries(),
            rng_domains: self.rng_domain_summaries(),
        }
    }

    /// Debug-only snapshot of every RNG stream currently persisted on
    /// `SimState`, for the debug overlay (see issue #35). When a new
    /// persisted stream is added to this struct, add one line here so it
    /// shows up too; there is intentionally no reflection magic for this
    /// since the set of streams changes rarely and explicitness keeps this
    /// list trustworthy.
    fn rng_domain_summaries(&self) -> Vec<RngDomainSummary> {
        vec![
            RngDomainSummary::new("economy", &self.economy_rng),
            RngDomainSummary::new("dynasty:mortality", &self.mortality_rng),
            RngDomainSummary::new("career", &self.career_rng),
        ]
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("SimState always serializes")
    }

    /// Load a save, migrating it up to `SAVE_SCHEMA_VERSION` first if it's
    /// an older but known version. Rejects malformed JSON, saves missing
    /// required fields, and saves from an unsupported future version with a
    /// distinct `SaveError` rather than ever returning a corrupted
    /// `SimState`. See `crate::save`.
    pub fn from_json(json: &str) -> Result<Self, SaveError> {
        let value = load_and_migrate(json)?;
        serde_json::from_value(value).map_err(|e| SaveError::Corrupt(e.to_string()))
    }
}

/// A compact, UI/CLI-friendly view of the state, so callers don't need to
/// walk the full hierarchy just to show a status line.
///
/// `seed` and `rng_domains` exist for the debug overlay (issue #35) rather
/// than for gameplay: they let a developer confirm which seed a run started
/// from and see RNG streams actually advancing, without exposing the raw
/// `SimState`. Both flow through the existing JSON summary, so adding more
/// debug fields later never requires changing the wasm bridge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateSummary {
    pub tick: u64,
    pub year: u64,
    pub seed: u64,
    pub system_count: usize,
    pub city_count: usize,
    pub total_population: u64,
    pub dynasty_name: String,
    pub dynasty_wealth: f64,
    pub dynasty_members: Vec<DynastyMemberSummary>,
    pub rng_domains: Vec<RngDomainSummary>,
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
    fn monthly_migration_never_changes_the_sector_wide_population_total() {
        fn total_population(state: &SimState) -> u64 {
            state
                .sector
                .systems
                .iter()
                .flat_map(|s| &s.planets)
                .flat_map(|p| &p.countries)
                .flat_map(|c| &c.cities)
                .map(|city| city.population.size)
                .sum()
        }

        let mut state = SimState::new(555);
        let before = total_population(&state);
        // Several years, so multiple month boundaries (and week/year
        // boundaries) are crossed; only migration ever changes city
        // population size, so the sector-wide total must stay put.
        state.step_days(365 * 5);
        let after = total_population(&state);
        assert_eq!(
            before, after,
            "migration alone must conserve total population"
        );
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
    fn loading_malformed_json_is_rejected_as_corrupt_not_a_panic() {
        let err = SimState::from_json("not json").unwrap_err();
        assert!(matches!(err, SaveError::Corrupt(_)));
    }

    #[test]
    fn loading_a_save_missing_schema_version_is_rejected_as_corrupt() {
        let err = SimState::from_json(r#"{"seed": 1}"#).unwrap_err();
        assert!(matches!(err, SaveError::Corrupt(_)));
    }

    #[test]
    fn loading_an_unsupported_future_version_is_rejected() {
        let mut value: serde_json::Value =
            serde_json::from_str(&SimState::new(1).to_json()).unwrap();
        value["schema_version"] = serde_json::Value::from(SAVE_SCHEMA_VERSION + 1);

        let err = SimState::from_json(&value.to_string()).unwrap_err();
        assert!(matches!(
            err,
            SaveError::UnsupportedFutureVersion {
                found,
                supported
            } if found == SAVE_SCHEMA_VERSION + 1 && supported == SAVE_SCHEMA_VERSION
        ));
    }

    #[test]
    fn new_with_system_count_respects_the_requested_count() {
        let state = SimState::new_with_system_count(7, 5);
        assert_eq!(state.sector.systems.len(), 5);
    }

    #[test]
    fn new_matches_new_with_system_count_at_the_default() {
        let a = SimState::new(2020);
        let b = SimState::new_with_system_count(2020, STARTING_SYSTEM_COUNT);
        assert_eq!(a.to_json(), b.to_json());
    }

    #[test]
    fn summary_reports_the_seed_and_the_rng_domains() {
        let state = SimState::new(2026);
        let summary = state.summary();
        assert_eq!(summary.seed, 2026);
        assert_eq!(summary.rng_domains.len(), 3);
        assert_eq!(summary.rng_domains[0].domain, "economy");
        assert_eq!(summary.rng_domains[1].domain, "dynasty:mortality");
        assert_eq!(summary.rng_domains[2].domain, "career");
    }

    #[test]
    fn summary_reports_the_founding_dynasty_member_as_head() {
        let state = SimState::new(2026);
        let summary = state.summary();
        assert_eq!(summary.dynasty_members.len(), 1);
        assert_eq!(
            summary.dynasty_members[0].role,
            crate::dynasty::DynastyRole::Head
        );
        assert!(summary.dynasty_members[0].alive);
    }

    #[test]
    fn same_seed_and_step_count_produce_an_identical_summary() {
        let mut a = SimState::new(2026);
        let mut b = SimState::new(2026);
        a.step_days(400);
        b.step_days(400);
        assert_eq!(a.summary(), b.summary());
    }

    #[test]
    fn the_economy_rng_fingerprint_advances_after_a_settled_week() {
        let state = SimState::new(2026);
        let before = state.summary().rng_domains[0].fingerprint.clone();

        let mut state = state;
        state.step_days(7);
        let after = state.summary().rng_domains[0].fingerprint.clone();

        assert_ne!(before, after);
    }

    #[test]
    fn a_fertile_head_can_gain_a_child_over_enough_years() {
        // Search a small range of seeds for one whose founder rolls a
        // birth within a generous window, rather than depending on a
        // single brittle seed (the founder's starting age is itself
        // seed-dependent).
        const YEARS: u32 = 40;
        let seed = (0..200u64)
            .find(|&seed| {
                let mut probe = SimState::new(seed);
                probe.step_days(YEARS * 360);
                probe.dynasty.members.len() > 1
            })
            .expect("expected at least one seed in range to produce a birth within 40 years");

        let mut a = SimState::new(seed);
        a.step_days(YEARS * 360);
        let mut b = SimState::new(seed);
        b.step_days(YEARS * 360);

        assert!(a.dynasty.members.len() > 1, "expected the dynasty to grow");
        // Same seed, same step count: the birth (its tick, id, portrait,
        // and traits) must reproduce exactly, not just "a birth happened".
        assert_eq!(a.to_json(), b.to_json());
        assert!(check_invariants(&a).is_empty());
    }

    #[test]
    fn the_mortality_rng_fingerprint_advances_after_a_year_passes() {
        let state = SimState::new(2026);
        let before = state.summary().rng_domains[1].fingerprint.clone();

        let mut state = state;
        state.step_days(crate::time::DAYS_PER_YEAR as u32);
        let after = state.summary().rng_domains[1].fingerprint.clone();

        assert_ne!(before, after);
    }

    #[test]
    fn the_founders_age_increases_by_one_after_a_year_passes() {
        // Aging happens before that year's mortality roll (see
        // `mortality::age_and_roll_mortality`), so the age increment holds
        // regardless of whether the founder survives the roll.
        let mut state = SimState::new(2026);
        let starting_age = state.dynasty.head().unwrap().age_years;

        state.step_days(crate::time::DAYS_PER_YEAR as u32);

        assert_eq!(state.dynasty.head().unwrap().age_years, starting_age + 1);
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

    #[test]
    fn starting_a_career_hires_the_founder_at_the_bottom_rung() {
        let mut state = SimState::new(2026);
        let home_city = state.dynasty.head().unwrap().home_city.unwrap();

        let career_id = state
            .start_career(
                CareerTrack::Corporate,
                state.dynasty.head_character_id,
                home_city,
            )
            .expect("founder should be hirable in their home city");

        let career = state.careers.iter().find(|c| c.id == career_id).unwrap();
        assert_eq!(career.level, 0);
        assert_eq!(career.job_title(), "Analyst");
    }

    #[test]
    fn starting_a_second_career_on_the_same_track_is_refused() {
        let mut state = SimState::new(2026);
        let home_city = state.dynasty.head().unwrap().home_city.unwrap();
        let head_id = state.dynasty.head_character_id;

        state
            .start_career(CareerTrack::Corporate, head_id, home_city)
            .unwrap();
        let err = state
            .start_career(CareerTrack::Corporate, head_id, home_city)
            .expect_err("the founder already holds a Corporate career");
        assert_eq!(
            err,
            StartCareerError::AlreadyOnTrack {
                character_id: head_id,
                track: CareerTrack::Corporate,
            }
        );
    }

    #[test]
    fn a_working_founder_accrues_salary_over_time() {
        let mut state = SimState::new(2026);
        let home_city = state.dynasty.head().unwrap().home_city.unwrap();
        let head_id = state.dynasty.head_character_id;
        state
            .start_career(CareerTrack::Corporate, head_id, home_city)
            .unwrap();

        let wealth_before = state.dynasty.head().unwrap().wealth;
        state.step_days(365);
        let wealth_after = state.dynasty.head().unwrap().wealth;

        assert!(
            wealth_after > wealth_before,
            "a year of salaried employment should grow the founder's wealth"
        );
    }

    #[test]
    fn same_seed_and_step_count_produce_identical_state_with_a_career_in_progress() {
        fn run(seed: u64) -> String {
            let mut state = SimState::new(seed);
            let home_city = state.dynasty.head().unwrap().home_city.unwrap();
            let head_id = state.dynasty.head_character_id;
            state
                .start_career(CareerTrack::Corporate, head_id, home_city)
                .unwrap();
            state.step_days(400);
            state.to_json()
        }

        assert_eq!(run(2026), run(2026));
    }

    #[test]
    fn the_career_rng_fingerprint_advances_after_a_settled_week_with_a_career_in_progress() {
        let mut state = SimState::new(2026);
        let home_city = state.dynasty.head().unwrap().home_city.unwrap();
        let head_id = state.dynasty.head_character_id;
        state
            .start_career(CareerTrack::Corporate, head_id, home_city)
            .unwrap();

        let before = state.summary().rng_domains[2].fingerprint.clone();
        state.step_days(7);
        let after = state.summary().rng_domains[2].fingerprint.clone();

        assert_ne!(before, after);
    }
}
