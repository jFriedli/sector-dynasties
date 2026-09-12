//! Culture as dimensions, not a label.
//!
//! Per `GAME_DESIGN.md`'s "Culture as dimensions" section, culture should be
//! built from a small set of measurable dimensions that feed real systems
//! (nepotism tolerance, marriage, career prestige, entrepreneurship, political
//! behavior) rather than flavor text. This module defines the first
//! `CultureProfile`. It is intentionally not attached to `Country` yet; that
//! wiring is a follow-up issue.

use serde::{Deserialize, Serialize};

/// The valid range for every dimension on a [`CultureProfile`].
///
/// Each dimension is a bipolar axis: `-1.0` is the strongest lean toward the
/// low pole, `1.0` the strongest lean toward the high pole, and `0.0` is
/// neutral/balanced. Values outside this range, or non-finite values
/// (`NaN`/infinite), are invalid; see [`CultureProfile::is_valid`].
pub const DIMENSION_RANGE: std::ops::RangeInclusive<f64> = -1.0..=1.0;

/// A culture's position along a small set of dimensions.
///
/// Chosen to matter to the first consumers called out in the issue tracker:
/// nepotism tolerance, marriage, and entrepreneurship. Every dimension is an
/// `f64` in [`DIMENSION_RANGE`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CultureProfile {
    /// `-1.0` collectivist (the group's needs come first) to `1.0`
    /// individualist (personal autonomy and achievement come first). Shapes
    /// marriage (arranged vs. chosen) and entrepreneurship (going it alone
    /// vs. deferring to the family/group).
    pub individualism: f64,
    /// `-1.0` progressive (norms are expected to change) to `1.0`
    /// traditional (established norms and roles are expected to hold).
    /// Shapes marriage expectations and career prestige.
    pub tradition: f64,
    /// `-1.0` risk-averse to `1.0` risk-seeking. Shapes entrepreneurship
    /// (willingness to start or fund a new business) and career choice.
    pub risk_tolerance: f64,
    /// `-1.0` distrustful of institutions to `1.0` trusting of them. Shapes
    /// how much a character relies on formal institutions versus personal
    /// and family networks, including nepotism tolerance.
    pub institutional_trust: f64,
    /// `-1.0` nepotism is a scandal to `1.0` nepotism is normal, expected
    /// family loyalty. The direct input to the nepotism tolerance mechanic
    /// described in `GAME_DESIGN.md`.
    pub nepotism_tolerance: f64,
}

impl CultureProfile {
    /// A neutral profile with every dimension at `0.0`.
    pub const NEUTRAL: CultureProfile = CultureProfile {
        individualism: 0.0,
        tradition: 0.0,
        risk_tolerance: 0.0,
        institutional_trust: 0.0,
        nepotism_tolerance: 0.0,
    };

    /// True if every dimension is finite and within [`DIMENSION_RANGE`].
    pub fn is_valid(&self) -> bool {
        [
            self.individualism,
            self.tradition,
            self.risk_tolerance,
            self.institutional_trust,
            self.nepotism_tolerance,
        ]
        .into_iter()
        .all(|value| value.is_finite() && DIMENSION_RANGE.contains(&value))
    }
}

impl Default for CultureProfile {
    fn default() -> Self {
        Self::NEUTRAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_profile_is_valid() {
        assert!(CultureProfile::NEUTRAL.is_valid());
        assert!(CultureProfile::default().is_valid());
    }

    #[test]
    fn profile_at_the_range_bounds_is_valid() {
        let profile = CultureProfile {
            individualism: -1.0,
            tradition: 1.0,
            risk_tolerance: -1.0,
            institutional_trust: 1.0,
            nepotism_tolerance: -1.0,
        };
        assert!(profile.is_valid());
    }

    #[test]
    fn profile_outside_the_range_is_invalid() {
        let mut profile = CultureProfile::NEUTRAL;
        profile.individualism = 1.0001;
        assert!(!profile.is_valid());

        let mut profile = CultureProfile::NEUTRAL;
        profile.tradition = -1.5;
        assert!(!profile.is_valid());
    }

    #[test]
    fn non_finite_dimensions_are_invalid() {
        let mut profile = CultureProfile::NEUTRAL;
        profile.risk_tolerance = f64::NAN;
        assert!(!profile.is_valid());

        let mut profile = CultureProfile::NEUTRAL;
        profile.institutional_trust = f64::INFINITY;
        assert!(!profile.is_valid());

        let mut profile = CultureProfile::NEUTRAL;
        profile.nepotism_tolerance = f64::NEG_INFINITY;
        assert!(!profile.is_valid());
    }

    #[test]
    fn serde_round_trips_exactly() {
        let profile = CultureProfile {
            individualism: 0.42,
            tradition: -0.17,
            risk_tolerance: 0.9,
            institutional_trust: -0.6,
            nepotism_tolerance: 0.05,
        };

        let json = serde_json::to_string(&profile).expect("serialize");
        let round_tripped: CultureProfile = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(profile, round_tripped);
    }
}
