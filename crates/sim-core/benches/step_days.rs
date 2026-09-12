//! Headless performance baseline for `SimState::step_days`.
//!
//! There is no performance requirement yet, only a need for a repeatable
//! number to compare future changes against. This benchmarks a full
//! simulated year (`step_days(365)`) at two sector sizes: the bootstrap
//! default (what a real new game starts with) and a larger generated sector,
//! so a change that scales badly with sector size shows up as a widening gap
//! between the two rather than being hidden by only ever testing the small
//! default.
//!
//! Run with `cargo bench -p sim-core`. Not part of the default `cargo test`
//! pass or CI (see the `test = false` note on the `[[bench]]` entry in
//! `Cargo.toml`); benchmarks are slow and their absolute numbers are noisy
//! across machines, so they're for local, on-demand comparison only.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use sim_core::SimState;

/// Matches `state::STARTING_SYSTEM_COUNT`, the size a real new game starts
/// with. That constant is private to `sim-core`, so it's repeated here; if
/// it ever changes, update this to match so the "bootstrap" case stays
/// representative.
const BOOTSTRAP_SYSTEM_COUNT: u32 = 3;

/// An arbitrarily larger sector, well beyond anything the bootstrap slice
/// generates today, to give the benchmark something to scale against.
const LARGE_SYSTEM_COUNT: u32 = 60;

/// One simulated year per iteration: enough steps to exercise the weekly
/// settlement path (`economy::settle_week`) many times without making a
/// single benchmark run impractically slow.
const DAYS_PER_ITERATION: u32 = 365;

const SEED: u64 = 2026;

fn bench_step_days(c: &mut Criterion) {
    let mut group = c.benchmark_group("step_days");

    for (label, system_count) in [
        ("bootstrap_default", BOOTSTRAP_SYSTEM_COUNT),
        ("larger_sector", LARGE_SYSTEM_COUNT),
    ] {
        group.bench_with_input(
            BenchmarkId::new(label, system_count),
            &system_count,
            |b, &system_count| {
                // `iter_batched` excludes the setup (sector generation)
                // from the measured time, so this benchmarks step_days
                // itself, not worldgen. A fresh SimState per iteration
                // avoids one run's population/wealth drift from skewing
                // later iterations' step cost.
                b.iter_batched(
                    || SimState::new_with_system_count(SEED, system_count),
                    |mut state| {
                        state.step_days(DAYS_PER_ITERATION);
                        black_box(state.summary());
                    },
                    BatchSize::LargeInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_step_days);
criterion_main!(benches);
