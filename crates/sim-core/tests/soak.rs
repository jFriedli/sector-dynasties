//! Long-running soak test. Deliberately excluded from the default `cargo
//! test` run (see docs/TESTING.md): CI's normal pass stays fast, and this
//! runs on demand or on a schedule via `cargo test --test soak -- --ignored`.

use sim_core::invariants::check_invariants;
use sim_core::SimState;

#[test]
#[ignore = "soak test: run explicitly with `cargo test --test soak -- --ignored`"]
fn two_hundred_years_never_violates_invariants_or_diverges_by_seed() {
    for seed in [1u64, 2, 42, 1_000_000] {
        let mut state = SimState::new(seed);
        for day in 0..(200 * 360) {
            state.step_one_day();
            if day % 360 == 0 {
                let violations = check_invariants(&state);
                assert!(
                    violations.is_empty(),
                    "seed {seed} day {day}: {violations:?}"
                );
            }
        }
    }
}

/// Balancing regression test for issue #106. `check_invariants` alone only
/// catches structurally invalid values (NaN, out-of-range, dangling
/// references); it never notices a value that is valid but implausible,
/// like every city's unemployment drifting into a narrow corner of its
/// allowed range. This test runs the same four seeds as the invariant soak
/// test above for a 50-year stretch (the horizon issue #106 asks for) and
/// checks the resulting spread of `PopulationGroup::unemployment_rate`
/// across every city against the plausible-range bounds tuned in
/// `economy::UNEMPLOYMENT_OUTPUT_SENSITIVITY` and
/// `economy::UNEMPLOYMENT_RESIDUAL_NOISE_HALF_RANGE`. Before that tuning
/// pass, these same four seeds produced a standard deviation of ~0.102 and
/// a mean of ~0.179 with one city already pinned at the `0.35` ceiling;
/// the thresholds below sit between that pre-tuning behavior and the
/// post-tuning numbers (mean ~0.121, standard deviation ~0.085) so a future
/// change that meaningfully widens the spread again fails loudly here
/// rather than only showing up in a manual soak run.
#[test]
#[ignore = "soak test: run explicitly with `cargo test --test soak -- --ignored`"]
fn fifty_year_soak_keeps_unemployment_within_a_plausible_spread() {
    const YEARS: u32 = 50;
    const DAYS_PER_YEAR: u32 = 360;
    // Within 1.5 percentage points of either wall of the `[0.01, 0.35]`
    // clamp in `economy::settle_week`.
    const NEAR_BOUNDARY_EPSILON: f64 = 0.015;

    let mut rates: Vec<f64> = Vec::new();
    for seed in [1u64, 2, 42, 1_000_000] {
        let mut state = SimState::new(seed);
        state.step_days(YEARS * DAYS_PER_YEAR);

        assert!(
            check_invariants(&state).is_empty(),
            "seed {seed}: invariant violations after {YEARS} years"
        );

        for system in &state.sector.systems {
            for planet in &system.planets {
                for country in &planet.countries {
                    for city in &country.cities {
                        rates.push(city.population.unemployment_rate);
                    }
                }
            }
        }
    }

    assert!(!rates.is_empty(), "expected at least one city's rate");
    let n = rates.len() as f64;
    let mean = rates.iter().sum::<f64>() / n;
    let variance = rates.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    assert!(
        (0.03..0.25).contains(&mean),
        "mean unemployment across {} cities after {YEARS} years drifted to an \
         implausible level: {mean:.4}",
        rates.len()
    );
    assert!(
        std_dev < 0.10,
        "unemployment spread across {} cities after {YEARS} years is too wide \
         (std dev {std_dev:.4}), suggesting the noise/output-sensitivity \
         constants in economy.rs need retuning",
        rates.len()
    );

    let near_boundary = rates
        .iter()
        .filter(|r| **r <= 0.01 + NEAR_BOUNDARY_EPSILON || **r >= 0.35 - NEAR_BOUNDARY_EPSILON)
        .count();
    let near_boundary_fraction = near_boundary as f64 / n;
    assert!(
        near_boundary_fraction <= 0.25,
        "{near_boundary}/{} cities sat within {NEAR_BOUNDARY_EPSILON} of the \
         unemployment clamp after {YEARS} years ({near_boundary_fraction:.2} of \
         all cities), suggesting unemployment is pinning at an extreme rather \
         than settling into a plausible range",
        rates.len()
    );
}
