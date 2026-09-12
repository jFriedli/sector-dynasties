//! A small, rule-based utility AI for NPC career decisions.
//!
//! This is intentionally not a career simulation: it is a scoring function
//! over a handful of competing [`CareerOption`]s (e.g. "stay with the
//! current employer" vs. "take a new offer"), used once #26/#50 give NPCs
//! (non-dynasty characters) careers of their own. Per `GAME_DESIGN.md`,
//! NPC decisions must never call an LLM; this stays a small, auditable
//! scoring function instead.
//!
//! The score for each option combines rational factors (wage, prestige)
//! with `loyalty`, a non-rational factor: attachment to whichever option is
//! the character's current one, independent of whether it is the best-paid
//! or most prestigious choice. Without it this would just be profit
//! maximization, which `GAME_DESIGN.md` explicitly wants NPCs to avoid.
//!
//! The scoring is a pure function of its inputs, so it needs no RNG domain
//! of its own (see `crate::rng` for why other systems, like `economy` and
//! `portrait`, do need one): the same inputs always produce the same
//! decision, which is what makes it auditable in the first place. A future
//! change that adds randomness here (e.g. an impulsive-decision trait)
//! should give it its own named `SimRng` domain rather than reuse another
//! system's stream, the same as everywhere else in this crate.

use serde::{Deserialize, Serialize};

/// Weight of the normalized wage score in the overall utility.
pub const WAGE_WEIGHT: f64 = 0.5;
/// Weight of the prestige score in the overall utility.
pub const PRESTIGE_WEIGHT: f64 = 0.3;
/// Weight of the non-rational loyalty bonus, applied only to whichever
/// option matches [`CareerDecisionInput::current_option_id`].
pub const LOYALTY_WEIGHT: f64 = 0.4;

/// One concrete choice available to an NPC facing a career decision (e.g. a
/// specific employer, or "keep the current job").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CareerOption {
    /// Stable identifier for this option, e.g. an employer id or `"stay"`.
    /// Only meaningful within one [`CareerDecisionInput`].
    pub id: String,
    /// Expected wage under this option, in whatever currency unit the
    /// caller uses. Only relative magnitudes across the option set matter;
    /// this is normalized against the highest wage on offer before scoring.
    pub expected_wage: f64,
    /// Prestige of this option, in `[0.0, 1.0]`. Values outside that range
    /// are clamped.
    pub prestige: f64,
}

/// Everything the utility function needs to decide between an NPC's
/// competing career options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CareerDecisionInput {
    /// The competing options. Must contain at least one entry.
    pub options: Vec<CareerOption>,
    /// The `id` of the option representing the character's current
    /// situation (their current employer), if any. `None` means the
    /// character has no incumbent option to be loyal to, e.g. they are
    /// choosing a first job among several offers.
    pub current_option_id: Option<String>,
    /// The character's loyalty trait, in `[0.0, 1.0]`: how much inertia they
    /// have toward staying put regardless of a better rational offer.
    /// `0.0` is no loyalty at all (a pure profit-maximizer); `1.0` is
    /// maximally loyal. Values outside that range are clamped.
    pub loyalty: f64,
}

/// The outcome of scoring a [`CareerDecisionInput`]: which option won, and
/// the full per-option score breakdown for auditing or UI display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CareerDecision {
    /// `id` of the highest-scoring option.
    pub chosen_option_id: String,
    /// `(option id, utility score)` for every option that was considered,
    /// in the same order as [`CareerDecisionInput::options`].
    pub scores: Vec<(String, f64)>,
}

/// Score every option in `input` and return the winner.
///
/// Deterministic: calling this twice with equal inputs always produces
/// equal outputs, since it is a pure function with no hidden state or
/// randomness. Ties are broken in favor of the earlier option in
/// `input.options`.
///
/// # Panics
///
/// Panics if `input.options` is empty; a career decision needs at least one
/// option to choose from.
pub fn decide_career(input: &CareerDecisionInput) -> CareerDecision {
    assert!(
        !input.options.is_empty(),
        "decide_career requires at least one option"
    );

    let max_wage = input
        .options
        .iter()
        .map(|option| option.expected_wage)
        .fold(0.0_f64, f64::max);
    let loyalty = input.loyalty.clamp(0.0, 1.0);

    let scores: Vec<(String, f64)> = input
        .options
        .iter()
        .map(|option| {
            let wage_score = if max_wage > 0.0 {
                (option.expected_wage / max_wage).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let prestige_score = option.prestige.clamp(0.0, 1.0);
            let is_current = input.current_option_id.as_deref() == Some(option.id.as_str());
            let loyalty_bonus = if is_current {
                loyalty * LOYALTY_WEIGHT
            } else {
                0.0
            };

            let utility =
                wage_score * WAGE_WEIGHT + prestige_score * PRESTIGE_WEIGHT + loyalty_bonus;
            (option.id.clone(), utility)
        })
        .collect();

    let mut best_index = 0;
    for (index, (_, score)) in scores.iter().enumerate().skip(1) {
        if *score > scores[best_index].1 {
            best_index = index;
        }
    }

    CareerDecision {
        chosen_option_id: scores[best_index].0.clone(),
        scores,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stay_or_change(loyalty: f64) -> CareerDecisionInput {
        CareerDecisionInput {
            options: vec![
                CareerOption {
                    id: "stay".to_string(),
                    expected_wage: 100.0,
                    prestige: 0.3,
                },
                CareerOption {
                    id: "change".to_string(),
                    expected_wage: 140.0,
                    prestige: 0.35,
                },
            ],
            current_option_id: Some("stay".to_string()),
            loyalty,
        }
    }

    #[test]
    fn same_inputs_always_produce_the_same_decision() {
        let input = stay_or_change(0.6);
        let a = decide_career(&input);
        let b = decide_career(&input);
        assert_eq!(a, b);
    }

    #[test]
    fn a_purely_rational_npc_takes_the_better_offer() {
        let decision = decide_career(&stay_or_change(0.0));
        assert_eq!(decision.chosen_option_id, "change");
    }

    #[test]
    fn high_loyalty_keeps_a_worse_paying_job_over_a_better_offer() {
        // Same rational facts as the test above (better wage and prestige
        // elsewhere); only the loyalty trait differs. This is the
        // non-rational factor the issue asks for: it changes the outcome
        // even though "change" remains the higher-paying, higher-prestige
        // option in both cases.
        let decision = decide_career(&stay_or_change(1.0));
        assert_eq!(decision.chosen_option_id, "stay");
    }

    #[test]
    fn loyalty_bonus_only_applies_to_the_current_option() {
        let input = CareerDecisionInput {
            options: vec![
                CareerOption {
                    id: "a".to_string(),
                    expected_wage: 100.0,
                    prestige: 0.5,
                },
                CareerOption {
                    id: "b".to_string(),
                    expected_wage: 100.0,
                    prestige: 0.5,
                },
            ],
            current_option_id: None,
            loyalty: 1.0,
        };
        let decision = decide_career(&input);
        // Identical rational scores and no current option to be loyal to:
        // the tie-break falls to the first option, not whichever loyalty
        // would have favored.
        assert_eq!(decision.chosen_option_id, "a");
        assert_eq!(decision.scores[0].1, decision.scores[1].1);
    }

    #[test]
    fn out_of_range_loyalty_and_prestige_are_clamped_not_trusted() {
        let mut input = stay_or_change(5.0);
        input.options[0].prestige = 2.0;
        let decision = decide_career(&input);
        // A loyalty of 5.0 clamps to 1.0, same as the fully-loyal case
        // above, and an out-of-range prestige clamps to 1.0 rather than
        // blowing the utility scale past what a valid input could produce.
        assert_eq!(decision.chosen_option_id, "stay");
        for (_, score) in &decision.scores {
            assert!(score.is_finite());
            assert!(*score <= WAGE_WEIGHT + PRESTIGE_WEIGHT + LOYALTY_WEIGHT);
        }
    }

    #[test]
    #[should_panic(expected = "at least one option")]
    fn deciding_with_no_options_panics_rather_than_silently_choosing_nothing() {
        let input = CareerDecisionInput {
            options: vec![],
            current_option_id: None,
            loyalty: 0.5,
        };
        decide_career(&input);
    }

    #[test]
    fn scores_cover_every_option_in_input_order() {
        let input = stay_or_change(0.5);
        let decision = decide_career(&input);
        let ids: Vec<&str> = decision.scores.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, vec!["stay", "change"]);
    }

    #[test]
    fn serde_round_trips_exactly() {
        let decision = decide_career(&stay_or_change(0.6));
        let json = serde_json::to_string(&decision).expect("serialize");
        let round_tripped: CareerDecision = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decision, round_tripped);
    }
}
