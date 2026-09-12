//! Union power's effect on business labor costs: the minimal business cost
//! consumer required by issue #71.
//!
//! Businesses don't exist as a persisted, owned concept yet. Issue #27 ("Add
//! a Mining Company business archetype") is where the first real `Business`
//! type lands, settled weekly and owned by a dynasty; see epic #6 for where
//! that grows (logistics, finance, mergers, bankruptcy). This module is
//! intentionally that and nothing more: a couple of representative
//! archetypes and a pure labor-cost/net-income calculation proving
//! `crate::world::Country::union_power` actually raises labor costs (and so
//! lowers net income) rather than sitting decorative. Issue #27, once it
//! introduces real settled businesses, should read `Country::union_power`
//! through `labor_cost_fraction`/`net_income` below rather than this module
//! growing into a second business system.

/// A broad category of business, each with its own baseline labor intensity
/// before union power adjusts it. Kept to a couple of representative
/// archetypes rather than the full roster epic #6 will eventually need.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusinessArchetype {
    /// Labor-heavy: a mining company or factory floor, where wages already
    /// make up a large share of revenue even before union power.
    MiningCompany,
    /// Capital-heavy: a financial firm, where labor is a smaller share of
    /// revenue and union power moves net income less.
    FinanceFirm,
}

impl BusinessArchetype {
    /// Baseline labor cost as a fraction of revenue at `union_power == 0.0`
    /// (no organized labor). See `labor_cost_fraction` for how union power
    /// scales this upward.
    fn base_labor_cost_fraction(self) -> f64 {
        match self {
            BusinessArchetype::MiningCompany => 0.35,
            BusinessArchetype::FinanceFirm => 0.15,
        }
    }
}

/// How much larger an archetype's baseline labor cost fraction can grow,
/// proportionally, under fully organized labor (`union_power == 1.0`).
/// Scaling by the archetype's own baseline (rather than by its headroom to
/// `1.0`) means a labor-heavy archetype, which already spends more of its
/// revenue on wages, sees the larger *absolute* swing from unionizing, not
/// the smaller one. Chosen so the swing is clearly visible without ever
/// being able to push labor cost past 100% of revenue.
const UNION_POWER_LABOR_COST_BONUS: f64 = 0.4;

/// This archetype's labor cost as a fraction of revenue at the given
/// `union_power`. `union_power` is expected to already be within
/// `0.0..=1.0` (see `invariants.rs`), but the result is clamped defensively
/// here too so a corrupt value can't push the labor cost fraction outside a
/// sane range. Higher union power raises the fraction of revenue that must
/// go to wages, the hook proving `Country::union_power` affects a
/// calculation rather than sitting decorative.
pub fn labor_cost_fraction(archetype: BusinessArchetype, union_power: f64) -> f64 {
    let union_power = union_power.clamp(0.0, 1.0);
    let base = archetype.base_labor_cost_fraction();
    (base * (1.0 + union_power * UNION_POWER_LABOR_COST_BONUS)).clamp(0.0, 1.0)
}

/// Net income for one settlement period: `revenue` minus labor cost minus
/// `running_cost` (any other non-labor running cost, e.g. facilities or
/// upkeep; pass `0.0` for a pure labor-cost comparison). `revenue` is
/// defensively clamped to non-negative, since a business with no revenue
/// this period has no labor cost to pay against it either.
pub fn net_income(
    archetype: BusinessArchetype,
    revenue: f64,
    union_power: f64,
    running_cost: f64,
) -> f64 {
    let revenue = revenue.max(0.0);
    let labor_cost = revenue * labor_cost_fraction(archetype, union_power);
    revenue - labor_cost - running_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labor_cost_fraction_grows_with_union_power() {
        let unorganized = labor_cost_fraction(BusinessArchetype::MiningCompany, 0.0);
        let organized = labor_cost_fraction(BusinessArchetype::MiningCompany, 1.0);
        assert_eq!(unorganized, 0.35);
        assert!(organized > unorganized);
        assert!(organized <= 1.0);
    }

    #[test]
    fn labor_cost_fraction_clamps_a_corrupt_union_power_value() {
        let low = labor_cost_fraction(BusinessArchetype::MiningCompany, -5.0);
        let high = labor_cost_fraction(BusinessArchetype::MiningCompany, 5.0);
        assert_eq!(
            low,
            labor_cost_fraction(BusinessArchetype::MiningCompany, 0.0)
        );
        assert_eq!(
            high,
            labor_cost_fraction(BusinessArchetype::MiningCompany, 1.0)
        );
        assert!(high <= 1.0);
    }

    #[test]
    fn higher_union_power_lowers_net_income_holding_everything_else_constant() {
        // Same archetype, same revenue and running cost, different
        // union-power levels: the required comparative test from the
        // issue's Testing section.
        let low_union = net_income(BusinessArchetype::MiningCompany, 10_000.0, 0.0, 500.0);
        let high_union = net_income(BusinessArchetype::MiningCompany, 10_000.0, 1.0, 500.0);

        assert!(
            high_union < low_union,
            "full union power ({high_union}) should not out-earn no organized \
             labor ({low_union}) holding revenue and running cost constant"
        );
    }

    #[test]
    fn a_capital_heavy_archetype_is_less_sensitive_to_union_power_than_a_labor_heavy_one() {
        let mining_swing = labor_cost_fraction(BusinessArchetype::MiningCompany, 1.0)
            - labor_cost_fraction(BusinessArchetype::MiningCompany, 0.0);
        let finance_swing = labor_cost_fraction(BusinessArchetype::FinanceFirm, 1.0)
            - labor_cost_fraction(BusinessArchetype::FinanceFirm, 0.0);

        assert!(
            finance_swing < mining_swing,
            "finance's labor-cost swing ({finance_swing}) should be smaller than \
             mining's ({mining_swing})"
        );
    }

    #[test]
    fn net_income_never_produces_a_non_finite_result_for_valid_inputs() {
        for union_power in [0.0, 0.25, 0.5, 0.75, 1.0] {
            for archetype in [
                BusinessArchetype::MiningCompany,
                BusinessArchetype::FinanceFirm,
            ] {
                let income = net_income(archetype, 1_000.0, union_power, 100.0);
                assert!(income.is_finite());
            }
        }
    }
}
