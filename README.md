# Sector Dynasties

_"Sector Dynasties" is a placeholder codename, not a final title. Rename freely._

A seeded, deterministic sci-fi grand-strategy simulation about building (or losing)
an intergenerational family dynasty inside a living economic, political, and social
world, inspired in spirit by games like Crusader Kings but not cloning their
mechanics. You don't control a state or an army; you control the current head of a
family. See `GAME_DESIGN.md` for the full design intent.

This repository is bootstrapped for many autonomous agents (Codex, Claude, and
humans) to work on independently. If you're an agent: read `AGENTS.md` before
picking up an issue.

## Status

Foundation and architecture bootstrap (milestone M0) is complete: a deterministic
headless simulation core, a CLI runner, a WASM bridge, a minimal React UI, tests,
and CI all exist and pass. The vertical slice (M1) is the current milestone. See
`docs/ROADMAP.md`.

## Architecture at a glance

```
crates/sim-core   pure, headless simulation: world, dynasty, economy, time, rng
crates/sim-cli    headless CLI runner over sim-core (scripted runs, soak tests)
crates/sim-wasm   thin wasm-bindgen bridge, no game rules of its own
web/              React + TypeScript UI (Vite), calls into sim-wasm
docs/             architecture, roadmap, testing, content, and ADRs
```

`sim-core` has no I/O, no rendering, and builds unmodified to
`wasm32-unknown-unknown`. See `docs/ARCHITECTURE.md` and
`docs/adr/0001-simulation-core-stack.md` for why this stack was chosen and how it
was validated (including a same-seed cross-check proving the native CLI and the
WASM bridge produce bit-identical simulation state).

## Quick start

Rust (stable) plus the WASM target and wasm-pack:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
rustup target add wasm32-unknown-unknown
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

Run the simulation headlessly:

```bash
cargo test --workspace
cargo run -p sim-cli -- run --seed 42 --years 5
cargo run -p sim-cli -- run --preset tutorial --years 5   # named seed, see sim_core::presets
```

Run the UI (Node 20+; this repo was bootstrapped and tested against Node 24):

```bash
cd web
npm install
npm run build:wasm   # generates web/src/wasm/, gitignored, rebuild after touching Rust
npm run dev
```

Build the static production bundle (`web/dist/`, relocatable, deployable to
any static host including a GitHub Pages subpath) with `npm run build`; see
`docs/DEPLOYMENT.md` for details.

## Documentation map

- `AGENTS.md` — canonical rules for autonomous workers (issue claiming, worktrees,
  high-collision files, PR conventions).
- `CONTRIBUTING.md` — local setup and PR conventions for anyone.
- `GAME_DESIGN.md` — core design principles.
- `docs/ARCHITECTURE.md` — module map, determinism, persistence.
- `docs/DEPLOYMENT.md` — building and hosting the static `web/dist/` output
  (works from a domain root or a subpath, e.g. GitHub Pages).
- `docs/adr/` — architecture decision records.
- `docs/ROADMAP.md` — milestone sequencing.
- `docs/TESTING.md` — the full testing strategy and how to run each layer.
- `docs/CONTENT_GUIDE.md` — rules for player-facing writing and event content.
- `docs/ASSETS.md` — asset licensing and the portrait pipeline plan.
- `docs/STEAM_READINESS.md` — constraints that keep Steam an optional distribution
  layer rather than an architecture dependency.

## License

MIT, see `LICENSE`.
