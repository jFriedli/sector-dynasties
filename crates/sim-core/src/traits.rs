//! Character traits: a small fixed set.
//!
//! Per issue #38 ("Add character traits as a small fixed set"), characters
//! are currently featureless beyond age/wealth/alive/portrait. `Trait` adds
//! a handful of personality tags that other systems (career progression,
//! future events) can read to differentiate characters mechanically rather
//! than only cosmetically. The list is intentionally kept small (see
//! [`ALL_TRAITS`]); growing it is ongoing content work, not a bootstrap
//! concern.
//!
//! Traits are assigned deterministically from the world seed and the
//! character's id via a dedicated `SimRng` stream (domain
//! `"traits:<id>"`), matching the pattern established by
//! `crate::portrait::PortraitDescriptor::generate_for_character`: the same
//! seed always produces the same traits for the same character, independent
//! of the order other systems draw from their own streams.

use serde::{Deserialize, Serialize};

use crate::rng::SimRng;
use crate::world::EntityId;

/// A single personality trait. Kept small and fixed per the issue's intent;
/// see the module doc comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trait {
    /// Pushes harder for career advancement.
    Ambitious,
    /// Spends less, saves more.
    Frugal,
    /// Prone to poor health, missing opportunities as a result.
    Sickly,
    /// Well liked, smooths social and career interactions.
    Charismatic,
    /// Consistent, reliable work, noticed by superiors.
    Diligent,
    /// Takes bigger risks, for better or worse.
    Reckless,
}

/// Every trait, in a stable order. Kept small (4-6 entries) for the
/// bootstrap per the issue's acceptance criteria.
pub const ALL_TRAITS: [Trait; 6] = [
    Trait::Ambitious,
    Trait::Frugal,
    Trait::Sickly,
    Trait::Charismatic,
    Trait::Diligent,
    Trait::Reckless,
];

/// The maximum number of traits a single character can have. A character
/// always has at least one.
pub const MAX_TRAITS_PER_CHARACTER: usize = 2;

/// Probability a character is assigned a second, distinct trait in
/// addition to their first.
const SECOND_TRAIT_CHANCE: f64 = 0.3;

/// Draw a character's traits from an already-derived `SimRng` stream. The
/// caller decides how that stream was seeded; see
/// [`generate_for_character`] for the standard per-character derivation.
///
/// Always returns at least one trait and never more than
/// [`MAX_TRAITS_PER_CHARACTER`], with no duplicates.
pub fn generate(rng: &mut SimRng) -> Vec<Trait> {
    let count = ALL_TRAITS.len();
    let first_index = rng.pick_index(count);
    let mut traits = vec![ALL_TRAITS[first_index]];

    if rng.chance(SECOND_TRAIT_CHANCE) {
        // Pick uniformly among the `count - 1` traits other than the one
        // already chosen, without a reject-and-retry loop: offset from
        // `first_index` by a random amount in `[1, count - 1]` (mod
        // `count`) always lands on a different index.
        let offset = 1 + rng.pick_index(count - 1);
        let second_index = (first_index + offset) % count;
        traits.push(ALL_TRAITS[second_index]);
    }

    traits
}

/// Deterministically generate the traits for `character_id` under
/// `world_seed`. Each character gets its own independent stream (domain
/// `"traits:<id>"`), so generating one character's traits never shifts
/// another's, regardless of creation order.
pub fn generate_for_character(world_seed: u64, character_id: EntityId) -> Vec<Trait> {
    let mut rng = SimRng::from_seed(world_seed, &format!("traits:{character_id}"));
    generate(&mut rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_and_character_id_generate_the_same_traits() {
        let a = generate_for_character(42, 7);
        let b = generate_for_character(42, 7);
        assert_eq!(a, b);
    }

    #[test]
    fn different_character_ids_can_diverge_even_with_the_same_seed() {
        let traits: Vec<Vec<Trait>> = (0..20).map(|id| generate_for_character(42, id)).collect();
        assert!(
            traits.windows(2).any(|pair| pair[0] != pair[1]),
            "expected at least some variation in traits across characters, got {traits:?}"
        );
    }

    #[test]
    fn generating_one_characters_traits_does_not_affect_a_sibling() {
        let alone = generate_for_character(99, 5);

        let _ = generate_for_character(99, 1);
        let _ = generate_for_character(99, 2);
        let _ = generate_for_character(99, 3);
        let after_siblings = generate_for_character(99, 5);

        assert_eq!(alone, after_siblings);
    }

    #[test]
    fn every_character_has_between_one_and_the_max_traits() {
        for id in 0..500 {
            let traits = generate_for_character(1, id);
            assert!(!traits.is_empty());
            assert!(traits.len() <= MAX_TRAITS_PER_CHARACTER);
        }
    }

    #[test]
    fn traits_for_a_character_never_contain_duplicates() {
        for id in 0..500 {
            let traits = generate_for_character(2026, id);
            let mut unique = traits.clone();
            unique.sort_by_key(|t| ALL_TRAITS.iter().position(|a| a == t).unwrap());
            unique.dedup();
            assert_eq!(
                unique.len(),
                traits.len(),
                "duplicate trait for character {id}: {traits:?}"
            );
        }
    }

    #[test]
    fn traits_round_trip_through_json_exactly() {
        let original = generate_for_character(2026, 3);
        let json = serde_json::to_string(&original).expect("traits always serialize");
        let restored: Vec<Trait> = serde_json::from_str(&json).expect("traits always deserialize");
        assert_eq!(original, restored);
    }
}
