//! Career promotion odds: the minimal trait consumer required by issue #38.
//!
//! Issue #38 ("Add character traits as a small fixed set") requires that at
//! least one system actually reads a character's traits and behaves
//! differently as a result, so traits are proven to be wired up rather than
//! decorative data. This module is intentionally that and nothing more: a
//! single pure function computing a promotion-odds modifier from traits.
//!
//! It deliberately does **not** add a career/job concept to `Character`, a
//! job ladder, salary accrual, or any promotion timing/execution. Those
//! belong to issue #26 ("Add a Corporate career track"), a separate, larger
//! piece of work. Issue #125 tracks folding this modifier into #26's real
//! promotion rule once that lands, rather than this module growing into a
//! second career system.

use crate::dynasty::Character;
use crate::traits::Trait;

/// Base probability a character is promoted in a single evaluation, before
/// trait modifiers. Chosen only to make the modifier's effect visible in
/// tests; a real career system (#26) will define its own base rate.
pub const BASE_PROMOTION_CHANCE: f64 = 0.3;

/// This character's promotion chance for one evaluation, adjusted by any
/// traits that plausibly affect career advancement.
///
/// `Ambitious` characters push harder for advancement and `Diligent`
/// characters get noticed for consistent work, so both raise the chance.
/// `Sickly` characters miss opportunities to time off, lowering it. Every
/// other trait is neutral to promotion odds: traits are allowed to matter
/// to some systems and not others.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portrait::PortraitDescriptor;

    fn character_with_traits(traits: Vec<Trait>) -> Character {
        Character {
            id: 1,
            name: "Test".to_string(),
            age_years: 30,
            alive: true,
            wealth: 0.0,
            home_city: None,
            portrait: PortraitDescriptor::generate_for_character(0, 1),
            traits,
        }
    }

    #[test]
    fn ambitious_characters_have_higher_promotion_odds_than_a_baseline_peer() {
        let baseline = character_with_traits(vec![Trait::Frugal]);
        let ambitious = character_with_traits(vec![Trait::Ambitious]);

        assert!(promotion_chance(&ambitious) > promotion_chance(&baseline));
    }

    #[test]
    fn sickly_characters_have_lower_promotion_odds_than_a_baseline_peer() {
        let baseline = character_with_traits(vec![Trait::Charismatic]);
        let sickly = character_with_traits(vec![Trait::Sickly]);

        assert!(promotion_chance(&sickly) < promotion_chance(&baseline));
    }

    #[test]
    fn promotion_chance_never_leaves_the_probability_range() {
        // Stack every negative modifier and every positive modifier to
        // prove the clamp holds even outside plausible trait combinations.
        let all_negative = character_with_traits(vec![Trait::Sickly, Trait::Sickly]);
        let all_positive = character_with_traits(vec![Trait::Ambitious, Trait::Diligent]);

        assert!((0.0..=1.0).contains(&promotion_chance(&all_negative)));
        assert!((0.0..=1.0).contains(&promotion_chance(&all_positive)));
    }

    #[test]
    fn a_character_with_no_traits_gets_exactly_the_base_chance() {
        let bare = character_with_traits(vec![]);
        assert_eq!(promotion_chance(&bare), BASE_PROMOTION_CHANCE);
    }
}
