//! Yearly character aging and age-based mortality.
//!
//! Part of #3: characters previously never aged or died after creation.
//! [`age_and_roll_mortality`] runs once per in-world year (see
//! `SimClock::is_year_boundary`): every living character's `age_years`
//! increases by one, then faces a mortality roll drawn from a dedicated
//! `SimRng` stream (domain `"dynasty:mortality"`, see `SimState`), kept
//! separate from every other stream per `crate::rng`'s per-domain design so
//! a death roll never perturbs, say, the economy stream's sequence.
//!
//! A dead character is marked `alive = false` and stays in
//! `Dynasty::members` rather than being removed (`crate::invariants` and
//! `crate::dynasty::Character::is_valid` still see them); they never age or
//! roll for mortality again once dead.
//!
//! The mortality curve below ([`MORTALITY_CURVE`]) is a placeholder per the
//! issue's notes: rough, monotonically increasing brackets rather than a
//! researched actuarial table. Tuning it for pacing is future content work.

use crate::dynasty::Dynasty;
use crate::rng::SimRng;

/// `(min_age, annual_probability)` brackets in ascending age order. A
/// character's probability is that of the highest bracket whose `min_age`
/// they've reached; the first bracket must start at age 0 so every age is
/// covered. Placeholder values, not balanced.
const MORTALITY_CURVE: &[(u32, f64)] = &[
    (0, 0.002),
    (40, 0.004),
    (60, 0.015),
    (75, 0.045),
    (90, 0.12),
    (105, 0.35),
];

/// The annual probability that a character of `age_years` dies this year,
/// per the placeholder [`MORTALITY_CURVE`].
pub fn annual_mortality_probability(age_years: u32) -> f64 {
    let mut probability = MORTALITY_CURVE[0].1;
    for &(min_age, bracket_probability) in MORTALITY_CURVE {
        if age_years < min_age {
            break;
        }
        probability = bracket_probability;
    }
    probability
}

/// Age every living character in `dynasty` by one year and roll mortality
/// for each, using `rng` (the dedicated `"dynasty:mortality"` stream).
///
/// Iterates `dynasty.members` in order so the sequence of rolls drawn from
/// `rng` is a deterministic function of the dynasty's member order, matching
/// the same-seed determinism guarantee the rest of the crate relies on.
/// Already-dead characters are skipped entirely: no aging, no roll, so a
/// dead character's `age_years` is frozen at their age of death.
pub fn age_and_roll_mortality(dynasty: &mut Dynasty, rng: &mut SimRng) {
    for character in &mut dynasty.members {
        if !character.alive {
            continue;
        }
        character.age_years += 1;
        if rng.chance(annual_mortality_probability(character.age_years)) {
            character.alive = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynasty::Character;
    use crate::portrait::PortraitDescriptor;

    fn character(id: crate::world::EntityId, age_years: u32, alive: bool) -> Character {
        Character {
            id,
            name: format!("Character {id}"),
            age_years,
            alive,
            wealth: 100.0,
            home_city: None,
            portrait: PortraitDescriptor::generate_for_character(0, id),
            traits: crate::traits::generate_for_character(0, id),
        }
    }

    fn dynasty_of(members: Vec<Character>) -> Dynasty {
        let head_character_id = members.first().map(|c| c.id).unwrap_or(0);
        Dynasty {
            name: "Test Dynasty".to_string(),
            head_character_id,
            members,
        }
    }

    #[test]
    fn mortality_probability_is_monotonically_non_decreasing_with_age() {
        let mut previous = annual_mortality_probability(0);
        for age in 1..200 {
            let current = annual_mortality_probability(age);
            assert!(
                current >= previous,
                "mortality probability decreased at age {age}: {current} < {previous}"
            );
            previous = current;
        }
    }

    #[test]
    fn mortality_probability_stays_a_valid_probability() {
        for age in 0..300 {
            let p = annual_mortality_probability(age);
            assert!((0.0..=1.0).contains(&p), "age {age} gave probability {p}");
        }
    }

    #[test]
    fn a_living_characters_age_increases_by_one_year() {
        let mut dynasty = dynasty_of(vec![character(1, 30, true)]);
        let mut rng = SimRng::from_seed(1, "dynasty:mortality");
        age_and_roll_mortality(&mut dynasty, &mut rng);
        assert_eq!(dynasty.members[0].age_years, 31);
    }

    #[test]
    fn a_dead_character_never_ages_or_rolls_again() {
        let mut dynasty = dynasty_of(vec![character(1, 50, false)]);
        let mut rng = SimRng::from_seed(1, "dynasty:mortality");
        age_and_roll_mortality(&mut dynasty, &mut rng);
        // Age is frozen and the character stays dead, not re-animated.
        assert_eq!(dynasty.members[0].age_years, 50);
        assert!(!dynasty.members[0].alive);
    }

    #[test]
    fn a_dead_character_never_re_animates_across_many_years() {
        let mut dynasty = dynasty_of(vec![character(1, 50, false)]);
        let mut rng = SimRng::from_seed(1, "dynasty:mortality");
        for _ in 0..500 {
            age_and_roll_mortality(&mut dynasty, &mut rng);
        }
        assert!(!dynasty.members[0].alive);
        assert_eq!(dynasty.members[0].age_years, 50);
    }

    #[test]
    fn an_ancient_character_eventually_dies_within_a_bounded_number_of_years() {
        // A very old character rolls at the top mortality bracket every
        // year; across many years the probability of surviving all of them
        // is negligible, so this should never need anywhere near the cap to
        // catch a death, unless the mortality roll is broken (e.g. always
        // false).
        let mut dynasty = dynasty_of(vec![character(1, 150, true)]);
        let mut rng = SimRng::from_seed(1, "dynasty:mortality");
        let mut died = false;
        for _ in 0..500 {
            age_and_roll_mortality(&mut dynasty, &mut rng);
            if !dynasty.members[0].alive {
                died = true;
                break;
            }
        }
        assert!(died, "an ancient character never died across 500 years");
    }

    #[test]
    fn same_seed_produces_the_same_death_ticks() {
        // Determinism test per the issue's testing note: same seed -> same
        // death ticks, across a population of characters at varying ages.
        fn run(seed: u64) -> Vec<(bool, u32)> {
            let mut dynasty = dynasty_of((0..30).map(|id| character(id, 20 + id, true)).collect());
            let mut rng = SimRng::from_seed(seed, "dynasty:mortality");
            for _ in 0..80 {
                age_and_roll_mortality(&mut dynasty, &mut rng);
            }
            dynasty
                .members
                .iter()
                .map(|c| (c.alive, c.age_years))
                .collect()
        }

        assert_eq!(run(2026), run(2026));
    }

    #[test]
    fn different_seeds_can_produce_different_death_ticks() {
        fn run(seed: u64) -> Vec<(bool, u32)> {
            let mut dynasty = dynasty_of((0..30).map(|id| character(id, 60 + id, true)).collect());
            let mut rng = SimRng::from_seed(seed, "dynasty:mortality");
            for _ in 0..100 {
                age_and_roll_mortality(&mut dynasty, &mut rng);
            }
            dynasty
                .members
                .iter()
                .map(|c| (c.alive, c.age_years))
                .collect()
        }

        assert_ne!(run(1), run(2));
    }
}
