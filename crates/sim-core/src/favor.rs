//! Character-to-character favors: a lightweight IOU mechanic and the first
//! non-wealth influence currency, per `GAME_DESIGN.md`'s "Politics without a
//! magic influence currency" section. A `Favor` always names exactly who
//! owes whom, so influence is traceable back to a specific relationship
//! rather than spent from an abstract pool.
//!
//! See `crate::career::promotion_chance_with_favors` for the mechanic issue
//! #67 wires this into: favors owed *to* a character (they can call them in)
//! nudge that character's own promotion odds, on top of the trait-based
//! modifier `crate::career::promotion_chance` already provides.

use serde::{Deserialize, Serialize};

use crate::dynasty::Dynasty;
use crate::world::EntityId;

/// A single outstanding favor: `debtor_id` owes `creditor_id`. `magnitude`
/// is how large the favor is (an arbitrary but consistent unit, larger
/// means more leverage) and is always positive; direction lives entirely in
/// which id is the creditor and which is the debtor, not in the sign of
/// `magnitude`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Favor {
    /// The character who is owed and can call this favor in.
    pub creditor_id: EntityId,
    /// The character who owes it.
    pub debtor_id: EntityId,
    /// Size of the favor. Must be finite and positive; see [`Favor::is_valid`].
    pub magnitude: f64,
}

impl Favor {
    /// A favor is valid when its magnitude is a finite positive number and
    /// it doesn't name the same character as both creditor and debtor (a
    /// character cannot owe themselves a favor).
    pub fn is_valid(&self) -> bool {
        self.magnitude.is_finite() && self.magnitude > 0.0 && self.creditor_id != self.debtor_id
    }
}

/// Total outstanding leverage `character_id` currently holds: the sum of
/// magnitudes of every favor owed *to* them, i.e. where they are the
/// creditor. Favors where `character_id` is the debtor don't count here,
/// they affect whichever creditor could call those in instead.
pub fn leverage_held_by(favors: &[Favor], character_id: EntityId) -> f64 {
    favors
        .iter()
        .filter(|favor| favor.creditor_id == character_id)
        .map(|favor| favor.magnitude)
        .sum()
}

/// Why granting a favor was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantFavorError {
    /// `creditor_id` doesn't resolve to any dynasty member.
    UnknownCreditor(EntityId),
    /// `debtor_id` doesn't resolve to any dynasty member.
    UnknownDebtor(EntityId),
    /// The ids resolved, but the resulting favor still fails
    /// [`Favor::is_valid`] (e.g. a non-positive magnitude, or the same
    /// character on both sides).
    InvalidFavor,
}

/// Create a favor of `magnitude`, owed by `debtor_id` to `creditor_id`.
/// Fails without constructing a `Favor` if either id doesn't reference an
/// existing member of `dynasty`, or if the result wouldn't satisfy
/// [`Favor::is_valid`]. This is what keeps issue #67's invariant ("a favor
/// never references a nonexistent character") true by construction for
/// every favor created this way; `invariants::check_invariants` still
/// re-checks it structurally in case a favor ever reaches `SimState` some
/// other way (e.g. a hand-edited save).
pub fn grant_favor(
    dynasty: &Dynasty,
    creditor_id: EntityId,
    debtor_id: EntityId,
    magnitude: f64,
) -> Result<Favor, GrantFavorError> {
    if !dynasty.members.iter().any(|c| c.id == creditor_id) {
        return Err(GrantFavorError::UnknownCreditor(creditor_id));
    }
    if !dynasty.members.iter().any(|c| c.id == debtor_id) {
        return Err(GrantFavorError::UnknownDebtor(debtor_id));
    }

    let favor = Favor {
        creditor_id,
        debtor_id,
        magnitude,
    };
    if !favor.is_valid() {
        return Err(GrantFavorError::InvalidFavor);
    }
    Ok(favor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynasty::Character;
    use crate::portrait::PortraitDescriptor;

    fn favor(creditor_id: EntityId, debtor_id: EntityId, magnitude: f64) -> Favor {
        Favor {
            creditor_id,
            debtor_id,
            magnitude,
        }
    }

    fn character(id: EntityId) -> Character {
        Character {
            id,
            name: format!("Member {id}"),
            age_years: 30,
            alive: true,
            wealth: 0.0,
            home_city: None,
            portrait: PortraitDescriptor::generate_for_character(0, id),
            traits: Vec::new(),
        }
    }

    fn dynasty_with_members(ids: &[EntityId]) -> Dynasty {
        Dynasty {
            name: "House Test".to_string(),
            head_character_id: ids[0],
            members: ids.iter().map(|&id| character(id)).collect(),
        }
    }

    #[test]
    fn a_favor_with_positive_finite_magnitude_and_distinct_parties_is_valid() {
        assert!(favor(1, 2, 5.0).is_valid());
    }

    #[test]
    fn a_favor_owed_to_oneself_is_invalid() {
        assert!(!favor(1, 1, 5.0).is_valid());
    }

    #[test]
    fn a_non_finite_or_non_positive_magnitude_is_invalid() {
        assert!(!favor(1, 2, 0.0).is_valid());
        assert!(!favor(1, 2, -3.0).is_valid());
        assert!(!favor(1, 2, f64::NAN).is_valid());
        assert!(!favor(1, 2, f64::INFINITY).is_valid());
    }

    #[test]
    fn leverage_sums_only_favors_where_the_character_is_the_creditor() {
        let favors = vec![
            favor(1, 2, 3.0),  // 1 is owed by 2
            favor(1, 3, 2.0),  // 1 is owed by 3
            favor(2, 1, 10.0), // 1 owes 2, doesn't count toward 1's leverage
        ];
        assert_eq!(leverage_held_by(&favors, 1), 5.0);
        assert_eq!(leverage_held_by(&favors, 2), 10.0);
    }

    #[test]
    fn a_character_with_no_favors_owed_to_them_has_zero_leverage() {
        let favors = vec![favor(1, 2, 3.0)];
        assert_eq!(leverage_held_by(&favors, 2), 0.0);
        assert_eq!(leverage_held_by(&favors, 999), 0.0);
    }

    #[test]
    fn serde_round_trips_exactly() {
        let original = favor(1, 2, 4.5);
        let json = serde_json::to_string(&original).expect("serialize");
        let round_tripped: Favor = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, round_tripped);
    }

    #[test]
    fn granting_a_favor_between_two_existing_members_succeeds() {
        let dynasty = dynasty_with_members(&[1, 2]);
        let granted = grant_favor(&dynasty, 1, 2, 3.0).expect("both ids exist");
        assert_eq!(granted, favor(1, 2, 3.0));
    }

    #[test]
    fn granting_a_favor_with_an_unknown_creditor_is_refused() {
        let dynasty = dynasty_with_members(&[1, 2]);
        assert_eq!(
            grant_favor(&dynasty, 999, 2, 3.0),
            Err(GrantFavorError::UnknownCreditor(999))
        );
    }

    #[test]
    fn granting_a_favor_with_an_unknown_debtor_is_refused() {
        let dynasty = dynasty_with_members(&[1, 2]);
        assert_eq!(
            grant_favor(&dynasty, 1, 999, 3.0),
            Err(GrantFavorError::UnknownDebtor(999))
        );
    }

    #[test]
    fn granting_a_favor_with_a_non_positive_magnitude_is_refused() {
        let dynasty = dynasty_with_members(&[1, 2]);
        assert_eq!(
            grant_favor(&dynasty, 1, 2, 0.0),
            Err(GrantFavorError::InvalidFavor)
        );
    }

    #[test]
    fn granting_a_favor_to_oneself_is_refused_even_though_the_id_exists() {
        let dynasty = dynasty_with_members(&[1, 2]);
        assert_eq!(
            grant_favor(&dynasty, 1, 1, 3.0),
            Err(GrantFavorError::InvalidFavor)
        );
    }
}
