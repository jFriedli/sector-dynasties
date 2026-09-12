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
//!
//! [`promotion_chance_with_favors`] is issue #67's consumer: outstanding
//! favors owed to a character (see `crate::favor`) are a legible,
//! relationship-based source of influence over their own promotion odds, on
//! top of the trait-based modifier below.

use crate::dynasty::Character;
use crate::favor::{self, Favor};
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

    fn favor_owed_to(creditor_id: u32, debtor_id: u32, magnitude: f64) -> Favor {
        Favor {
            creditor_id,
            debtor_id,
            magnitude,
        }
    }

    #[test]
    fn a_character_owed_a_favor_has_higher_promotion_odds_than_an_identical_peer_without_one() {
        // Same traits (and thus the same base `promotion_chance`) for both;
        // only the outstanding favor differs, so this isolates the favor
        // system's effect on the consuming mechanic per issue #67's
        // required comparative test.
        let owed = character_with_traits(vec![Trait::Frugal]);
        let mut not_owed = character_with_traits(vec![Trait::Frugal]);
        not_owed.id = 2;

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
        let debtor = character_with_traits(vec![Trait::Frugal]);
        let favors = vec![favor_owed_to(99, debtor.id, 5.0)];

        assert_eq!(
            promotion_chance_with_favors(&debtor, &favors),
            promotion_chance(&debtor)
        );
    }

    #[test]
    fn favor_bonus_never_reaches_the_cap_and_never_leaves_the_probability_range() {
        let character = character_with_traits(vec![Trait::Ambitious, Trait::Diligent]);
        let huge_favor = vec![favor_owed_to(character.id, 99, 1_000_000.0)];

        let chance = promotion_chance_with_favors(&character, &huge_favor);
        assert!((0.0..=1.0).contains(&chance));
        assert!(chance < promotion_chance(&character) + MAX_FAVOR_PROMOTION_BONUS);
    }

    #[test]
    fn multiple_favors_owed_to_the_same_character_stack() {
        let character = character_with_traits(vec![Trait::Frugal]);
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
}
