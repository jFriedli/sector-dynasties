# Architecture

See [ADR 0001](adr/0001-simulation-core-stack.md) for how this stack was chosen and
validated. This document describes the module map and the rules that keep the
architecture from rotting as more agents add systems.

## The hard boundary: simulation core vs. everything else

`crates/sim-core` is the only place authoritative game rules may live. It has no
I/O, no rendering, and (checked by CI) builds unmodified to `wasm32-unknown-unknown`.
Everything else consumes it:

```
crates/sim-core   pure simulation: world, dynasty, economy, time, rng, invariants
crates/sim-cli    headless binary over sim-core, for scripted runs and soak tests
crates/sim-wasm   thin wasm-bindgen bridge, zero game rules of its own
web/              React/TypeScript UI, calls into sim-wasm, renders state
```

If you find yourself writing a game rule (a formula, a threshold, a decision) inside
`sim-wasm` or `web/`, it belongs in `sim-core` instead. The UI's job is to call
`step`, read the resulting state or summary, and render it.

## Module map inside sim-core

- `time.rs` — `SimClock`. One tick = one day. Weekly/monthly/yearly systems check
  the clock rather than being driven by separate timers, so the world is never fully
  recomputed every tick. See "Simulation time" below.
- `rng.rs` — `SimRng`, a deterministic, dependency-free RNG. See "Determinism" below.
- `world.rs` — the Sector → StarSystem → Planet → Country → City hierarchy and
  `PopulationGroup`. `Country::government` is a `GovernmentProfile` of independent
  component values (federalism, franchise, economic liberalism, press freedom)
  rather than one enum, per the project's government-from-institutions goal. Add new
  dimensions as new fields; don't collapse them into a label until display time.
- `worldgen.rs` — seeded, reproducible generation of a starting sector. Deliberately
  small today (a handful of systems/planets/countries/cities); growing this into real
  procedural history is backlog work under the `worldgen` label.
- `dynasty.rs` — `Character` and `Dynasty`. The player always controls
  `Dynasty::head()`; other members are simulated but not directly controlled.
- `economy.rs` — weekly settlement over cities. Intentionally minimal (no goods,
  production chains, or trade routes yet); it exists to prove the tick/frequency
  architecture. Real production chains are `economy`/`trade` backlog work.
- `invariants.rs` — `check_invariants(&SimState)`, cheap enough to run every tick in
  debug/test builds. A violation is always a bug, never a game event. Extend this
  whenever you add state that has a validity rule (a range, a non-NaN requirement, a
  foreign-key-style reference that must resolve).
- `save.rs` — save format version detection and migration (`load_and_migrate`,
  `SaveError`). See "Persistence" below; this is the one place a schema-bumping change
  needs a new migration step.
- `state.rs` — `SimState`, the single authoritative struct, and its `step_one_day` /
  `step_days`. Owns the per-domain RNG streams that need to persist across ticks (so
  far, just `economy_rng`). Also owns save/load (`to_json`/`from_json`) and
  `StateSummary`, a compact view for UI/CLI callers that don't want to walk the full
  hierarchy.

## Simulation time

One tick equals one in-world day (`time::DAYS_PER_WEEK/MONTH/YEAR` define the other
units in ticks). `SimState::step_one_day` always advances the clock, then asks it
`is_week_boundary()` / `is_month_boundary()` / `is_year_boundary()` before running the
corresponding system. Add a new lower-frequency system (population, culture,
demographics) the same way: check the clock, don't add a second timer.

## Determinism

`rng.rs` implements two small, well-known, dependency-free primitives: splitmix64 to
split a world seed into independent stream seeds, and xoshiro256** as the generator.
Every simulation domain that needs randomness gets its own named `SimRng::from_seed
(world_seed, "domain")` stream (`"worldgen"`, `"economy"`, `"dynasty"`, and so on, with
`"events:character:<id>"`-style suffixes for per-entity streams). This means adding or
removing a random draw in one system never shifts the sequence any other system sees.
`rng.rs`'s tests demonstrate and pin this property; keep it true for any new domain.

Any stream that must persist across ticks (i.e. isn't reseeded from scratch each time)
has to live in `SimState` so save/load resumes the same future as an uninterrupted run
— see `state.rs`'s `resuming_from_a_save_continues_the_same_future_as_an_uninterrupted_run`
test for the property being protected.

## Persistence

`SimState` derives `Serialize`/`Deserialize` and is the entire save format; there is
no separate save schema to keep in sync. Serialization is completely storage-agnostic:
`to_json`/`from_json` produce and consume a plain `String`, so a save's destination
(a file, a browser storage adapter, an in-memory buffer in a test) is entirely a
caller concern. `sim-core` and `sim-wasm` never touch a filesystem, `localStorage`, or
IndexedDB directly; only `sim-cli` (native files, see below) and eventually a
browser-side storage adapter in `web/` (backlog, not built yet: `#34`/`#89` cover the
save-slot UI, IndexedDB should back it for anything beyond the trivial bootstrap slice,
never `localStorage`, since campaign saves are not small) are I/O.

`crates/sim-core/src/save.rs` owns save format versioning: `SAVE_SCHEMA_VERSION` in
`state.rs` is the current schema, and `save::load_and_migrate` checks it before
`state.rs` does the final typed deserialization. Three outcomes: a JSON `Value` at the
current version is deserialized directly; a `Value` at an older known version runs
through `save::migrate_step` (one match arm per `from_version`, additive only) until it
reaches the current version; a `Value` newer than `SAVE_SCHEMA_VERSION` is rejected as
`SaveError::UnsupportedFutureVersion` rather than guessed at. Anything that isn't valid
JSON, or is JSON missing a required field (including `schema_version` itself), is
`SaveError::Corrupt`. Loading never silently produces a corrupted `SimState`; add a
migration test with a synthetic old-version fixture (see `save.rs`'s tests) any time
you actually bump `SAVE_SCHEMA_VERSION`.

`crates/sim-cli` proves native filesystem persistence works end to end today:
`sim-cli run --save <path>` writes a save, and `sim-cli load --path <path> --years <n>`
resumes it and advances further. This is also the reproduction workflow for a bug
report: a seed and a day count, or an exact save file and a day count, reproduces the
state headlessly without touching the UI or a graphical editor.

`serde_json`'s `float_roundtrip` feature is required (see ADR 0001) for save/load to
be exact; it's already enabled in every crate that touches `SimState` JSON. Keep it
enabled in any new crate that does the same.

## Windows, desktop, and Steam

CI's `windows` job builds and tests the whole Rust workspace (and the `wasm32`
target) on `windows-latest`; that's an automated proxy for "does this compile and
pass on Windows," not a substitute for a human or agent actually running the editor
or a packaged build there (see `AGENTS.md`'s Windows/Linux split). Nothing in
`sim-core`, `sim-cli`, or `sim-wasm` uses a Linux-only API. Tauri (for a native
desktop build) and Steam (for distribution) are both deferred until there's a
packaging reason to add them; see `docs/STEAM_READINESS.md` for the constraints that
keep Steam an optional distribution layer rather than an architecture dependency, and
issues `#101`-`#103` for the packaging spikes.

## Frontend

`web/` is a Vite + React + TypeScript app. `web/src/wasm/` is generated by
`npm run build:wasm` (a `wasm-pack build --target web` wrapper) and gitignored;
rebuild it after touching `sim-core` or `sim-wasm`. `web/src/content/` holds every
player-facing string so the copy lint (`web/tests/copy-lint.test.ts`) can scan them
in one place; see `docs/CONTENT_GUIDE.md`.

## Adding a new hierarchy level or entity

Prefer adding fields/variants to the existing `world.rs` types over introducing a
parallel hierarchy. If a genuinely new top-level concept is needed (e.g.
organizations, businesses), give it its own module next to `world.rs`/`dynasty.rs`,
reference existing entities by `EntityId`, and add its invariants to
`invariants.rs`. Avoid circular strong ownership between modules; prefer referencing
by ID and looking up through `SimState` when a system needs cross-module data.
