//! A yearly chance for the dynasty head to have a child.
//!
//! Per issue #25 ("Add basic character birth for the dynasty head"), this
//! keeps the fertility model deliberately simple: once a year, if the
//! living dynasty head is within a plausible fertile age range, there is a
//! fixed chance a new `Character` is added to the dynasty as their child,
//! starting at age 0 in the head's home city. Richer relationship/marriage
//! modeling (a second parent, multiple partners, explicit family trees) is
//! out of scope here; see the #3 epic and issues #49/#23 for that territory.
//!
//! Like `crate::portrait` and `crate::traits`, the roll is a pure function
//! of the world seed, the head's id, and the simulated year, drawn from its
//! own dedicated `SimRng` stream (domain `"dynasty:birth:<head_id>:<year>"`).
//! Nothing here needs to persist across ticks: the same seed, head, and year
//! always produce the same outcome, independent of what any other system's
//! stream has consumed, and regardless of how the caller reached that year
//! (an uninterrupted run or resuming from a save).

use crate::dynasty::{Character, Dynasty};
use crate::portrait::PortraitDescriptor;
use crate::rng::SimRng;
use crate::traits;
use crate::world::EntityId;

/// Youngest age at which the head is considered fertile under this
/// deliberately simple model.
pub const MIN_FERTILE_AGE: u32 = 18;
/// Oldest age at which the head is still considered fertile.
pub const MAX_FERTILE_AGE: u32 = 45;

/// Chance, checked once per year, that a fertile living head has a child.
const ANNUAL_BIRTH_CHANCE: f64 = 0.18;

/// Small fixed set of given names for newly born characters. Kept short and
/// placeholder-plain on purpose, matching the founder's own bare "Founder"
/// name; a richer name generator is future content work, not this issue's
/// scope.
const CHILD_GIVEN_NAMES: [&str; 10] = [
    "Ari", "Bren", "Cael", "Dara", "Elin", "Fenn", "Iris", "Joss", "Kiran", "Lior",
];

/// The result of the yearly birth check: either nothing happened, or a new
/// character with this id was added to `Dynasty::members`.
pub type BirthOutcome = Option<EntityId>;

fn birth_domain(head_id: EntityId, year: u64) -> String {
    format!("dynasty:birth:{head_id}:{year}")
}

/// The next id to hand out for a new dynasty member: one past the highest
/// id currently in `dynasty.members`. Deterministic from the current
/// membership alone, so it never depends on creation order across systems.
fn next_character_id(dynasty: &Dynasty) -> EntityId {
    dynasty.members.iter().map(|c| c.id).max().unwrap_or(0) + 1
}

/// Check whether the dynasty head has a child this year, deterministically
/// from `world_seed`, the head's id, and `year`. If so, the new `Character`
/// is pushed onto `dynasty.members` and its id is returned.
///
/// Does nothing (returns `None`) if the dynasty has no head, the head is
/// not alive, or the head's age falls outside
/// `[MIN_FERTILE_AGE, MAX_FERTILE_AGE]`.
pub fn maybe_birth_child(world_seed: u64, dynasty: &mut Dynasty, year: u64) -> BirthOutcome {
    let (head_id, home_city) = {
        let head = dynasty.head()?;
        if !head.alive || !(MIN_FERTILE_AGE..=MAX_FERTILE_AGE).contains(&head.age_years) {
            return None;
        }
        (head.id, head.home_city)
    };

    let mut rng = SimRng::from_seed(world_seed, &birth_domain(head_id, year));
    if !rng.chance(ANNUAL_BIRTH_CHANCE) {
        return None;
    }

    let child_id = next_character_id(dynasty);
    let given_name = CHILD_GIVEN_NAMES[rng.pick_index(CHILD_GIVEN_NAMES.len())];
    let child = Character {
        id: child_id,
        name: given_name.to_string(),
        age_years: 0,
        alive: true,
        wealth: 0.0,
        home_city,
        portrait: PortraitDescriptor::generate_for_character(world_seed, child_id),
        traits: traits::generate_for_character(world_seed, child_id),
    };
    dynasty.members.push(child);

    Some(child_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dynasty_with_head(age_years: u32, alive: bool, home_city: Option<EntityId>) -> Dynasty {
        Dynasty {
            name: "House Test".to_string(),
            head_character_id: 1,
            members: vec![Character {
                id: 1,
                name: "Head".to_string(),
                age_years,
                alive,
                wealth: 1_000.0,
                home_city,
                portrait: PortraitDescriptor::generate_for_character(0, 1),
                traits: traits::generate_for_character(0, 1),
            }],
        }
    }

    /// Scan a wide range of seeds and years until at least one birth and at
    /// least one non-birth year are seen, so the tests below never depend
    /// on a single brittle magic seed to exercise both branches.
    fn find_seed_with_a_birth(head_age: u32) -> (u64, u64) {
        for seed in 0..500u64 {
            for year in 1..50u64 {
                let mut dynasty = dynasty_with_head(head_age, true, Some(7));
                if maybe_birth_child(seed, &mut dynasty, year).is_some() {
                    return (seed, year);
                }
            }
        }
        panic!("expected at least one (seed, year) combination to produce a birth");
    }

    #[test]
    fn same_seed_and_year_produce_the_same_birth_outcome() {
        let (seed, year) = find_seed_with_a_birth(30);

        let mut a = dynasty_with_head(30, true, Some(7));
        let mut b = dynasty_with_head(30, true, Some(7));
        let outcome_a = maybe_birth_child(seed, &mut a, year);
        let outcome_b = maybe_birth_child(seed, &mut b, year);

        assert_eq!(outcome_a, outcome_b);
        assert_eq!(a.members.len(), b.members.len());
        if let Some(child_id) = outcome_a {
            let child_a = a.members.iter().find(|c| c.id == child_id).unwrap();
            let child_b = b.members.iter().find(|c| c.id == child_id).unwrap();
            assert_eq!(child_a.name, child_b.name);
            assert_eq!(child_a.portrait, child_b.portrait);
            assert_eq!(child_a.traits, child_b.traits);
        }
    }

    #[test]
    fn a_birth_adds_a_child_at_age_zero_in_the_heads_home_city() {
        let (seed, year) = find_seed_with_a_birth(30);
        let mut dynasty = dynasty_with_head(30, true, Some(42));

        let child_id = maybe_birth_child(seed, &mut dynasty, year).expect("expected a birth");
        let child = dynasty.members.iter().find(|c| c.id == child_id).unwrap();

        assert_eq!(child.age_years, 0);
        assert!(child.alive);
        assert_eq!(child.home_city, Some(42));
        assert_ne!(child.id, dynasty.head_character_id);
    }

    #[test]
    fn a_new_childs_id_never_collides_with_an_existing_member() {
        let (seed, year) = find_seed_with_a_birth(30);
        let mut dynasty = dynasty_with_head(30, true, Some(7));
        dynasty.members.push(Character {
            id: 5,
            name: "Sibling".to_string(),
            age_years: 3,
            alive: true,
            wealth: 0.0,
            home_city: Some(7),
            portrait: PortraitDescriptor::generate_for_character(0, 5),
            traits: traits::generate_for_character(0, 5),
        });

        let before_ids: Vec<EntityId> = dynasty.members.iter().map(|c| c.id).collect();
        if let Some(child_id) = maybe_birth_child(seed, &mut dynasty, year) {
            assert!(!before_ids.contains(&child_id));
        }
    }

    #[test]
    fn a_dead_head_never_has_a_child() {
        let mut dynasty = dynasty_with_head(30, false, Some(7));
        for year in 1..50 {
            assert_eq!(maybe_birth_child(1, &mut dynasty, year), None);
        }
        assert_eq!(dynasty.members.len(), 1);
    }

    #[test]
    fn a_head_outside_the_fertile_age_range_never_has_a_child() {
        for &age in &[0, 5, 17, 46, 90] {
            let mut dynasty = dynasty_with_head(age, true, Some(7));
            for year in 1..200 {
                assert_eq!(maybe_birth_child(1, &mut dynasty, year), None);
            }
            assert_eq!(dynasty.members.len(), 1);
        }
    }

    #[test]
    fn a_head_with_no_dynasty_member_produces_no_birth() {
        let mut dynasty = Dynasty {
            name: "Empty".to_string(),
            head_character_id: 1,
            members: vec![],
        };
        assert_eq!(maybe_birth_child(1, &mut dynasty, 1), None);
    }

    #[test]
    fn different_years_can_diverge_even_with_the_same_seed() {
        let seed = 1;
        let outcomes: Vec<BirthOutcome> = (1..80)
            .map(|year| {
                let mut dynasty = dynasty_with_head(30, true, Some(7));
                maybe_birth_child(seed, &mut dynasty, year)
            })
            .collect();
        assert!(
            outcomes.iter().any(Option::is_some) && outcomes.iter().any(Option::is_none),
            "expected a mix of birth and non-birth years across a wide sample, got {outcomes:?}"
        );
    }
}
