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
