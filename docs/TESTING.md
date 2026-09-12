# Testing strategy

## Layers that exist today

- **Rust unit tests** (`crates/*/src/**/*.rs`, `#[cfg(test)]` modules): the bulk of
  coverage. Run with `cargo test --workspace`.
- **Rust integration/determinism tests** (`crates/sim-core/src/state.rs`,
  `crates/sim-core/src/worldgen.rs`, `crates/sim-core/src/economy.rs`): same-seed
  runs must produce byte-identical `SimState` JSON. This is the project's central
  correctness property; don't weaken it to make a test pass.
- **Serialization round-trip tests** (`state::tests::save_and_load_round_trips_exactly`,
  `resuming_from_a_save_continues_the_same_future_as_an_uninterrupted_run`): save/load
  must be exact, and resuming must continue the same future as never having paused.
- **Save versioning and migration tests** (`crates/sim-core/src/save.rs`,
  `state::tests::loading_*`): malformed JSON, a save missing `schema_version`, and a
  save from an unsupported future version are each rejected with a distinct
  `SaveError` rather than ever loading into a corrupted `SimState`. A synthetic
  old-version fixture proves the migration chain dispatches correctly even though
  there has only ever been one real schema version so far.
- **CLI native persistence test** (`crates/sim-cli/src/main.rs`,
  `saving_and_loading_a_file_continues_the_same_future_as_an_uninterrupted_run`):
  `sim-cli run --save` then `sim-cli load` round-trips through a real file on disk and
  continues the same future as an uninterrupted run.
- **Invariant checks** (`crates/sim-core/src/invariants.rs`): cheap structural
  validity checks (no NaN/negative population or treasury, dynasty head resolves to a
  real member) runnable every tick in debug/test builds. A violation is a bug.
  `stepping_never_produces_invariant_violations` runs these across a multi-year run.
- **A soak test** (`crates/sim-core/tests/soak.rs`): 200 simulated years across four
  seeds, checking invariants yearly. Marked `#[ignore]` so it does not run in the
  default fast pass; run it explicitly or from a scheduled workflow.
- **Frontend unit tests** (`web/tests/*.test.ts`, Vitest): currently the copy lint.
  Add more here as pure TS logic accumulates outside components.
- **The copy lint** (`web/tests/copy-lint.test.ts`): scans every exported string
  under `web/src/content/` for an em dash or semicolon and fails the build if found.
  Any new content module under `src/content/` is covered automatically as long as its
  exported values are strings, arrays, or plain objects.

## Running everything locally

```bash
# Rust
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Frontend (from web/)
npm run build:wasm   # regenerate the wasm package after touching sim-core/sim-wasm
npm run lint
npm run format:check
npx tsc -b
npm test
npm run build
```

`Scripts/test.sh` at the repo root runs the Rust half; see that script for the exact
invocation used by CI.

## What's intentionally not built yet

These are real gaps, not oversights, and are tracked as backlog issues rather than
built speculatively during bootstrap:

- Property-based tests (a `proptest`/`quickcheck`-style crate for the RNG and
  economy invariants would be a good early one).
- Playwright end-to-end tests: deferred until there's enough real UI to justify them
  (see ADR 0001). Don't add Playwright to prove a two-button page works.
- A real save migration: `save.rs` has the scaffolding and a synthetic-fixture test,
  but there has only ever been one real schema version so far. Add a real migration
  step and a fixture from the actual old shape the first time `SAVE_SCHEMA_VERSION`
  bumps.
- Performance/benchmark tests: no population-scale stress yet; add
  `criterion`-based benchmarks once a system's performance actually matters.
- Visual regression tests and fuzzing: valuable later, not before there's UI or
  content parsers worth protecting.

## Guidance for adding tests to new systems

- If a system introduces new random draws, give it (or reuse) a named `SimRng`
  domain and add a same-seed determinism test, the same shape as
  `economy::tests::same_seed_settlement_is_deterministic`.
- If a system introduces new state with a validity rule (a range, a required
  reference, a non-NaN requirement), add a check to `invariants::check_invariants`
  and a test that a violation is actually caught (see
  `invariants::tests::a_dangling_dynasty_head_is_caught` as the pattern: mutate the
  state into an invalid shape and assert the checker notices).
- If a system's output is part of `SimState`, the save/load round-trip test
  parameterization doesn't need duplicating per-system: it walks the whole struct
  already. Just make sure your new type derives `Serialize`/`Deserialize`.
- Don't add a soak-test-style long run for every system; the existing one already
  exercises the full `step_one_day` path. Add a new long run only if a system has a
  failure mode that only appears after many years (e.g. slow drift, accumulation).
