# ADR 0001: Simulation core stack

Status: Accepted (2026-09-12)

## Context

The project brief proposed a starting hypothesis: Rust simulation core, TypeScript
frontend, React UI, a WASM bridge (wasm-bindgen), Vitest for frontend unit tests, Rust
unit/integration tests, and a headless CLI. This needed to be validated, not assumed,
against: agent productivity, Linux and Windows development, performance, determinism,
testing, debugging, build complexity, WASM compatibility, and long-term maintainability.

## Decision

The hypothesis is confirmed. The bootstrap slice (`crates/sim-core`, `crates/sim-cli`,
`crates/sim-wasm`, `web/`) proves it end to end:

- `sim-core` is a pure, dependency-light Rust crate (only `serde`/`serde_json`) with no
  I/O and no rendering. It builds unmodified to `wasm32-unknown-unknown`.
- `sim-cli` is a thin headless binary over `sim-core`, useful for scripted runs,
  soak tests, and reducing a bug report to a seed and a day count.
- `sim-wasm` is a thin `wasm-bindgen` bridge with no simulation rules of its own,
  built with `wasm-pack build --target web`.
- The React/TypeScript frontend (Vite + Vitest) loads the generated WASM package and
  calls into it directly.
- A same-seed run was verified bit-identical between the native CLI and the WASM
  bridge (see `crates/sim-core/src/state.rs` tests and the manual cross-check during
  bootstrap), confirming determinism survives the WASM boundary.
- Tauri is deferred until there's a reason to ship a native desktop build; nothing in
  the current architecture blocks adding it later, since the simulation core has no
  browser dependency.
- Playwright is deferred until there is enough real UI to justify end-to-end browser
  tests; Vitest covers unit-level frontend logic (including the copy lint) today.

## Consequences and known gaps

- **Toolchain weight for agents.** Rust + wasm-pack + Node is a heavier local setup
  than a single-language stack. This is accepted because it buys determinism,
  performance headroom for later population-scale simulation, and a real
  compile-time barrier between "simulation rule" and "rendering code" that a
  single-language stack would rely on discipline alone to maintain.
- **`serde_json` float round-trip.** By default, `serde_json` 1.0.151's float parser
  can be off by one ULP on reparse (verified during bootstrap: `0.9187639019828279`
  round-tripped to `0.918763901982828`), which silently breaks exact save/load
  equality tests. All three crates enable the `float_roundtrip` feature to fix this.
  Any future crate that parses `SimState` JSON must do the same.
- **No custom RNG dependency.** `sim-core::rng` hand-rolls splitmix64 (for seed
  splitting) and xoshiro256** (for generation) instead of depending on the `rand`
  crate. This keeps the dependency surface minimal and gives full control over the
  domain-separated stream design described in docs/ARCHITECTURE.md. If a future need
  (e.g. statistical quality audits, more algorithms) justifies it, swapping to `rand`
  is a contained change inside `sim-core::rng` only.
- **Windows validation is unverified by this bootstrap.** Everything above was run on
  Kali Linux. `rustup`, `cargo`, `wasm-pack`, and Node all support Windows, and none of
  the code uses Linux-only APIs, but a Windows agent should confirm the same commands
  work there and file a bug if not. See the `needs-windows-validation`-flavored notes
  in `AGENTS.md`.
- **Generated WASM output is not committed.** `web/src/wasm/` is produced by
  `npm run build:wasm` and gitignored. CI and any local frontend work must run that
  script (or `npm run build`, which does not itself rebuild wasm) before building or
  testing the frontend. See `docs/TESTING.md`.
- **Windows Tauri packaging spike (issue #103).** A minimal `web/src-tauri` Tauri v2
  shell (no IPC/commands, no simulation logic, `sim-core` untouched) wraps the
  existing `web/dist` static build. What's actually proven: the config, generated
  icons, and Rust shell compile and produce an NSIS-targeted bundle on GitHub's
  `windows-latest` CI runner (`.github/workflows/ci.yml`'s `windows-tauri` job
  running `npx tauri build`), and the same shell also compiles and launches without
  crashing under Xvfb on Linux (a sanity check only, not Windows evidence, since this
  work was done from Kali Linux). What is **not** proven: that the native window,
  the WebView2-backed webview, or the NSIS installer actually behave correctly when
  double-clicked on real Windows hardware. Nothing here contradicts the "Windows
  validation is unverified" gap above; it narrows it from "the whole toolchain" to
  specifically "runtime UX," tracked as a follow-up issue linked from the PR.

## Alternatives considered

- **TypeScript-only simulation core** (no Rust): simpler toolchain, but weaker
  guarantees around numeric determinism across JS engines and no natural boundary
  forcing "no simulation logic in the UI." Rejected for a project whose stated
  architecture goal is a rules/rendering split enforced by more than convention.
- **Bevy or another Rust game engine as the core**: pulls in a large dependency and an
  ECS/rendering opinion the project does not need yet, since the simulation is
  headless-first and the UI is a separate React app. Rejected for now; nothing here
  prevents adopting an ECS internally to `sim-core` later if entity counts demand it.
