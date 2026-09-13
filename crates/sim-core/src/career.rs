//! Career tracks: a job ladder, deterministic promotion, and salary accrual
//! on the weekly settlement cadence.
//!
//! Issue #38 added a minimal `promotion_chance(&Character) -> f64`: a pure
//! trait-reading function, added only to prove `crate::traits::Trait`
//! actually affects a system rather than sitting decorative, with no real
//! job concept behind it. Issue #26 ("Add a Corporate career track") is
//! that real system, and issue #125 asked for the old modifier to be folded
//! into it rather than living on as a second promotion-odds concept; this
//! module is that fold. `promotion_chance` below is unchanged in behavior
//! from the #38 version, now actually driving the weekly promotion roll in
//! [`settle_week`] instead of standing alone.
//!
//! Only [`CareerTrack::Corporate`] exists today. A future Political or
//! Academic track (see epic #3) adds a variant and a match arm to each
//! function on `CareerTrack`, the same pattern `crate::business::
//! BusinessArchetype` uses for business archetypes: a new track's ladder,
//! pay, and pacing live in its own match arms rather than a shared formula,
//! and nothing outside this module needs to change.
//!
//! [`promotion_chance_with_favors`] is issue #67's consumer: outstanding
//! favors owed to a character (see `crate::favor`) are a legible,
//! relationship-based source of influence over their own promotion odds, on
//! top of the trait-based modifier below, and [`settle_week`] rolls
//! promotion using it rather than the plain trait-only `promotion_chance`,
//! so a character's real career outcomes actually feel favor leverage.

use serde::{Deserialize, Serialize};

use crate::dynasty::{Character, Dynasty};
use crate::favor::{self, Favor};
use crate::rng::SimRng;
use crate::traits::Trait;
use crate::world::{CitySpecialization, EntityId, Sector};

/// Base probability, per week, that an eligible character is promoted,
/// before trait modifiers. See [`promotion_chance`].
pub const BASE_PROMOTION_CHANCE: f64 = 0.3;

/// This character's promotion chance for one weekly evaluation, adjusted by
/// any traits that plausibly affect career advancement.
///
/// `Ambitious` characters push harder for advancement and `Diligent`
/// characters get noticed for consistent work, so both raise the chance.
/// `Sickly` characters miss opportunities to time off, lowering it. Every
/// other trait is neutral to promotion odds: traits are allowed to matter to
/// some systems and not others.
pub fn promotion_chance(character: &Character) -> f64 {
    let modifier: f64 = character
        .traits
        .iter()
        .map(|trait_| match trait_ {
            Trait::Ambitious => 0.15,
            Trait::Diligent => 0.10,
            Trait::Sickly => -0.10,
            Trait::Frugal | Trait::Charismatic | Trait::Reckless => 0.0,
        })
        .sum();

    (BASE_PROMOTION_CHANCE + modifier).clamp(0.0, 1.0)
}

/// Maximum additive boost outstanding favors can contribute to
/// [`promotion_chance_with_favors`], however much leverage is held. Kept
/// well below the traits' own headroom so a character cannot favor-trade
/// their way to a guaranteed promotion.
pub const MAX_FAVOR_PROMOTION_BONUS: f64 = 0.25;

/// Favor leverage at which the bonus reaches half of
/// [`MAX_FAVOR_PROMOTION_BONUS`]. Diminishing returns: a single large favor
/// still matters, but stockpiling many small ones can't push the bonus past
/// the cap.
pub const FAVOR_PROMOTION_HALF_LEVERAGE: f64 = 3.0;

/// The additive promotion-chance bonus for holding `leverage` worth of
/// outstanding favors (the sum of magnitudes owed *to* a character, see
/// [`favor::leverage_held_by`]). Zero leverage gives zero bonus; the bonus
/// rises with diminishing returns toward [`MAX_FAVOR_PROMOTION_BONUS`] and
/// never reaches or exceeds it.
fn favor_promotion_bonus(leverage: f64) -> f64 {
    if !leverage.is_finite() || leverage <= 0.0 {
        return 0.0;
    }
    MAX_FAVOR_PROMOTION_BONUS * (leverage / (leverage + FAVOR_PROMOTION_HALF_LEVERAGE))
}

/// Like [`promotion_chance`], but also lets favors owed to `character`
/// nudge the odds upward: a character who can call in favors from
/// colleagues or patrons has a real, legible edge in getting appointed,
/// independent of their own traits. Favors where `character` is the debtor
/// (they owe someone else) don't affect their own odds here, see
/// `crate::favor::leverage_held_by`.
pub fn promotion_chance_with_favors(character: &Character, favors: &[Favor]) -> f64 {
    let leverage = favor::leverage_held_by(favors, character.id);
    (promotion_chance(character) + favor_promotion_bonus(leverage)).clamp(0.0, 1.0)
}

/// Which career track a [`Career`] belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CareerTrack {
    Corporate,
}

/// A character can't be promoted the week they're hired (or the week after a
/// promotion): this many settled weeks at the current level must pass
/// first. Applies to every track today; a future track with genuinely
/// different pacing would give this its own match arm rather than a shared
/// constant.
const MIN_WEEKS_BEFORE_PROMOTION: u32 = 12;

/// How much larger a salary can grow purely from a healthy employer
/// treasury, relative to a treasury of zero. See [`treasury_salary_factor`].
const TREASURY_SALARY_HALF_SCALE: f64 = 50_000.0;

impl CareerTrack {
    /// Job titles for this track's ladder, lowest rung first. Index into
    /// this (clamped) is a [`Career::level`].
    fn job_titles(&self) -> &'static [&'static str] {
        match self {
            CareerTrack::Corporate => &["Analyst", "Manager", "Director"],
        }
    }

    /// The highest valid `level` on this track's ladder.
    pub fn max_level(&self) -> u8 {
        (self.job_titles().len() - 1) as u8
    }

    /// The job title for `level` on this track, clamped to the ladder's top
    /// rung rather than panicking on a corrupt out-of-range level.
    pub fn job_title(&self, level: u8) -> &'static str {
        let titles = self.job_titles();
        titles[(level as usize).min(titles.len() - 1)]
    }

    /// Base weekly salary at `level`, before the employer city's
    /// specialization and treasury adjust it. See [`Career::weekly_salary`].
    fn base_weekly_salary(&self, level: u8) -> f64 {
        let level = level.min(self.max_level());
        match self {
            CareerTrack::Corporate => match level {
                0 => 60.0,  // Analyst
                1 => 110.0, // Manager
                _ => 190.0, // Director
            },
        }
    }

    /// How much this track's salary is scaled by the employer city's
    /// specialization. A corporate job pays best in a city actually
    /// organized around finance; a mining town's corporate jobs are the
    /// least lucrative of the bunch, mirroring `economy.rs`'s own
    /// specialization multiplier ordering.
    fn specialization_salary_multiplier(&self, specialization: CitySpecialization) -> f64 {
        match self {
            CareerTrack::Corporate => match specialization {
                CitySpecialization::Finance => 1.3,
                CitySpecialization::Research => 1.15,
                CitySpecialization::Manufacturing => 1.05,
                CitySpecialization::Logistics => 1.0,
                CitySpecialization::Mining => 0.95,
            },
        }
    }
}

/// How much an employer city's treasury scales its salaries: `1.0` at an
/// empty treasury, rising toward (never reaching) `2.0` as treasury grows,
/// so a struggling city still pays a baseline salary and a thriving one
/// pays up to twice as much, without letting an unbounded treasury blow
/// salaries up without limit. Defensively clamps a corrupt negative
/// treasury to `0.0` rather than trusting the caller (see `invariants.rs`,
/// which already flags a negative treasury as a bug).
fn treasury_salary_factor(treasury: f64) -> f64 {
    let treasury = treasury.max(0.0);
    1.0 + treasury / (treasury + TREASURY_SALARY_HALF_SCALE)
}

/// A character's job on some [`CareerTrack`]: rung on the ladder, employer
/// city, and tenure at the current rung.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Career {
    pub id: EntityId,
    pub track: CareerTrack,
    /// The dynasty member holding this job.
    pub character_id: EntityId,
    /// The city this job is based in. Looked up through `Sector::find_city`
    /// each settlement rather than holding a direct reference, per
    /// `docs/ARCHITECTURE.md`'s "reference by id" rule.
    pub employer_city_id: EntityId,
    /// Index into the track's job ladder; always `<= track.max_level()`.
    pub level: u8,
    /// Weeks settled since starting this level (reset on promotion). Gates
    /// promotion eligibility; see [`MIN_WEEKS_BEFORE_PROMOTION`].
    pub weeks_at_level: u32,
}

impl Career {
    pub fn is_valid(&self) -> bool {
        self.level <= self.track.max_level()
    }

    /// The job title this career currently holds.
    pub fn job_title(&self) -> &'static str {
        self.track.job_title(self.level)
    }

    /// This week's salary, given the employer city's current
    /// specialization and treasury. Pure function of already-validated
    /// sector state (see the module docs' "reference by id" note), so
    /// settlement stays deterministic given the current sector without a
    /// dedicated RNG stream for pay itself.
    fn weekly_salary(&self, employer_city: &crate::world::City) -> f64 {
        self.track.base_weekly_salary(self.level)
            * self
                .track
                .specialization_salary_multiplier(employer_city.specialization)
            * treasury_salary_factor(employer_city.treasury)
    }
}

/// Why starting a career was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartCareerError {
    /// `employer_city_id` doesn't resolve to any city in the sector.
    UnknownEmployerCity(EntityId),
    /// `character_id` already holds a career on this track. A character can
    /// hold at most one job per track at a time.
    AlreadyOnTrack {
        character_id: EntityId,
        track: CareerTrack,
    },
}

/// Start a new career for `character_id` on `track`, based in
/// `employer_city_id`. Fails without mutating anything if the employer city
/// doesn't exist, or the character already holds a career on this track.
/// Always starts at the bottom rung (`level == 0`).
pub fn start_career(
    id: EntityId,
    track: CareerTrack,
    character_id: EntityId,
    employer_city_id: EntityId,
    sector: &Sector,
    existing_careers: &[Career],
) -> Result<Career, StartCareerError> {
    if sector.find_city(employer_city_id).is_none() {
        return Err(StartCareerError::UnknownEmployerCity(employer_city_id));
    }
    if existing_careers
        .iter()
        .any(|c| c.character_id == character_id && c.track == track)
    {
        return Err(StartCareerError::AlreadyOnTrack {
            character_id,
            track,
        });
    }

    Ok(Career {
        id,
        track,
        character_id,
        employer_city_id,
        level: 0,
        weeks_at_level: 0,
    })
}

/// Run one weekly settlement over every career: pay this week's salary into
/// the holder's `wealth`, then roll for promotion using a dedicated
/// `"career"`-domain `SimRng` stream once the character has held the
/// current level for at least `MIN_WEEKS_BEFORE_PROMOTION` weeks. The
/// promotion roll reads `favors` through [`promotion_chance_with_favors`],
/// so leverage a character holds over colleagues (issue #67) measurably
/// helps their own real career outcomes, not just a standalone comparison.
/// A career whose employer city no longer resolves (a stale reference,
/// which `invariants.rs` also flags) is skipped rather than panicking; a
/// career whose holder is no longer a resolvable dynasty member (should
/// not happen outside a corrupt save) simply accrues no salary that week
/// but still advances tenure, so a later fix doesn't need to replay
/// history.
pub fn settle_week(
    careers: &mut [Career],
    sector: &Sector,
    dynasty: &mut Dynasty,
    favors: &[Favor],
    rng: &mut SimRng,
) {
    for career in careers.iter_mut() {
        career.weeks_at_level += 1;

        let Some(city) = sector.find_city(career.employer_city_id) else {
            continue;
        };

        let holder = dynasty
            .members
            .iter_mut()
            .find(|c| c.id == career.character_id);
        let chance = holder
            .as_deref()
            .map(|character| promotion_chance_with_favors(character, favors));

        if let Some(holder) = holder {
            holder.wealth += career.weekly_salary(city);
        }

        // Always draw from the stream, even when not yet eligible for
        // promotion (below minimum tenure, or already at the top rung):
        // this keeps `career`'s RNG stream advancing every settled week
        // regardless of any one career's tenure, the same way `economy`'s
        // stream always draws twice per city per week rather than only
        // when a particular condition holds.
        let eligible = career.level < career.track.max_level()
            && career.weeks_at_level >= MIN_WEEKS_BEFORE_PROMOTION;
        let promoted = rng.chance(chance.unwrap_or(BASE_PROMOTION_CHANCE));
        if eligible && promoted {
            career.level += 1;
            career.weeks_at_level = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portrait::PortraitDescriptor;
    use crate::world::{City, Country, GovernmentProfile, Planet, PopulationGroup, StarSystem};

    fn character_with_traits(id: EntityId, traits: Vec<Trait>) -> Character {
        Character {
            id,
            name: "Test".to_string(),
            age_years: 30,
            alive: true,
            wealth: 0.0,
            home_city: None,
            portrait: PortraitDescriptor::generate_for_character(0, id),
            traits,
        }
    }

    // --- promotion_chance (unchanged behavior from the #38 placeholder) ---

    #[test]
    fn ambitious_characters_have_higher_promotion_odds_than_a_baseline_peer() {
        let baseline = character_with_traits(1, vec![Trait::Frugal]);
        let ambitious = character_with_traits(2, vec![Trait::Ambitious]);

        assert!(promotion_chance(&ambitious) > promotion_chance(&baseline));
    }

    #[test]
    fn sickly_characters_have_lower_promotion_odds_than_a_baseline_peer() {
        let baseline = character_with_traits(1, vec![Trait::Charismatic]);
        let sickly = character_with_traits(2, vec![Trait::Sickly]);

        assert!(promotion_chance(&sickly) < promotion_chance(&baseline));
    }

    #[test]
    fn promotion_chance_never_leaves_the_probability_range() {
        let all_negative = character_with_traits(1, vec![Trait::Sickly, Trait::Sickly]);
        let all_positive = character_with_traits(2, vec![Trait::Ambitious, Trait::Diligent]);

        assert!((0.0..=1.0).contains(&promotion_chance(&all_negative)));
        assert!((0.0..=1.0).contains(&promotion_chance(&all_positive)));
    }

    #[test]
    fn a_character_with_no_traits_gets_exactly_the_base_chance() {
        let bare = character_with_traits(1, vec![]);
        assert_eq!(promotion_chance(&bare), BASE_PROMOTION_CHANCE);
    }

    fn favor_owed_to(creditor_id: EntityId, debtor_id: EntityId, magnitude: f64) -> Favor {
        Favor {
            creditor_id,
            debtor_id,
            magnitude,
        }
    }

    // --- promotion_chance_with_favors ---

    #[test]
    fn a_character_owed_a_favor_has_higher_promotion_odds_than_an_identical_peer_without_one() {
        // Same traits (and thus the same base `promotion_chance`) for both;
        // only the outstanding favor differs, so this isolates the favor
        // system's effect on the consuming mechanic per issue #67's
        // required comparative test.
        let owed = character_with_traits(1, vec![Trait::Frugal]);
        let not_owed = character_with_traits(2, vec![Trait::Frugal]);

        let favors = vec![favor_owed_to(owed.id, 99, 5.0)];

        assert_eq!(
            promotion_chance_with_favors(&not_owed, &favors),
            promotion_chance(&not_owed),
            "a character with no favors owed to them gets no bonus"
        );
        assert!(
            promotion_chance_with_favors(&owed, &favors)
                > promotion_chance_with_favors(&not_owed, &favors),
            "being owed a favor must measurably raise promotion odds over an identical peer"
        );
    }

    #[test]
    fn owing_a_favor_does_not_affect_the_debtors_own_promotion_odds() {
        let debtor = character_with_traits(1, vec![Trait::Frugal]);
        let favors = vec![favor_owed_to(99, debtor.id, 5.0)];

        assert_eq!(
            promotion_chance_with_favors(&debtor, &favors),
            promotion_chance(&debtor)
        );
    }

    #[test]
    fn favor_bonus_never_reaches_the_cap_and_never_leaves_the_probability_range() {
        let character = character_with_traits(1, vec![Trait::Ambitious, Trait::Diligent]);
        let huge_favor = vec![favor_owed_to(character.id, 99, 1_000_000.0)];

        let chance = promotion_chance_with_favors(&character, &huge_favor);
        assert!((0.0..=1.0).contains(&chance));
        assert!(chance < promotion_chance(&character) + MAX_FAVOR_PROMOTION_BONUS);
    }

    #[test]
    fn multiple_favors_owed_to_the_same_character_stack() {
        let character = character_with_traits(1, vec![Trait::Frugal]);
        let one_favor = vec![favor_owed_to(character.id, 99, 2.0)];
        let two_favors = vec![
            favor_owed_to(character.id, 99, 2.0),
            favor_owed_to(character.id, 100, 2.0),
        ];

        assert!(
            promotion_chance_with_favors(&character, &two_favors)
                > promotion_chance_with_favors(&character, &one_favor)
        );
    }

    // --- ladder shape ---

    #[test]
    fn corporate_ladder_runs_analyst_to_manager_to_director() {
        let track = CareerTrack::Corporate;
        assert_eq!(track.job_title(0), "Analyst");
        assert_eq!(track.job_title(1), "Manager");
        assert_eq!(track.job_title(2), "Director");
        assert_eq!(track.max_level(), 2);
    }

    #[test]
    fn a_level_past_the_top_rung_clamps_rather_than_panics() {
        let track = CareerTrack::Corporate;
        assert_eq!(track.job_title(200), track.job_title(track.max_level()));
    }

    // --- treasury_salary_factor ---

    #[test]
    fn treasury_salary_factor_stays_within_one_and_two() {
        assert_eq!(treasury_salary_factor(0.0), 1.0);
        assert!(treasury_salary_factor(1_000_000.0) < 2.0);
        assert!(treasury_salary_factor(1_000_000.0) > 1.9);
    }

    #[test]
    fn treasury_salary_factor_clamps_a_corrupt_negative_treasury() {
        assert_eq!(treasury_salary_factor(-500.0), treasury_salary_factor(0.0));
    }

    #[test]
    fn a_richer_treasury_pays_more_holding_everything_else_constant() {
        let poor = treasury_salary_factor(0.0);
        let rich = treasury_salary_factor(100_000.0);
        assert!(rich > poor);
    }

    // --- start_career ---

    fn city_with(specialization: CitySpecialization, treasury: f64) -> City {
        City {
            id: 1,
            name: "Test City".to_string(),
            specialization,
            population: PopulationGroup {
                size: 10_000,
                average_wealth: 100.0,
                unemployment_rate: 0.1,
            },
            treasury,
            recent_output_index: 1.0,
        }
    }

    fn sector_with(city: City) -> Sector {
        Sector {
            seed: 1,
            name: "Test Sector".to_string(),
            systems: vec![StarSystem {
                id: 1,
                name: "Test System".to_string(),
                planets: vec![Planet {
                    id: 1,
                    name: "Test Planet".to_string(),
                    resource_tags: Vec::new(),
                    resource_abundance: Vec::new(),
                    countries: vec![Country {
                        id: 1,
                        name: "Test Country".to_string(),
                        government: GovernmentProfile {
                            federalism: 0.5,
                            franchise: 0.5,
                            economic_liberalism: 0.5,
                            press_freedom: 0.5,
                            legislative_strength: 0.5,
                            judicial_independence: 0.5,
                        },
                        backstory: String::new(),
                        social_mobility: 0.5,
                        union_power: 0.5,
                        cities: vec![city],
                    }],
                }],
            }],
        }
    }

    #[test]
    fn starting_a_career_in_an_unknown_city_is_refused() {
        let sector = Sector {
            seed: 1,
            name: "Empty Sector".to_string(),
            systems: Vec::new(),
        };
        let err = start_career(1, CareerTrack::Corporate, 7, 999, &sector, &[])
            .expect_err("an empty sector has no city 999 to hire in");
        assert_eq!(err, StartCareerError::UnknownEmployerCity(999));
    }

    #[test]
    fn starting_a_career_in_a_known_city_succeeds_at_the_bottom_rung() {
        let sector = sector_with(city_with(CitySpecialization::Finance, 0.0));
        let career = start_career(1, CareerTrack::Corporate, 7, 1, &sector, &[])
            .expect("should hire into a known city");
        assert_eq!(career.level, 0);
        assert_eq!(career.weeks_at_level, 0);
        assert_eq!(career.job_title(), "Analyst");
    }

    #[test]
    fn a_character_already_on_a_track_cannot_start_a_second_career_on_it() {
        let sector = sector_with(city_with(CitySpecialization::Finance, 0.0));
        let existing = vec![start_career(1, CareerTrack::Corporate, 7, 1, &sector, &[]).unwrap()];

        let err = start_career(2, CareerTrack::Corporate, 7, 1, &sector, &existing)
            .expect_err("character 7 already holds a Corporate career");
        assert_eq!(
            err,
            StartCareerError::AlreadyOnTrack {
                character_id: 7,
                track: CareerTrack::Corporate,
            }
        );
    }

    // --- settle_week ---

    fn owner(id: EntityId) -> Character {
        Character {
            id,
            name: "Owner".to_string(),
            age_years: 40,
            alive: true,
            wealth: 1_000.0,
            home_city: Some(1),
            portrait: PortraitDescriptor::generate_for_character(0, id),
            traits: Vec::new(),
        }
    }

    #[test]
    fn settlement_pays_salary_into_the_holders_wealth() {
        let city = city_with(CitySpecialization::Finance, 0.0);
        let sector = sector_with(city.clone());
        let mut dynasty = Dynasty {
            name: "House Test".to_string(),
            head_character_id: 7,
            members: vec![owner(7)],
        };
        let mut careers =
            vec![start_career(1, CareerTrack::Corporate, 7, 1, &sector, &[]).unwrap()];
        let mut rng = SimRng::from_seed(1, "career");

        let wealth_before = dynasty.members[0].wealth;
        settle_week(&mut careers, &sector, &mut dynasty, &[], &mut rng);

        let expected_salary = careers[0].weekly_salary(&city);
        assert!(expected_salary > 0.0);
        assert_eq!(dynasty.members[0].wealth, wealth_before + expected_salary);
        assert_eq!(careers[0].weeks_at_level, 1);
    }

    #[test]
    fn settlement_skips_a_career_whose_employer_city_no_longer_resolves() {
        let city = city_with(CitySpecialization::Finance, 0.0);
        let hire_sector = sector_with(city);
        let mut careers =
            vec![start_career(1, CareerTrack::Corporate, 7, 1, &hire_sector, &[]).unwrap()];
        let empty_sector = Sector {
            seed: 1,
            name: "Empty Sector".to_string(),
            systems: Vec::new(),
        };
        let mut dynasty = Dynasty {
            name: "House Test".to_string(),
            head_character_id: 7,
            members: vec![owner(7)],
        };
        let mut rng = SimRng::from_seed(1, "career");

        settle_week(&mut careers, &empty_sector, &mut dynasty, &[], &mut rng);

        assert_eq!(
            dynasty.members[0].wealth, 1_000.0,
            "no employer city, no salary"
        );
        assert_eq!(careers[0].weeks_at_level, 1, "tenure still advances");
    }

    #[test]
    fn no_promotion_is_rolled_before_the_minimum_tenure_is_reached() {
        let city = city_with(CitySpecialization::Finance, 0.0);
        let sector = sector_with(city);
        let mut dynasty = Dynasty {
            name: "House Test".to_string(),
            head_character_id: 7,
            // Stack every positive trait so even a very lucky RNG stream
            // would still be caught if tenure gating were broken.
            members: vec![Character {
                traits: vec![Trait::Ambitious, Trait::Diligent],
                ..owner(7)
            }],
        };
        let mut careers =
            vec![start_career(1, CareerTrack::Corporate, 7, 1, &sector, &[]).unwrap()];
        let mut rng = SimRng::from_seed(1, "career");

        for _ in 0..(MIN_WEEKS_BEFORE_PROMOTION - 1) {
            settle_week(&mut careers, &sector, &mut dynasty, &[], &mut rng);
        }

        assert_eq!(careers[0].level, 0, "no promotion before minimum tenure");
    }

    #[test]
    fn a_character_is_eventually_promoted_off_the_bottom_rung() {
        let city = city_with(CitySpecialization::Finance, 0.0);
        let sector = sector_with(city);
        let mut dynasty = Dynasty {
            name: "House Test".to_string(),
            head_character_id: 7,
            members: vec![Character {
                traits: vec![Trait::Ambitious, Trait::Diligent],
                ..owner(7)
            }],
        };
        let mut careers =
            vec![start_career(1, CareerTrack::Corporate, 7, 1, &sector, &[]).unwrap()];
        let mut rng = SimRng::from_seed(1, "career");

        // Generously long window: at a >50% weekly chance once eligible,
        // failing to promote across two full years would be a red flag,
        // not bad luck.
        for _ in 0..104 {
            settle_week(&mut careers, &sector, &mut dynasty, &[], &mut rng);
        }

        assert!(careers[0].level > 0, "expected at least one promotion");
    }

    #[test]
    fn promotion_never_advances_past_the_ladders_top_rung() {
        let city = city_with(CitySpecialization::Finance, 0.0);
        let sector = sector_with(city);
        let mut dynasty = Dynasty {
            name: "House Test".to_string(),
            head_character_id: 7,
            members: vec![Character {
                traits: vec![Trait::Ambitious, Trait::Diligent],
                ..owner(7)
            }],
        };
        let mut careers =
            vec![start_career(1, CareerTrack::Corporate, 7, 1, &sector, &[]).unwrap()];
        let mut rng = SimRng::from_seed(1, "career");

        for _ in 0..2000 {
            settle_week(&mut careers, &sector, &mut dynasty, &[], &mut rng);
        }

        assert_eq!(careers[0].level, CareerTrack::Corporate.max_level());
    }

    #[test]
    fn same_seed_weekly_settlement_is_deterministic() {
        let run = || {
            let city = city_with(CitySpecialization::Finance, 0.0);
            let sector = sector_with(city);
            let mut dynasty = Dynasty {
                name: "House Test".to_string(),
                head_character_id: 7,
                members: vec![owner(7)],
            };
            let mut careers =
                vec![start_career(1, CareerTrack::Corporate, 7, 1, &sector, &[]).unwrap()];
            let mut rng = SimRng::from_seed(42, "career");
            for _ in 0..40 {
                settle_week(&mut careers, &sector, &mut dynasty, &[], &mut rng);
            }
            (
                careers[0].level,
                careers[0].weeks_at_level,
                dynasty.members[0].wealth,
            )
        };

        assert_eq!(run(), run());
    }
}
