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
- **Desktop packaging spikes (issues #102, #103).** `web/src-tauri` is a Tauri v2
  shell (no simulation logic; `sim-core` untouched) wrapping the existing `web/`
  build in a native window, with a filesystem-backed `SaveSlotStore` (see
  `web/src/tauriFsSaveSlotStore.ts`) swapped in for IndexedDB when running inside
  it. #102 built this against Linux first; #103 layers Windows-target packaging on
  top of the same crate rather than forking a second one: `tauri.windows.conf.json`
  (Tauri's own platform-config-file mechanism, merged automatically on a Windows
  build) turns on NSIS bundling and points at an added `icon.ico`, and
  `.github/workflows/ci.yml` gets a `windows-tauri` job. What's actually proven:
  the config, icons, and Rust shell compile and produce an NSIS-targeted bundle on
  GitHub's real `windows-latest` CI runner (that job runs `npx tauri build` against
  a `web/dist` built ahead of time by the `frontend` job, not rebuilt on Windows —
  see below for why), and the same shell also compiles and launches without
  crashing under Xvfb on Linux (a sanity check only, not Windows evidence, since
  this work was done from Kali Linux). Issue #152 subsequently validated commit
  `ba8d3bb` on Windows 11 25H2 (build 26200.9445, x64): the CI-produced `web-dist`
  artifact bundled locally with Rust 1.98.1 and MSVC, the NSIS install completed
  with exit code 0 in current-user mode, and it created the expected app files and
  Start menu shortcut. The installed native window loaded the frontend through
  `http://tauri.localhost/` using the already-installed WebView2 runtime
  (152.0.4191.66). Advancing from year 0 to year 1 and selecting another city both
  updated the UI without WebView errors. The runtime-download path was not needed
  on this machine. The installer and installed executable are not Authenticode
  signed; the locally built installer also had no Mark of the Web, so this pass is
  not representative of SmartScreen reputation prompts for downloaded releases.
  Production signing is tracked in issue #161.
- **Found while building the above: the frontend didn't typecheck on a
  case-insensitive filesystem.** `web/src` had PascalCase-component/
  camelCase-logic filename pairs (`DynastyPanel.tsx`/`dynastyPanel.ts`, and
  three more) that only collided under `tsc -b` on Windows/macOS, not Linux,
  which is why this had never surfaced in CI before (the `frontend` job only
  ever ran on `ubuntu-latest`). #103 worked around it in its own CI job
  (consume a pre-built dist rather than rebuild the frontend on Windows)
  rather than fixing it, since the fix was a cross-cutting rename touching
  several other features' files. Fixed in #158 by renaming every logic
  module to a `*Logic.ts` suffix (`dynastyPanelLogic.ts`, and so on) instead
  of a same-name case pair; #103's CI workaround is left in place since it's
  still valid and slightly faster.

## Alternatives considered

- **TypeScript-only simulation core** (no Rust): simpler toolchain, but weaker
  guarantees around numeric determinism across JS engines and no natural boundary
  forcing "no simulation logic in the UI." Rejected for a project whose stated
  architecture goal is a rules/rendering split enforced by more than convention.
- **Bevy or another Rust game engine as the core**: pulls in a large dependency and an
  ECS/rendering opinion the project does not need yet, since the simulation is
  headless-first and the UI is a separate React app. Rejected for now; nothing here
  prevents adopting an ECS internally to `sim-core` later if entity counts demand it.
